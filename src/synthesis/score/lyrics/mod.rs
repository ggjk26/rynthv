mod dictionary;
mod japanese;
mod mandarin;

use crate::{
    model::Language,
    synthesis::{
        plan::{NoteInput, OwnerInput},
        voice::{Result, VoiceError},
    },
};
use dictionary::Dictionary;
use mandarin::MandarinLexicon;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

/// Immutable resource dictionaries are loaded on first use of a language, not
/// on each edit and not all at application startup. They never cache rendered audio.
pub struct LyricFrontend {
    directory: Option<PathBuf>,
    default_language: Language,
    dictionaries: HashMap<Language, OnceLock<Result<Lexicon>>>,
}

enum Lexicon {
    Mandarin(Arc<MandarinLexicon>),
    Japanese(Arc<japanese::Japanese>),
    Dictionary(Arc<Dictionary>),
}

static BUILTIN_MANDARIN: OnceLock<Arc<MandarinLexicon>> = OnceLock::new();
static BUILTIN_JAPANESE: OnceLock<Arc<japanese::Japanese>> = OnceLock::new();
static BUILTIN_ENGLISH: OnceLock<Arc<Dictionary>> = OnceLock::new();

impl LyricFrontend {
    pub fn new(
        directory: Option<&Path>,
        default_language: &str,
        supported: &[String],
    ) -> Result<Self> {
        let parse = |id: &str| {
            Language::parse(id).ok_or_else(|| VoiceError::new(format!("无法识别声库语言：{id}")))
        };
        let default_language = parse(default_language)?;
        let dictionaries = supported
            .iter()
            .map(|id| Ok((parse(id)?, OnceLock::new())))
            .collect::<Result<HashMap<_, _>>>()?;
        if !dictionaries.contains_key(&default_language) {
            return Err(VoiceError::new("声库未声明支持其默认语言"));
        }
        Ok(Self {
            directory: directory.map(Path::to_owned),
            default_language,
            dictionaries,
        })
    }

    pub(super) fn language(&self, input: &OwnerInput) -> Result<Language> {
        let language = super::options::language(input, self.default_language)?;
        if !self.dictionaries.contains_key(&language) {
            return Err(VoiceError::new(format!(
                "当前声库不支持{}",
                language.label()
            )));
        }
        Ok(language)
    }

    pub(super) fn note_language(&self, note: &NoteInput, inherited: Language) -> Result<Language> {
        let language = super::options::note_language(note, inherited)?;
        if !self.dictionaries.contains_key(&language) {
            return Err(VoiceError::new(format!(
                "当前声库不支持{}",
                language.label()
            )));
        }
        Ok(language)
    }

    pub(super) fn resolve(&self, language: Language, lyrics: &str) -> Result<Vec<Vec<String>>> {
        let dictionary = self
            .dictionaries
            .get(&language)
            .ok_or_else(|| VoiceError::new(format!("当前声库不支持{}", language.label())))?
            .get_or_init(|| match (self.directory.as_deref(), language) {
                (Some(directory), Language::Mandarin) =>
                    MandarinLexicon::load(directory).map(Arc::new).map(Lexicon::Mandarin),
                (Some(directory), Language::Japanese) =>
                    japanese::Japanese::load(directory).map(Arc::new).map(Lexicon::Japanese),
                (Some(directory), language) =>
                    Dictionary::load(directory, language).map(Arc::new).map(Lexicon::Dictionary),
                (None, Language::Mandarin) => Ok(Lexicon::Mandarin(Arc::clone(
                    BUILTIN_MANDARIN.get_or_init(|| Arc::new(MandarinLexicon::builtin()))
                ))),
                (None, Language::Japanese) => Ok(Lexicon::Japanese(Arc::clone(
                    BUILTIN_JAPANESE.get_or_init(|| Arc::new(japanese::Japanese::builtin()))
                ))),
                (None, Language::English) => Ok(Lexicon::Dictionary(Arc::clone(
                    BUILTIN_ENGLISH.get_or_init(|| Arc::new(Dictionary::builtin_english()))
                ))),
                (None, language) => Err(VoiceError::new(format!(
                    "{}尚无内置字词词典，请指定 clf-data 词典目录或为音符输入显式音素",
                    language.label()
                ))),
            })
            .as_ref()
            .map_err(Clone::clone)?;
        let result = match dictionary {
            Lexicon::Mandarin(dictionary) => dictionary.resolve(lyrics),
            Lexicon::Japanese(dictionary) => dictionary.resolve(lyrics),
            Lexicon::Dictionary(dictionary) => dictionary.resolve(lyrics),
        }?;
        if result.is_empty() || result.iter().any(Vec::is_empty) {
            return Err(VoiceError::new(format!(
                "{}歌词没有可合成的音素",
                language.label()
            )));
        }
        Ok(result)
    }
}
