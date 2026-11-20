use super::dictionary::{PhoneTable, Phones, WordReadings, read, unknown};
use crate::synthesis::voice::{Result, VoiceError};
use std::{collections::HashMap, path::Path};

pub(super) struct Japanese {
    phones: PhoneTable,
    kana: HashMap<char, String>,
    words: WordReadings,
}

impl Japanese {
    pub(super) fn load(directory: &Path) -> Result<Self> {
        let phones = romaji(directory)?;
        let mut kana = HashMap::new();
        for name in [
            "japanese-hira2romaji-dict.txt",
            "japanese-kata2romaji-dict.txt",
            "japanese-sute2romaji-dict.txt",
        ] {
            for line in read(&directory.join(name))?.lines() {
                let mut fields = line.split_whitespace();
                if let (Some(key), Some(value)) = (fields.next(), fields.next()) {
                    let mut chars = key.chars();
                    if let Some(key) = chars.next()
                        && chars.next().is_none()
                    {
                        kana.insert(key, value.to_owned());
                    }
                }
            }
        }
        let words = WordReadings::load(&directory.join("edict.txt"))?;
        Ok(Self {
            phones,
            kana,
            words,
        })
    }

    /// Offline kana and romaji table. Kanji readings require an external EDICT
    /// or explicit phonemes; no unverified G2P guesses are made.
    pub(super) fn builtin() -> Self {
        let mut phones = PhoneTable::default();
        let mut kana = HashMap::new();
        for (character, romaji, phonemes) in [
            ("あ", "a", "a"),
            ("い", "i", "i"),
            ("う", "u", "u"),
            ("え", "e", "e"),
            ("お", "o", "o"),
            ("か", "ka", "k a"),
            ("き", "ki", "k i"),
            ("く", "ku", "k u"),
            ("け", "ke", "k e"),
            ("こ", "ko", "k o"),
            ("さ", "sa", "s a"),
            ("し", "shi", "sh i"),
            ("す", "su", "s u"),
            ("せ", "se", "s e"),
            ("そ", "so", "s o"),
            ("た", "ta", "t a"),
            ("ち", "chi", "ch i"),
            ("つ", "tsu", "ts u"),
            ("て", "te", "t e"),
            ("と", "to", "t o"),
            ("な", "na", "n a"),
            ("に", "ni", "n i"),
            ("ぬ", "nu", "n u"),
            ("ね", "ne", "n e"),
            ("の", "no", "n o"),
            ("は", "ha", "h a"),
            ("ひ", "hi", "h i"),
            ("ふ", "fu", "f u"),
            ("へ", "he", "h e"),
            ("ほ", "ho", "h o"),
            ("ま", "ma", "m a"),
            ("み", "mi", "m i"),
            ("む", "mu", "m u"),
            ("め", "me", "m e"),
            ("も", "mo", "m o"),
            ("や", "ya", "y a"),
            ("ゆ", "yu", "y u"),
            ("よ", "yo", "y o"),
            ("ら", "ra", "r a"),
            ("り", "ri", "r i"),
            ("る", "ru", "r u"),
            ("れ", "re", "r e"),
            ("ろ", "ro", "r o"),
            ("わ", "wa", "w a"),
            ("を", "wo", "o"),
            ("ん", "n", "N"),
            ("が", "ga", "g a"),
            ("ぎ", "gi", "g i"),
            ("ぐ", "gu", "g u"),
            ("げ", "ge", "g e"),
            ("ご", "go", "g o"),
            ("ざ", "za", "z a"),
            ("じ", "ji", "j i"),
            ("ず", "zu", "z u"),
            ("ぜ", "ze", "z e"),
            ("ぞ", "zo", "z o"),
            ("だ", "da", "d a"),
            ("ぢ", "ji", "j i"),
            ("づ", "zu", "z u"),
            ("で", "de", "d e"),
            ("ど", "do", "d o"),
            ("ば", "ba", "b a"),
            ("び", "bi", "b i"),
            ("ぶ", "bu", "b u"),
            ("べ", "be", "b e"),
            ("ぼ", "bo", "b o"),
            ("ぱ", "pa", "p a"),
            ("ぴ", "pi", "p i"),
            ("ぷ", "pu", "p u"),
            ("ぺ", "pe", "p e"),
            ("ぽ", "po", "p o"),
            ("ぁ", "a", "a"),
            ("ぃ", "i", "i"),
            ("ぅ", "u", "u"),
            ("ぇ", "e", "e"),
            ("ぉ", "o", "o"),
            ("ゔ", "vu", "v u"),
        ] {
            phones.insert_reading(romaji, phonemes);
            if let Some(hiragana) = character.chars().next() {
                kana.insert(hiragana, romaji.to_owned());
                let code = hiragana as u32;
                if (0x3041..=0x3096).contains(&code)
                    && let Some(katakana) = char::from_u32(code + 0x60)
                {
                    kana.insert(katakana, romaji.to_owned());
                }
            }
        }
        for (romaji, phonemes) in [
            ("kya", "ky a"),
            ("kyu", "ky u"),
            ("kyo", "ky o"),
            ("gya", "gy a"),
            ("gyu", "gy u"),
            ("gyo", "gy o"),
            ("nya", "ny a"),
            ("nyu", "ny u"),
            ("nyo", "ny o"),
            ("hya", "hy a"),
            ("hyu", "hy u"),
            ("hyo", "hy o"),
            ("bya", "by a"),
            ("byu", "by u"),
            ("byo", "by o"),
            ("pya", "py a"),
            ("pyu", "py u"),
            ("pyo", "py o"),
            ("mya", "my a"),
            ("myu", "my u"),
            ("myo", "my o"),
            ("rya", "ry a"),
            ("ryu", "ry u"),
            ("ryo", "ry o"),
            ("sha", "sh a"),
            ("shu", "sh u"),
            ("sho", "sh o"),
            ("cha", "ch a"),
            ("chu", "ch u"),
            ("cho", "ch o"),
            ("ja", "j a"),
            ("ju", "j u"),
            ("jo", "j o"),
            ("sha", "sh a"),
            ("shu", "sh u"),
            ("sho", "sh o"),
            ("cha", "ch a"),
            ("chu", "ch u"),
            ("cho", "ch o"),
            ("ja", "j a"),
            ("ju", "j u"),
            ("jo", "j o"),
            ("wa", "w a"),
            ("wi", "w i"),
            ("we", "w e"),
            ("wo", "o"),
            ("she", "sh e"),
            ("che", "ch e"),
            ("je", "j e"),
            ("tsa", "ts a"),
            ("tsi", "ts i"),
            ("tse", "ts e"),
            ("tso", "ts o"),
            ("fa", "f a"),
            ("fi", "f i"),
            ("fe", "f e"),
            ("fo", "f o"),
            ("kwa", "k w a"),
            ("gwa", "g w a"),
        ] {
            phones.insert_reading(romaji, phonemes);
        }
        Self { phones, kana, words: WordReadings::default() }
    }

    pub(super) fn resolve(&self, lyrics: &str) -> Result<Vec<Phones>> {
        let mut result = Vec::new();
        for token in lyrics
            .split(|ch: char| ch.is_whitespace() || ch == '\'' || ch == '’')
            .filter(|s| !s.is_empty())
        {
            if token.is_ascii() {
                result.extend(self.phones.segment(&token.to_lowercase())?);
            } else if token
                .chars()
                .all(|ch| self.kana.contains_key(&ch) || matches!(ch, 'ー' | 'っ' | 'ッ'))
            {
                result.extend(self.resolve_kana(token)?);
            } else {
                for reading in self.words.resolve(token)? {
                    result.extend(self.resolve_kana(&reading)?);
                }
            }
        }
        if result.is_empty() {
            return Err(unknown(lyrics));
        }
        Ok(result)
    }

    fn resolve_kana(&self, text: &str) -> Result<Vec<Phones>> {
        let mut result: Vec<Phones> = Vec::new();
        let mut pending = Vec::new();
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if matches!(ch, 'っ' | 'ッ') {
                pending.push("cl".to_owned());
                continue;
            }
            if ch == 'ー' {
                let vowel = result
                    .last()
                    .and_then(|phones| phones.last())
                    .filter(|phone| matches!(phone.as_str(), "a" | "i" | "u" | "e" | "o"))
                    .ok_or_else(|| VoiceError::new("日语长音前缺少元音，请填写罗马音或显式音素"))?;
                result.push(vec![vowel.clone()]);
                continue;
            }
            let mut romanized = self
                .kana
                .get(&ch)
                .cloned()
                .ok_or_else(|| unknown(&ch.to_string()))?;
            if let Some(&next) = chars.peek()
                && let Some(small) = small_kana(next)
            {
                let base = romanized.trim_end_matches(['a', 'i', 'u', 'e', 'o']);
                romanized = if let Some(vowel) = small.strip_prefix('y') {
                    if matches!(base, "sh" | "ch" | "j") {
                        format!("{base}{vowel}")
                    } else {
                        format!("{base}{small}")
                    }
                } else if romanized == "u" {
                    format!("w{small}")
                } else if matches!(romanized.as_str(), "ku" | "gu") {
                    format!("{base}w{small}")
                } else {
                    format!("{base}{small}")
                };
                chars.next();
            }
            pending.extend(self.phones.get(&romanized)?);
            result.push(std::mem::take(&mut pending));
        }
        if !pending.is_empty() {
            if let Some(last) = result.last_mut() {
                last.extend(pending);
            } else {
                result.push(pending);
            }
        }
        Ok(result)
    }
}

fn romaji(directory: &Path) -> Result<PhoneTable> {
    let mut table = PhoneTable::load(&directory.join("japanese-romaji-dict.txt"), false)?;
    for line in read(&directory.join("japanese-romaji-phones.txt"))?.lines() {
        let mut fields = line.trim_start_matches('\u{feff}').split_whitespace();
        if let (Some(phone), Some(class)) = (fields.next(), fields.next())
            && matches!(
                class,
                "stop" | "fricative" | "affricate" | "aspirate" | "liquid" | "nasal" | "semivowel"
            )
        {
            // Syllable readings such as n -> N keep priority over literal consonants.
            table.include_literal(phone);
        }
    }
    Ok(table)
}

fn small_kana(ch: char) -> Option<&'static str> {
    match ch {
        'ゃ' | 'ャ' => Some("ya"),
        'ゅ' | 'ュ' => Some("yu"),
        'ょ' | 'ョ' => Some("yo"),
        'ぁ' | 'ァ' => Some("a"),
        'ぃ' | 'ィ' => Some("i"),
        'ぅ' | 'ゥ' => Some("u"),
        'ぇ' | 'ェ' => Some("e"),
        'ぉ' | 'ォ' => Some("o"),
        _ => None,
    }
}
