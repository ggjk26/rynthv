//! Lightweight discovery of installed voice revisions, without loading model weights.

mod dnni;
pub(super) mod flat;
mod metadata;
mod nofs;
mod resource;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::{Result, VoiceFamily};
use metadata::MetadataIndex;

#[derive(Clone, Debug, Serialize)]
pub enum VoiceAvailability {
    /// The revision uses a supported format; full model validation happens on load.
    Loadable,
    Unsupported(String),
}

impl VoiceAvailability {
    pub fn is_loadable(&self) -> bool {
        matches!(self, Self::Loadable)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct VoiceCatalogEntry {
    pub database_id: String,
    pub revision: u32,
    /// Full release label (e.g. Flat 106b1) when revision is only numeric.
    pub version_label: Option<String>,
    pub model_version: Option<u8>,
    pub family: Option<VoiceFamily>,
    pub directory: PathBuf,
    pub display_name: String,
    pub vendor: String,
    pub genres: Vec<String>,
    pub pitch_range: Option<String>,
    pub languages: Vec<String>,
    pub styles: Vec<String>,
    pub model_dependencies: Vec<String>,
    pub icon_path: Option<PathBuf>,
    pub availability: VoiceAvailability,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct VoiceCatalog {
    pub entries: Vec<VoiceCatalogEntry>,
    pub warnings: Vec<String>,
}

impl VoiceCatalog {
    /// Discover supported DNNI revisions and original SV1 AI NOFS files without
    /// reading weights. A root can also be one SV1 voice folder or NOFS file.
    pub fn scan(root: &Path) -> Result<Self> {
        Self::scan_with_models(root, None)
    }

    /// Optional separate Flat model directory is for diagnostics, not synthesis.
    pub fn scan_with_models(root: &Path, flat_models: Option<&Path>) -> Result<Self> {
        let mut catalog = Self::default();
        if super::nofs::is_file(root) {
            catalog.entries.push(entry_nofs(root, flat_models)?);
            return Ok(catalog);
        }
        for path in super::nofs::paths(root)? {
            catalog.entries.push(entry_nofs(&path, flat_models)?);
        }
        let metadata = MetadataIndex::load(root);
        // Also accept a standalone, human-named folder containing DNNI
        // models (common in manually extracted voicebank collections).
        // Official UUID/revision installations remain handled below.
        if let Some(entry) = dnni::scan_named(root, &metadata)? {
            catalog.entries.push(entry);
        }
        for database in fs::read_dir(root)? {
            let database = match database {
                Ok(database) => database,
                Err(error) => {
                    catalog.warnings.push(format!(
                        "无法枚举声库目录 {}：{error}", root.display()
                    ));
                    continue;
                }
            };
            match database.file_type() {
                Ok(kind) if kind.is_dir() => {}
                Ok(_) => continue,
                Err(error) => {
                    catalog.warnings.push(format!(
                        "无法访问声库目录 {}：{error}", database.path().display()
                    ));
                    continue;
                }
            }
            if let Ok(database_id) = uuid::Uuid::parse_str(&database.file_name().to_string_lossy()) {
                if let Err(error) = dnni::scan(&database.path(), database_id, &mut catalog, &metadata) {
                    catalog.warnings.push(format!(
                        "声库 {} 的模型版本扫描失败：{error}", database.path().display()
                    ));
                }
            } else {
                match dnni::scan_named(&database.path(), &metadata) {
                    Ok(Some(entry)) => catalog.entries.push(entry),
                    Ok(None) => {}
                    Err(error) => catalog.warnings.push(format!(
                        "无法检查独立声库目录 {}：{error}", database.path().display()
                    )),
                }
            }
            match super::nofs::paths(&database.path()) {
                Ok(paths) => {
                    for path in paths {
                        catalog.entries.push(entry_nofs(&path, flat_models)?);
                    }
                }
                Err(error) => catalog.warnings.push(format!(
                    "无法读取声库目录 {}：{error}",
                    database.path().display()
                )),
            }
            // Flat managers may install a voice in a revision subdirectory,
            // while SV2 uses the same nesting for DNNI files.
            // A broken/unreadable Flat subdirectory must not hide every
            // other installed voicebank from the library catalog.
            match fs::read_dir(database.path()) {
                Ok(revisions) => {
                    for revision in revisions {
                        let revision = match revision {
                            Ok(revision) => revision,
                            Err(error) => {
                                catalog.warnings.push(format!(
                                    "无法枚举声库版本目录 {}：{error}",
                                    database.path().display()
                                ));
                                continue;
                            }
                        };
                        match revision.file_type() {
                            Ok(kind) if kind.is_dir() => {}
                            Ok(_) => continue,
                            Err(error) => {
                                catalog.warnings.push(format!(
                                    "无法识别声库版本目录 {}：{error}",
                                    revision.path().display()
                                ));
                                continue;
                            }
                        }
                        match super::nofs::paths(&revision.path()) {
                            Ok(paths) => {
                                for path in paths {
                                    catalog.entries.push(entry_nofs(&path, flat_models)?);
                                }
                            }
                            Err(error) => catalog.warnings.push(format!(
                                "无法读取声库版本目录 {}：{error}",
                                revision.path().display()
                            )),
                        }
                    }
                }
                Err(error) => catalog.warnings.push(format!(
                    "无法枚举声库子目录 {}：{error}",
                    database.path().display()
                )),
            }
        }
        let mut counts = std::collections::BTreeMap::new();
        for entry in &catalog.entries {
            *counts
                .entry((entry.database_id.clone(), entry.revision))
                .or_insert(0usize) += 1;
        }
        for entry in &mut catalog.entries {
            if counts[&(entry.database_id.clone(), entry.revision)] > 1 {
                entry.availability = VoiceAvailability::Unsupported(
                    "相同声库版本存在多个文件，请移除重复项后刷新".to_owned(),
                );
            }
        }
        catalog.entries.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then_with(|| left.database_id.cmp(&right.database_id))
                .then_with(|| right.revision.cmp(&left.revision))
        });
        Ok(catalog)
    }
}


fn entry_nofs(path: &Path, models: Option<&Path>) -> Result<VoiceCatalogEntry> {
    if flat::is_flat(path) || flat::is_flat_package(path) {
        flat::entry(path, models)
    } else {
        nofs::entry(path)
    }
}
