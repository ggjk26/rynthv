use super::LyricFrontend;
use crate::{
    model::{Language, PronunciationDictionary},
    synthesis::{
        plan::NoteInput,
        voice::{PreparedTree, Result, VoiceError},
    },
};

pub(super) fn resolve_note(
    note: &NoteInput,
    language: Language,
    frontend: &LyricFrontend,
    dictionary: Option<&PronunciationDictionary>,
    tree: &PreparedTree,
) -> Result<Vec<Vec<String>>> {
    // Synthesizer V uses "br" as a breath lyric. Resolve it against the
    // actual voice phoneme inventory rather than through the word dictionary.
    if note.lyrics.trim().eq_ignore_ascii_case("br")
        && note.phonemes.trim().is_empty()
    {
        let supported = tree.resolve_phone(language.id(), "br")
            .is_ok_and(|phone| !phone.used_silence_fallback);
        return Ok(vec![vec![if supported { "br" } else { "sil" }.to_owned()]]);
    }
    let explicit = (!note.phonemes.trim().is_empty())
        .then_some(note.phonemes.as_str())
        .or_else(|| {
            dictionary
                .filter(|dict| dict.language == language)
                .and_then(|dict| dict.entries.get(note.lyrics.trim()).map(String::as_str))
        });
    let syllables = if let Some(phones) = explicit {
        match split_syllables(phones, language, tree) {
            Ok(syllables) => syllables,
            Err(_) => {
                // A malformed explicit pronunciation is discarded as a whole:
                // never send only its valid prefix to the duration model.
                // Re-read the lyric; when that too is unavailable, silence
                // this note instead of breaking synthesis for the whole song.
                match frontend.resolve(language, &note.lyrics) {
                    Ok(syllables) => syllables,
                    Err(_) => vec![vec!["sil".to_owned()]],
                }
            }
        }
    } else {
        // Unknown ordinary lyrics (including unsupported silent tokens)
        // should affect only this note, not abort the whole phrase/song.
        frontend.resolve(language, &note.lyrics)
            .unwrap_or_else(|_| vec![vec!["sil".to_owned()]])
    };
    if note
        .attributes
        .get("evenSyllableDuration")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        Ok(vec![syllables.into_iter().flatten().collect()])
    } else {
        Ok(syllables)
    }
}

fn split_syllables(
    phones: &str,
    language: Language,
    tree: &PreparedTree,
) -> Result<Vec<Vec<String>>> {
    let tokens: Vec<String> = phones.split_whitespace().map(str::to_owned).collect();
    let mut nuclei = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let phone = tree.resolve_phone(language.id(), token)?;
        if phone.used_silence_fallback && token != "sil" {
            return Err(VoiceError::new(format!("无效音素：{token}")));
        }
        if matches!(phone.source_class, Some("vowel" | "diphthong")) {
            nuclei.push(index);
        }
    }
    let mut parts = Vec::new();
    let mut start = 0;
    for &end in nuclei.iter().take(nuclei.len().saturating_sub(1)) {
        parts.push(tokens[start..=end].to_vec());
        start = end + 1;
    }
    parts.push(tokens[start..].to_vec());
    Ok(parts)
}
