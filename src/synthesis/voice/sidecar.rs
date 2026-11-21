//! Externally extracted SV2 voicebank metadata (configs.txt + meta.json).
//! The sidecar describes a voice, not proof that its model graphs are usable.
use std::{fs, path::{Path, PathBuf}};
use serde::Deserialize;
use super::{Result, VoiceError, config::VoiceConfiguration};

const MAX_CONFIG: u64 = 64 * 1024;
const MAX_META: u64 = 64 * 1024;

#[derive(Deserialize)]
pub(super) struct Metadata {
    pub id: String,
    pub name: String,
    pub vendor: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub version_name: String,
    pub version_number: u32,
    pub dnni_file: String,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(rename = "pitchRange")]
    pub pitch_range: Option<String>,
    #[serde(default)]
    pub languages: Vec<String>,
}

pub(super) struct Sidecar {
    pub config: VoiceConfiguration,
    pub metadata: Metadata,
    pub model_path: PathBuf,
}

fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let size = fs::metadata(path)?.len();
    if size > limit {
        return Err(VoiceError::new(format!(
            "外部声库配置 {} 超过大小限制", path.display()
        )));
    }
    let bytes = fs::read(path)?;
    if bytes.len() as u64 > limit {
        return Err(VoiceError::new("外部声库配置超过大小限制"));
    }
    Ok(bytes)
}

/// Returns None only if neither sidecar file is present. A partial or
/// inconsistent sidecar must remain visible as an explicit error.
pub(super) fn load(directory: &Path) -> Result<Option<Sidecar>> {
    let config_path = directory.join("configs.txt");
    let metadata_path = directory.join("meta.json");
    if !config_path.exists() && !metadata_path.exists() {
        return Ok(None);
    }
    if !config_path.is_file() || !metadata_path.is_file() {
        return Err(VoiceError::new(
            "独立 SV2 声库必须同时提供 configs.txt 与 meta.json"
        ));
    }
    let configuration_bytes = bounded_read(&config_path, MAX_CONFIG)?;
    let config = VoiceConfiguration::parse(&configuration_bytes)?;
    let metadata: Metadata = serde_json::from_slice(&bounded_read(&metadata_path, MAX_META)?)
        .map_err(|e| VoiceError::new(format!("meta.json 格式无效：{e}")))?;
    if metadata.kind != "Voice Databases 2"
        || uuid::Uuid::parse_str(&metadata.id).is_err()
        || metadata.name != config.name
        || metadata.vendor != config.vendor
        || metadata.version_name != config.version
    {
        return Err(VoiceError::new(
            "meta.json 与 configs.txt 的 SV2 声库标识、名称、厂商或版本不一致"
        ));
    }
    // No arbitrary paths, absolute paths, or directory traversal from JSON.
    let file = Path::new(&metadata.dnni_file);
    if file.components().count() != 1
        || file.file_name().and_then(|s| s.to_str()) != Some(metadata.dnni_file.as_str())
        || !metadata.dnni_file.to_ascii_lowercase().ends_with(".dnni")
        || metadata.dnni_file.len() > 180
    {
        return Err(VoiceError::new("meta.json 的 dnni_file 不是安全的本地 DNNI 文件名"));
    }
    let expected_prefix = format!("{}_", metadata.id);
    if !metadata.dnni_file.starts_with(&expected_prefix) {
        return Err(VoiceError::new("meta.json 的 dnni_file 与声库 ID 不一致"));
    }
    let model_path = directory.join(file);
    if !model_path.is_file() {
        return Err(VoiceError::new(format!(
            "meta.json 指定的 DNNI 模型不存在：{}", metadata.dnni_file
        )));
    }
    Ok(Some(Sidecar { config, metadata, model_path }))
}
