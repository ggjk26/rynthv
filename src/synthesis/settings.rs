//! Portable resource locations and the selected voice relative to its library.

use super::voice::VoiceCatalog;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceSelection {
    pub database_id: String,
    pub revision: u32,
}

/// Distinct installation roots. SV1 and SV2 must not be inferred from the
/// same Synthesizer V installation folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationModelSlot {
    Sv2,
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceDirectory {
    Sv1,
    Sv2,
    Flat,
    FlatModels,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisSettings {
    pub databases: Option<PathBuf>,
    pub sv1_databases: Option<PathBuf>,
    pub flat_databases: Option<PathBuf>,
    pub flat_models: Option<PathBuf>,
    /// External DNNI v3 graph used only when an SV2 voice lacks Duration.
    #[serde(default)]
    pub sv2_duration_model: Option<PathBuf>,
    /// Explicit Flat Timing Model .dnni for metadata dependency checking.
    /// This is never silently injected into the SV2 Duration stage.
    #[serde(default)]
    pub flat_timing_model: Option<PathBuf>,
    pub dictionaries: Option<PathBuf>,
    pub selected_voice: Option<VoiceSelection>,
    /// Older and third-party RynthV builds stored voicebank locations under
    /// "roots" rather than the dedicated database fields. Keep the original
    /// JSON so updating another preference never discards those locations.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "read_legacy_roots")]
    pub roots: Option<serde_json::Value>,
}

/// Collect paths from both the old single-root and multi-root configuration
/// shapes. Objects can represent either a root descriptor with a path field or
/// a map of named roots (for example {"sv2": "...", "flat": "..."}).
/// Non-path metadata is ignored, never interpreted as a directory.
fn collect_legacy_roots<'a>(
    value: &'a serde_json::Value,
    paths: &mut Vec<&'a Path>,
) -> Result<(), String> {
    use serde_json::Value;
    if paths.len() > 128 {
        return Err("声库 roots 目录数量不能超过 128".into());
    }
    match value {
        Value::Null => Ok(()),
        Value::String(path) => {
            if path.trim().is_empty() {
                return Err("声库 roots 中的目录不能为空".into());
            }
            paths.push(Path::new(path));
            if paths.len() > 128 {
                return Err("声库 roots 目录数量不能超过 128".into());
            }
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                collect_legacy_roots(item, paths)?;
            }
            Ok(())
        }
        Value::Object(fields) => {
            // A list item may be {"kind":"sv2","path":"..."}; do not
            // mistake "kind", "name" or "label" for a path.
            if let Some(path) = ["path", "directory", "root", "location"]
                .into_iter()
                .find_map(|key| fields.get(key))
            {
                return collect_legacy_roots(path, paths);
            }
            let mut data_fields = 0;
            for (key, field) in fields {
                if matches!(
                    key.as_str(),
                    "kind" | "type" | "name" | "label" | "id"
                        | "enabled" | "recursive" | "version"
                        | "description" | "display_name"
                ) {
                    continue;
                }
                // Dictionary and model caches are not voicebank directories.
                if matches!(key.as_str(), "dictionaries" | "flat_models" | "models") {
                    continue;
                }
                data_fields += 1;
                collect_legacy_roots(field, paths)?;
            }
            // Metadata-only legacy records should not prevent users from
            // starting the app and selecting a new voicebank root.
            let _ = data_fields;
            Ok(())
        }
        _ => Err("声库 roots 必须是目录路径、路径数组或包含路径的对象".into()),
    }
}

fn read_legacy_roots<'de, D>(deserializer: D) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    let mut paths = Vec::new();
    collect_legacy_roots(&value, &mut paths).map_err(serde::de::Error::custom)?;
    // Empty legacy lists are valid: the user may not have configured any
    // voicebank yet. The UI must still start and allow choosing folders.
    Ok(Some(value))
}

impl SynthesisSettings {
    pub fn database_roots(&self) -> impl Iterator<Item = (&'static str, &Path)> {
        let mut roots: Vec<(&'static str, &Path)> = Vec::new();
        for (label, path) in [
            ("SV2", self.databases.as_deref()),
            ("SV1", self.sv1_databases.as_deref()),
            ("SV Flat", self.flat_databases.as_deref()),
        ] {
            if let Some(path) = path
                && !roots.iter().any(|(_, existing)| *existing == path)
            {
                roots.push((label, path));
            }
        }
        if let Some(legacy) = self.roots.as_ref() {
            let mut paths = Vec::new();
            // The root shape was checked when the settings were deserialized.
            if collect_legacy_roots(legacy, &mut paths).is_ok() {
                for path in paths {
                    if !roots.iter().any(|(_, existing)| *existing == path) {
                        roots.push(("Legacy", path));
                    }
                }
            }
        }
        roots.into_iter()
    }

    pub fn has_databases(&self) -> bool {
        self.database_roots().next().is_some()
    }

    /// Locate the three standard Windows database roots without reading or
    /// modifying any installed voicebank contents. Missing roots are left unset.
    /// The caller must explicitly choose whether to apply these values.
    pub fn installed_windows_roots() -> Result<Self, String> {
        let mut settings = Self::default();
        #[cfg(windows)]
        {
            let home = std::env::var_os("USERPROFILE")
                .filter(|home| !home.is_empty())
                .map(PathBuf::from)
                .ok_or_else(|| "USERPROFILE 未设置，无法定位 Documents。".to_owned())?;
            let documents = home.join("Documents");
            for (destination, candidate) in [
                (&mut settings.databases, documents.join("Dreamtonics/Synthesizer V Studio 2/databases")),
                (&mut settings.sv1_databases, documents.join("Dreamtonics/Synthesizer V Studio/databases")),
                (&mut settings.flat_databases, documents.join("OPSV/Dreamtonics/Synthesizer V Studio/databases")),
                (&mut settings.flat_models, documents.join("OPSV/Dreamtonics/Synthesizer V Studio/models")),
            ] {
                if candidate.is_dir() {
                    *destination = Some(candidate);
                }
            }
        }
        #[cfg(not(windows))]
        {
            return Err("只支持在 Windows 上检测 Dreamtonics 标准安装目录。".to_owned());
        }
        if !settings.has_databases() {
            return Err("未找到任何默认 Dreamtonics databases 目录，请手动指定声库根目录。".to_owned());
        }
        Ok(settings)
    }

    pub fn path() -> Result<PathBuf, String> {
        crate::platform::config_directory().map(|path| path.join("synthesis.json"))
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::path()?;
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|error| format!("无法读取合成设置 {}：{error}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!("无法读取合成设置 {}：{error}", path.display())),
        }
    }

    /// Explicit recovery for unreadable synthesis.json files. Do not silently
    /// discard a user's configured voicebank paths on application startup.
    /// Keep the original file intact under a unique backup name.
    pub fn backup_and_reset_invalid() -> Result<PathBuf, String> {
        let path = Self::path()?;
        if Self::load().is_ok() {
            return Err("当前合成设置可正常读取，无需重置。".to_owned());
        }
        let backup = path.with_file_name(format!(
            "synthesis-{}.json.bak",
            uuid::Uuid::new_v4()
        ));
        fs::rename(&path, &backup).map_err(|error| {
            format!(
                "无法备份原合成设置 {}：{error}",
                path.display()
            )
        })?;
        match Self::default().save() {
            Ok(()) => Ok(backup),
            Err(error) => match fs::rename(&backup, &path) {
                Ok(()) => Err(format!("无法重置合成设置：{error}；原配置已恢复。")),
                Err(restore_error) => Err(format!(
                    "无法重置合成设置：{error}；恢复失败：{restore_error}。原配置备份位置：{}",
                    backup.display()
                )),
            },
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path()?;
        // A malformed existing file needs explicit repair, not replacement by
        // defaults from a startup that could not deserialize it.
        Self::load()?;
        let parent = path.parent().expect("settings path has a parent");
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建合成设置目录 {}：{error}", parent.display()))?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("无法序列化合成设置：{error}"))?;
        let temporary = parent.join(format!("synthesis-{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| -> std::io::Result<()> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(|error| format!("无法保存合成设置 {}：{error}", path.display()))
    }

    pub fn selected_directory(&self) -> Result<PathBuf, String> {
        if !self.has_databases() {
            return Err("尚未设置声库目录。".to_owned());
        }
        let selected = self
            .selected_voice
            .as_ref()
            .ok_or_else(|| "尚未选择声库。".to_owned())?;
        let mut directories = Vec::new();
        let mut errors = Vec::new();
        for (label, root) in self.database_roots() {
            match VoiceCatalog::scan_with_models(root, self.flat_models.as_deref()) {
                Ok(catalog) => {
                    directories.extend(catalog.entries.into_iter().filter_map(|entry| {
                        (entry.database_id == selected.database_id
                            && entry.revision == selected.revision)
                            .then_some(entry.directory)
                    }))
                }
                Err(error) => errors.push(format!("{label} {}：{error}", root.display())),
            }
        }
        directories.sort();
        directories.dedup();
        match directories.len() {
            1 => Ok(directories.pop().expect("one selected directory")),
            0 => {
                let mut message = format!(
                    "声库 {}/{} 不在已配置的声库目录中。",
                    selected.database_id, selected.revision
                );
                if !errors.is_empty() {
                    message.push('\n');
                    message.push_str(&errors.join("\n"));
                }
                Err(message)
            }
            _ => Err(format!(
                "声库 {}/{} 对应多个文件，请检查声库目录配置。",
                selected.database_id, selected.revision
            )),
        }
    }
}

#[cfg(test)]
mod legacy_root_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_single_root_and_array_are_loadable() {
        for roots in [
            json!("C:\\Dreamtonics\\databases"),
            json!(["C:\\Dreamtonics\\databases", "D:\\OtherVoices"]),
        ] {
            let settings: SynthesisSettings =
                serde_json::from_value(json!({ "roots": roots })).unwrap();
            assert!(settings.has_databases());
            assert!(!settings.database_roots().collect::<Vec<_>>().is_empty());
            assert_eq!(
                serde_json::to_value(&settings).unwrap()["roots"],
                roots,
                "must preserve the user's original root data"
            );
        }
    }

    #[test]
    fn legacy_named_roots_and_descriptor_lists_are_loadable() {
        for roots in [
            json!({"sv2": "C:\\Dreamtonics", "sv1": "D:\\Studio1"}),
            json!([
                {"kind": "sv2", "path": "C:\\Dreamtonics"},
                {"name": "Legacy", "directory": "D:\\Studio1"}
            ]),
        ] {
            let settings: SynthesisSettings =
                serde_json::from_value(json!({ "roots": roots })).unwrap();
            assert_eq!(settings.database_roots().count(), 2);
        }
    }

    #[test]
    fn empty_roots_allow_choosing_library_later() {
        let settings: SynthesisSettings =
            serde_json::from_value(json!({ "roots": [] })).unwrap();
        assert!(!settings.has_databases());
        let settings: SynthesisSettings =
            serde_json::from_value(json!({ "roots": {} })).unwrap();
        assert!(!settings.has_databases());
    }

    #[test]
    fn explicit_roots_take_precedence_without_double_scan() {
        let settings: SynthesisSettings = serde_json::from_value(json!({
            "databases": "C:\\Dreamtonics",
            "roots": ["C:\\Dreamtonics", "D:\\Studio1"]
        }))
        .unwrap();
        let roots: Vec<_> = settings.database_roots().collect();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].0, "SV2");
        assert_eq!(roots[1].0, "Legacy");
    }

    #[test]
    fn reject_corrupt_legacy_roots_and_other_unknown_fields() {
        assert!(serde_json::from_value::<SynthesisSettings>(
            json!({"roots": [42]})
        ).is_err());
        assert!(serde_json::from_value::<SynthesisSettings>(
            json!({"unrelated_unknown_field": true})
        ).is_err());
    }
}
