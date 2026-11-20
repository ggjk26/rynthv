//! Read-only discovery of Synthesizer V Studio Flat's lightweight NOFS-JSON.
//! These files describe embeddings and references to external base models;
//! they do not contain a native SV1 NOFS model or a complete synthesis runtime.
use super::{VoiceAvailability, VoiceCatalogEntry};
use crate::synthesis::voice::{Result, VoiceError};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

const MAX_FLAT_METADATA: u64 = 1024 * 1024;

/// JSON NOFS and binary SV1 NOFS share a file extension, not a container.
pub(super) fn is_flat(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut prefix = [0u8; 256];
    let Ok(length) = file.read(&mut prefix) else {
        return false;
    };
    let slice = &prefix[..length];
    let slice = slice.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(slice);
    slice.iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(b'{')
}

pub(super) fn entry(path: &Path) -> Result<VoiceCatalogEntry> {
    let canonical = fs::canonicalize(path)?;
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    let id = format!("svflat:{}", crate::synthesis::voice::hex(&digest[..12]));
    let icon_path = path.parent().map(|parent| parent.join("profile.png"))
        .filter(|path| path.is_file());
    let name_fallback = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let mut result = VoiceCatalogEntry {
        database_id: id,
        revision: 0,
        model_version: None,
        family: None,
        directory: path.to_owned(),
        display_name: name_fallback,
        vendor: String::new(),
        genres: Vec::new(),
        pitch_range: None,
        languages: Vec::new(),
        icon_path,
        availability: VoiceAvailability::Unsupported(
            "Flat NOFS-JSON 已识别，但原生 Flat 基础模型、时长、F0 和声学推理尚未实现".into(),
        ),
    };
    match metadata(path) {
        Ok(metadata) => {
            let value = metadata.as_object().expect("validated JSON object");
            let string = |key: &str| value.get(key).and_then(Value::as_str);
            if let Some(name) = string("name").filter(|name| !name.is_empty()) {
                result.display_name = name.to_owned();
            }
            result.vendor = string("vendor").unwrap_or_default().to_owned();
            let version = string("version")
                .map(str::to_owned)
                .or_else(|| value.get("version").and_then(Value::as_u64).map(|v| v.to_string()))
                .unwrap_or_default();
            result.revision = version.parse::<u32>().unwrap_or(0);
            if let Some(language) = string("language").filter(|s| !s.is_empty()) {
                result.languages.push(language.to_owned());
            }
            if let Some(supported) = value.get("support_languages").and_then(Value::as_array) {
                for language in supported.iter().filter_map(Value::as_str) {
                    if !result.languages.iter().any(|item| item == language) {
                        result.languages.push(language.to_owned());
                    }
                }
            }
            let required = ["base_model", "sing_model", "timing_model", "f0_model"];
            let missing = required.into_iter()
                .filter(|field| string(field).is_none_or(str::is_empty))
                .collect::<Vec<_>>();
            let summary = if missing.is_empty() {
                "已解析基础模型与辅助模型标识"
            } else {
                "存在未声明的模型依赖"
            };
            result.availability = VoiceAvailability::Unsupported(format!(
                "SV Flat 声库（版本 {}）：{}{}。目前仅支持识别与展示元数据；未接入 Flat 模型推理，不能试听或导出",
                if version.is_empty() { "未知" } else { &version },
                summary,
                if missing.is_empty() { String::new() }
                else { format!("：{}", missing.join("、")) },
            ));
        }
        Err(error) => {
            result.availability = VoiceAvailability::Unsupported(
                format!("SV Flat NOFS-JSON 无法解析：{error}")
            );
        }
    }
    Ok(result)
}

fn metadata(path: &Path) -> Result<Value> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_FLAT_METADATA {
        return Err(VoiceError::new("Flat JSON 文件大小超过 1 MiB 限制"));
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(MAX_FLAT_METADATA + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FLAT_METADATA {
        return Err(VoiceError::new("Flat JSON 文件在读取时超过 1 MiB 限制"));
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let value: Value = serde_json::from_slice(bytes).map_err(|error| {
        VoiceError::new(format!("Flat JSON 格式无效：{error}"))
    })?;
    if !value.is_object() {
        return Err(VoiceError::new("Flat NOFS-JSON 根节点必须是 JSON 对象"));
    }
    Ok(value)
}
