use crate::synthesis::voice::{Result, VoiceError};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug)]
pub(super) struct Pinyin {
    entries: HashMap<String, Vec<String>>,
    max_key_bytes: usize,
}

impl Pinyin {
    pub fn load(path: &Path) -> Result<Self> {
        let mut entries = HashMap::new();
        let mut max_key_bytes = 0;
        for (index, line) in fs::read_to_string(path)?.lines().enumerate() {
            let line = line.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.split_whitespace();
            let key = fields.next().expect("nonempty line");
            let phones: Vec<String> = fields.map(str::to_owned).collect();
            if phones.is_empty() {
                return Err(VoiceError::new(format!(
                    "{}:{}: Mandarin syllable has no phonemes",
                    path.display(),
                    index + 1
                )));
            }
            max_key_bytes = max_key_bytes.max(key.len());
            if let Some(previous) = entries.insert(key.to_owned(), phones.clone())
                && previous != phones
            {
                return Err(VoiceError::new(format!(
                    "Conflicting Mandarin phoneme entries for {key:?}"
                )));
            }
        }
        if entries.is_empty() {
            return Err(VoiceError::new("Mandarin syllable dictionary is empty"));
        }
        Ok(Self {
            entries,
            max_key_bytes,
        })
    }


    /// Offline mappings are admitted only for syllables found in the bundled Hanzi data.
    pub(super) fn builtin(source: &str) -> Self {
        let mut entries = HashMap::new();
        let mut max_key_bytes = 0;
        for line in source.lines() {
            let Some((_, reading)) = line.split_once(": ") else {
                continue;
            };
            let spelling = reading
                .split('#').next().unwrap_or_default().trim()
                .split(',').next().unwrap_or_default().trim();
            let Some(mut key) = normalize(spelling) else {
                continue;
            };
            if key.ends_with(['0', '1', '2', '3', '4', '5']) {
                key.pop();
            }
            if let Some(phones) = builtin_phonemes(&key) {
                max_key_bytes = max_key_bytes.max(key.len());
                entries.entry(key).or_insert(phones);
            }
        }
        Self { entries, max_key_bytes }
    }

    pub fn syllable(&self, text: &str) -> Result<Vec<String>> {
        let mut key = normalize(text).ok_or_else(|| super::unknown(text))?;
        if key.ends_with(['0', '1', '2', '3', '4', '5']) {
            key.pop();
        }
        self.entries
            .get(&key)
            .cloned()
            .ok_or_else(|| super::unknown(text))
    }

    pub fn resolve(&self, text: &str) -> Result<Vec<Vec<String>>> {
        let key = normalize(text).ok_or_else(|| super::unknown(text))?;
        let bytes = key.as_bytes();
        let mut next = vec![None; key.len() + 1];
        next[key.len()] = Some((key.len(), key.len()));
        for start in (0..key.len()).rev() {
            if !bytes[start].is_ascii_lowercase() {
                continue;
            }
            let limit = key.len().min(start + self.max_key_bytes);
            // Prefer a complete syllable, but only when its suffix can also be read.
            for end in ((start + 1)..=limit).rev() {
                if !self.entries.contains_key(&key[start..end]) {
                    continue;
                }
                let after = if bytes
                    .get(end)
                    .is_some_and(|byte| matches!(byte, b'0'..=b'5'))
                {
                    end + 1
                } else {
                    end
                };
                if next[after].is_some() {
                    next[start] = Some((end, after));
                    break;
                }
            }
        }
        let mut start = 0;
        let mut result = Vec::new();
        while start < key.len() {
            let (end, after) = next[start].ok_or_else(|| super::unknown(text))?;
            result.push(self.entries[&key[start..end]].clone());
            start = after;
        }
        Ok(result)
    }
}

/// Tone spelling does not change the toneless phoneme inventory; the score supplies pitch.
fn normalize(text: &str) -> Option<String> {
    let value = text.to_lowercase().replace("u:", "v");
    let mut key = String::with_capacity(value.len());
    for ch in value.chars() {
        key.push(match ch {
            'ā' | 'á' | 'ǎ' | 'à' => 'a',
            'ē' | 'é' | 'ě' | 'è' => 'e',
            'ī' | 'í' | 'ǐ' | 'ì' => 'i',
            'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
            'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
            'ü' | 'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' => 'v',
            'a'..='z' | '0'..='5' => ch,
            _ => return None,
        });
    }
    (!key.is_empty()).then_some(key)
}


/// Pinyin uses a fixed, known SV Mandarin X-SAMPA inventory.
fn builtin_phonemes(spelling: &str) -> Option<Vec<String>> {
    let rewritten = match spelling {
        "yi" => "i", "ya" => "ia", "yao" => "iao", "ye" => "ie",
        "you" => "iu", "yan" => "ian", "yang" => "iang",
        "yin" => "in", "ying" => "ing", "yong" => "iong",
        "yu" => "v", "yue" => "ve", "yuan" => "van", "yun" => "vn",
        "wu" => "u", "wa" => "ua", "wai" => "uai", "wan" => "uan",
        "wang" => "uang", "wei" => "ui", "wen" => "un",
        "weng" => "ueng", "wo" => "uo",
        _ => spelling,
    };
    let (initial, mut final_spelling) = [
        ("zh", r"ts`"), ("ch", r"ts`h"), ("sh", r"s`"),
        ("b", "p"), ("p", "ph"), ("m", "m"), ("f", "f"),
        ("d", "t"), ("t", "th"), ("n", "n"), ("l", "l"),
        ("g", "k"), ("k", "kh"), ("h", "x"),
        ("j", r"ts\"), ("q", r"ts\h"), ("x", r"s\"),
        ("r", r"z`"), ("z", "ts"), ("c", "tsh"), ("s", "s"),
    ]
    .into_iter()
    .find_map(|(pinyin, phone)| rewritten.strip_prefix(pinyin).map(|rest| (phone, rest)))
    .unwrap_or(("", rewritten));
    if matches!(initial, r"ts`" | r"ts`h" | r"s`" | r"z`") && final_spelling == "i" {
        final_spelling = "retroflex-i";
    } else if matches!(initial, "ts" | "tsh" | "s") && final_spelling == "i" {
        final_spelling = "apical-i";
    } else if matches!(initial, r"ts\" | r"ts\h" | r"s\") {
        final_spelling = match final_spelling {
            "u" => "v", "ue" => "ve", "uan" => "van", "un" => "vn",
            other => other,
        };
    }
    let final_phones = match final_spelling {
        "a" => "a", "o" => "o", "e" => "7", "i" => "i",
        "u" => "u", "v" => "y",
        "retroflex-i" => "i`", "apical-i" => r"i\",
        "ai" => r"a :\i", "ei" => r"e :\i",
        "ao" => "AU", "ou" => "@U",
        "an" => "a :n", "en" => "@ :n",
        "ang" => "A N", "eng" => "@ N", "ong" => "U N",
        "ia" => "ia", "ie" => "ie", "iao" => "iAU", "iu" => "i@U",
        "ian" => "iE :n", "in" => "i :n",
        "iang" => "iA N", "ing" => "i N", "iong" => "iU N",
        "ua" => "ua", "uo" => "uo", "uai" => r"ua :\i",
        "ui" => r"ue :\i", "uan" => "ua :n", "un" => "u@ :n",
        "uang" => "uA N", "ueng" => "u@ N",
        "ve" => "yE", "van" => "y{ :n", "vn" => "yE :n",
        "er" => "a r\\`",
        _ => return None,
    };
    Some(
        initial.split_whitespace()
            .chain(final_phones.split_whitespace())
            .filter(|phone| !phone.is_empty())
            .map(str::to_owned).collect(),
    )
}
