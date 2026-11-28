use super::{Result, VoiceError};
use crate::{model::Language, synthesis::voice::phones::Reader};
use std::{collections::HashMap, fs::File, io::Read, path::Path};

pub(super) struct FlatPhoneTable {
    mappings: HashMap<String, String>,
}

impl FlatPhoneTable {
    pub fn open(directory: &Path, language: Language) -> Result<Self> {
        let table = format!("{}-phones", language.phone_set());
        let path = directory.join(format!("{table}.data"));
        Self::read(&path, &table)
            .map_err(|error| VoiceError::new(format!("{}: {error}", path.display())))
    }

    fn read(path: &Path, expected_name: &str) -> Result<Self> {
        // The files store a name and three length-prefixed string arrays.
        // The second array is descriptive; the graph owns numeric class IDs.
        const MAX_BYTES: u64 = 256 * 1024;
        let file = File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(VoiceError::new("Flat phoneme table must be a regular file"));
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(VoiceError::new("Flat phoneme table exceeds 256 KiB"));
        }
        let mut reader = Reader {
            bytes: &bytes,
            offset: 0,
        };
        let name = reader.string()?;
        let symbols = reader.strings()?;
        let classes = reader.strings()?;
        let mapped = reader.strings()?;
        if name != expected_name
            || reader.offset != bytes.len()
            || symbols.is_empty()
            || symbols.len() != classes.len()
            || symbols.len() != mapped.len()
            || classes.iter().any(String::is_empty)
        {
            return Err(VoiceError::new(
                "Flat phoneme table has an invalid name or array layout",
            ));
        }
        let valid = |symbol: &str| !symbol.is_empty() && !symbol.chars().any(char::is_whitespace);
        let mut mappings = HashMap::with_capacity(symbols.len());
        for (symbol, target) in symbols.into_iter().zip(mapped) {
            if !valid(&symbol) || !valid(&target) || mappings.insert(symbol, target).is_some() {
                return Err(VoiceError::new(
                    "Flat phoneme table has an invalid or duplicate symbol",
                ));
            }
        }
        Ok(Self { mappings })
    }

    pub fn mapped(&self, token: &str) -> Option<&str> {
        self.mappings.get(token).map(String::as_str)
    }
}
