use super::{FlatVoice, MAX_STAGE_BYTES, Result, VoiceError};
use crate::synthesis::voice::{PreparedTree, embedded};
use std::{fs::File, io::Read, path::Path};

pub(super) fn required_model<'a>(voice: &'a FlatVoice, role: &str) -> Result<(&'a str, &'a Path)> {
    let model = voice
        .models
        .iter()
        .find(|model| model.role == role)
        .ok_or_else(|| VoiceError::new(format!("Flat voice does not declare a {role} model")))?;
    let path = model.path.as_deref().ok_or_else(|| {
        VoiceError::new(format!(
            "Flat {role} model {} was not found in the supplied models directory",
            model.id
        ))
    })?;
    Ok((&model.id, path))
}

pub(super) fn load(path: &Path, root_type: [u8; 16], stage: &str) -> Result<PreparedTree> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_STAGE_BYTES {
        return Err(VoiceError::new(format!(
            "Flat {stage} model must be a regular file of at most 1 GiB"
        )));
    }
    let mut header = [0; 20];
    file.read_exact(&mut header)?;
    if embedded::root_type(&header).ok() != Some(root_type) {
        return Err(VoiceError::new(format!(
            "Flat {stage} prediction requires a readable inline DNNI v1/v2 DDS graph; Flat DNNI v3 is not supported by this adapter"
        )));
    }
    let mut data = header.to_vec();
    file.take(MAX_STAGE_BYTES + 1 - header.len() as u64)
        .read_to_end(&mut data)?;
    if data.len() as u64 > MAX_STAGE_BYTES {
        return Err(VoiceError::new(format!(
            "Flat {stage} model exceeds the 1 GiB limit"
        )));
    }
    PreparedTree::prepare_embedded(&data)
}
