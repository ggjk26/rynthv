//! Safe Flat JSON NOFS and SFPK sidecar discovery.
//! This is deliberately metadata-only until the native model stages support
//! Base Model conditioning and all Flat embedding spaces.
use super::{VoiceAvailability, VoiceCatalogEntry};
use crate::synthesis::voice::{flat::{FlatManifest, FlatVoice}, Result, VoiceError};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

pub(in crate::synthesis::voice) fn is_flat(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut prefix = [0u8; 256];
    let Ok(length) = file.read(&mut prefix) else {
        return false;
    };
    let slice = &prefix[..length];
    let slice = slice.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(slice);
    slice.iter().copied()
        .find(|byte| !byte.is_ascii_whitespace()) == Some(b'{')
}

/// A binary info.nofs accompanied by a Flat config.json is not SV1 native NOFS.
pub(in crate::synthesis::voice) fn is_flat_package(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name.eq_ignore_ascii_case("info.nofs"))
        && path.parent().is_some_and(|dir| dir.join("config.json").is_file())
}

pub(super) fn entry(path: &Path, model_root: Option<&Path>) -> Result<VoiceCatalogEntry> {
    let canonical = fs::canonicalize(path)?;
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    let id = format!("svflat:{}", crate::synthesis::voice::hex(&digest[..12]));
    let icon_path = path.parent().map(|parent| parent.join("profile.png"))
        .filter(|path| path.is_file());
    let mut entry = VoiceCatalogEntry {
        database_id: id,
        revision: 0,
        model_version: None,
        family: None,
        directory: path.to_owned(),
        display_name: path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
        vendor: String::new(),
        genres: Vec::new(),
        pitch_range: None,
        languages: Vec::new(),
        icon_path,
        availability: VoiceAvailability::Unsupported(
            "SV Flat 声库尚不支持合成".to_owned()
        ),
    };
    if is_flat(path) {
        match FlatVoice::open(path, model_root) {
            Ok(voice) => {
                entry.revision = revision(&voice.version);
                entry.display_name = voice.name;
                entry.vendor = voice.vendor;
                entry.languages = voice.supported_languages;
                entry.genres.push(format!("{} 个唱法", voice.styles.len()));
                let installed = voice.models.iter().filter(|item| item.installed).count();
                let checks = if let Some(root) = model_root {
                    format!("模型检查 {installed}/{}（{}）", voice.models.len(), root.display())
                } else {
                    "模型目录未配置".to_owned()
                };
                let notes = if voice.notes.is_empty() { String::new() }
                    else { format!("；{}", voice.notes.join("；")) };
                entry.availability = VoiceAvailability::Unsupported(format!(
                    "Flat NOFS-JSON v{}：{}{}。声码器及原生 Flat 音色条件推理尚未实现，暂不能试听/导出",
                    voice.version, checks, notes
                ));
            }
            Err(error) => entry.availability = VoiceAvailability::Unsupported(
                format!("Flat NOFS-JSON 解析失败：{error}")
            ),
        }
    } else if is_flat_package(path) {
        let manifest_path = path.parent().ok_or_else(|| {
            VoiceError::new("Flat info.nofs 没有可用的上级目录")
        })?.join("config.json");
        match FlatManifest::open(&manifest_path) {
            Ok(manifest) => {
                entry.revision = revision(&manifest.version);
                entry.display_name = manifest.name;
                entry.languages = manifest.phonesets;
                entry.availability = VoiceAvailability::Unsupported(format!(
                    "Flat SFPK 安装清单 v{}：{} 个模型记录。{}。info.nofs 二进制格式未解码；暂不能试听/导出",
                    manifest.version, manifest.model_ids.len(), manifest.notes.join("；")
                ));
            }
            Err(error) => entry.availability = VoiceAvailability::Unsupported(
                format!("Flat config.json 解析失败：{error}")
            ),
        }
    }
    Ok(entry)
}

fn revision(version: &str) -> u32 {
    // Handles e.g. 106b1 without falsely claiming that its full release
    // identifier is represented by the integer field.
    version.chars().take_while(char::is_ascii_digit)
        .collect::<String>().parse().unwrap_or(0)
}
