use super::{FlatVoice, MAX_STAGE_BYTES, Result, VoiceError, legacy};
use crate::synthesis::voice::{PreparedTree, embedded};
use std::{fs::File, io::Read, ops::Range, path::Path};

pub(in crate::synthesis::voice) struct InlineModel {
    bytes: Vec<u8>,
    graph: Range<usize>,
}

impl InlineModel {
    pub fn graph(&self) -> &[u8] {
        &self.bytes[self.graph.clone()]
    }

    pub fn file_bytes(&self) -> usize {
        self.bytes.len()
    }
}

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

pub(super) fn version(header: &[u8]) -> Option<u32> {
    if header.get(..4)? != [0xff, 0x00, 0xca, 0x7f] {
        return None;
    }
    Some(u32::from_le_bytes(header.get(4..8)?.try_into().ok()?))
}

fn open(path: &Path) -> Result<(File, [u8; 8], u64)> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let length = metadata.len();
    if !metadata.is_file() || !(8..=MAX_STAGE_BYTES).contains(&length) {
        return Err(VoiceError::new(
            "Flat 模型必须是 1 GiB 以内的常规 DNNI 文件",
        ));
    }
    let mut header = [0; 8];
    file.read_exact(&mut header)?;
    Ok((file, header, length))
}

pub(in crate::synthesis::voice) fn check_header(path: &Path) -> Result<()> {
    let (_, header, length) = open(path)?;
    match version(&header) {
        Some(1 | 2) => Ok(()),
        Some(3) if legacy::plausible_length(length) => legacy::key().map(|_| ()),
        _ => Err(VoiceError::new("不支持的 Flat DDS 模型文件头")),
    }
}

pub(in crate::synthesis::voice) fn read(path: &Path, root_type: [u8; 16]) -> Result<InlineModel> {
    let (file, header, length) = open(path)?;
    let key = match version(&header) {
        Some(1 | 2) => None,
        Some(3) if legacy::plausible_length(length) => Some(legacy::key()?),
        Some(3) => return Err(VoiceError::new("Flat DNNI v3 legacy 密文长度无效")),
        _ => return Err(VoiceError::new("不支持的 Flat DDS 模型文件头")),
    };
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length as usize + 1)
        .map_err(|_| VoiceError::new("无法为 Flat 模型分配读取缓冲区"))?;
    bytes.extend_from_slice(&header);
    file.take(length - header.len() as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != length {
        return Err(VoiceError::new("Flat 模型文件在读取期间发生变化"));
    }
    let graph = match key {
        Some(key) => legacy::unwrap(&mut bytes, &key)?,
        None => 0..bytes.len(),
    };
    let model = InlineModel { bytes, graph };
    if embedded::root_type(model.graph())? != root_type {
        return Err(VoiceError::new(
            "Flat DDS 图的根节点与清单声明的模型角色不符",
        ));
    }
    Ok(model)
}

pub(super) fn load(path: &Path, root_type: [u8; 16], stage: &str) -> Result<PreparedTree> {
    let model =
        read(path, root_type).map_err(|error| VoiceError::new(format!("Flat {stage}: {error}")))?;
    PreparedTree::prepare_embedded(model.graph())
}
