//! Safe Flat JSON NOFS and SFPK sidecar discovery.
//! This is deliberately metadata-only until the native model stages support
//! Base Model conditioning and all Flat embedding spaces.
use super::{VoiceAvailability, VoiceCatalogEntry};
use crate::synthesis::voice::{flat::{FlatManifest, FlatVoice, locate_model, valid_model_id}, Result, VoiceError};
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

/// Flat info.nofs is a Flat container even if its package manifest is absent.
pub(in crate::synthesis::voice) fn is_flat_package(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name.eq_ignore_ascii_case("info.nofs"))
}

pub(super) fn entry(path: &Path, model_root: Option<&Path>) -> Result<VoiceCatalogEntry> {
    let canonical = fs::canonicalize(path)?;
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    let id = format!("svflat:{}", crate::synthesis::voice::hex(&digest[..12]));
    let icon_path = path.parent().and_then(|parent| {
        let exact = parent.join("profile.png");
        if exact.is_file() {
            Some(exact)
        } else {
            fs::read_dir(parent).ok()?.take(512).flatten()
                .find(|item| item.file_name().eq_ignore_ascii_case("profile.png")
                    && item.path().is_file())
                .map(|item| item.path())
        }
    });
    let mut entry = VoiceCatalogEntry {
        database_id: id,
        revision: 0,
        version_label: None,
        model_version: None,
        family: None,
        directory: path.to_owned(),
        display_name: if is_flat_package(path) {
            path.parent().and_then(Path::file_name).unwrap_or_default()
                .to_string_lossy().into_owned()
        } else {
            path.file_stem().unwrap_or_default().to_string_lossy().into_owned()
        },
        vendor: String::new(),
        genres: Vec::new(),
        pitch_range: None,
        languages: Vec::new(),
        styles: Vec::new(),
        model_dependencies: Vec::new(),
        icon_path,
        availability: VoiceAvailability::Unsupported(
            "SV Flat 声库尚不支持合成".to_owned()
        ),
    };
    if is_flat(path) {
        match FlatVoice::open(path, model_root) {
            Ok(voice) => {
                entry.revision = revision(&voice.version);
                entry.version_label = Some(voice.version.clone());
                entry.display_name = voice.name;
                entry.vendor = voice.vendor;
                entry.languages = voice.supported_languages;
                entry.styles = voice.styles.iter().map(|style| style.name.clone()).collect();
                entry.model_dependencies = voice.models.iter()
                    .map(|model| {
                        let status = if model.installed {
                            "已找到文件"
                        } else if model_root.is_some() {
                            "未找到文件"
                        } else {
                            "未检查：未配置模型目录"
                        };
                        format!("{}：{}（{status}）", model.role, model.id)
                    })
                    .collect();
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
        let parent = path.parent().ok_or_else(|| {
            VoiceError::new("Flat info.nofs 没有可用的上级目录")
        })?;
        let manifest_path = parent.join("config.json");
        let manifest_path = if manifest_path.is_file() {
            manifest_path
        } else {
            // Some cross-platform archives retain Config.JSON casing. Only
            // look in the same install folder; never follow manifest paths.
            fs::read_dir(parent).ok().and_then(|items| {
                items.take(512).flatten()
                    .find(|item| item.file_name().eq_ignore_ascii_case("config.json")
                        && item.path().is_file())
                    .map(|item| item.path())
            }).unwrap_or(manifest_path)
        };
        match FlatManifest::open(&manifest_path) {
            Ok(manifest) => {
                entry.revision = revision(&manifest.version);
                entry.version_label = Some(manifest.version.clone());
                entry.display_name = manifest.name;
                entry.languages = manifest.phonesets;
                let mut installed = 0usize;
                entry.model_dependencies = manifest.model_ids.iter().enumerate()
                    .map(|(i, id)| {
                        let role = manifest.model_types.get(i).map(String::as_str).unwrap_or("unknown");
                        let found = model_root.is_some_and(|root| manifest_model_exists(root, role, id));
                        if found { installed += 1; }
                        let status = if found {
                            "已找到文件"
                        } else if model_root.is_some() {
                            "未找到文件"
                        } else {
                            "未检查：未配置模型目录"
                        };
                        format!("{role}：{id}（{status}）")
                    })
                    .collect();
                let check = if model_root.is_some() {
                    format!("依赖模型 {installed}/{} 已找到", manifest.model_ids.len())
                } else {
                    "未配置 Flat 模型目录".to_owned()
                };
                entry.availability = VoiceAvailability::Unsupported(format!(
                    "Flat SFPK 安装清单 v{}：{check}。{}。info.nofs 二进制格式未解码；暂不能试听/导出",
                    manifest.version, manifest.notes.join("；")
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
    version.chars().take_while(|ch| ch.is_ascii_digit())
        .collect::<String>().parse().unwrap_or(0)
}

/// Read-only, bounded lookup: never use arbitrary manifest strings as paths.
fn manifest_model_exists(root: &Path, role: &str, id: &str) -> bool {
    if !valid_model_id(id)
        || !role.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return false;
    }
    let preferred = match role.to_ascii_lowercase().as_str() {
        "acoustic" | "base_model" | "timbre" => "acoustic",
        "pitch" | "sing_model" => "pitch",
        "f0" | "f0_model" => "f0",
        "duration" | "timing_model" => "duration",
        "vocoder" => "vocoder",
        _ => "models",
    };
    locate_model(root, id, preferred).is_some()
}

#[cfg(test)]
mod recognition_tests {
    use super::manifest_model_exists;
    use std::fs;

    #[test]
    fn finds_flat_dependencies_without_path_traversal() {
        let root = std::env::temp_dir().join(format!("rynthv-flat-{}", std::process::id()));
        fs::create_dir_all(root.join("pitch")).unwrap();
        fs::write(root.join("pitch").join("test-model.dnni"), b"fixture").unwrap();
        assert!(manifest_model_exists(&root, "sing_model", "test-model"));
        assert!(!manifest_model_exists(&root, "sing_model", "../test-model"));
        assert!(!manifest_model_exists(&root, "sing_model", "/tmp/test-model"));
        let _ = fs::remove_dir_all(root);
    }
}
