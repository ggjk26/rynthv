use super::{
    VoiceAvailability, VoiceCatalog, VoiceCatalogEntry, metadata::MetadataIndex, resource,
};
use crate::synthesis::voice::{Result, VoiceFamily, format::FormatContext};
use std::{fs, path::Path};

pub(super) fn scan(
    database: &Path,
    database_id: uuid::Uuid,
    catalog: &mut VoiceCatalog,
    metadata: &MetadataIndex,
) -> Result<()> {
    let context = FormatContext::dnni_v3();
    let revisions = match fs::read_dir(database) {
        Ok(revisions) => revisions,
        Err(error) => {
            catalog
                .warnings
                .push(format!("无法读取声库 {database_id}：{error}"));
            return Ok(());
        }
    };
    for revision in revisions {
        let revision = revision?;
        if !revision.file_type()?.is_dir() {
            continue;
        }
        let Ok(version) = revision.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        // A numeric revision folder can belong to an SV Flat package.
        // Only create an SV2 catalog entry if an actual DNNI model is present;
        // the Flat info.nofs is discovered separately.
        match has_dnni_models(&revision.path()) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(error) => {
                catalog.warnings.push(format!(
                    "无法检查声库版本目录 {}：{error}",
                    revision.path().display()
                ));
                continue;
            }
        }
        let mut entry = VoiceCatalogEntry {
            database_id: database_id.to_string(),
            revision: version,
            version_label: None,
            model_version: None,
            family: None,
            directory: revision.path(),
            display_name: database_id.to_string(),
            vendor: String::new(),
            genres: Vec::new(),
            pitch_range: None,
            languages: Vec::new(),
            styles: Vec::new(),
            model_dependencies: Vec::new(),
            icon_path: None,
            availability: VoiceAvailability::Loadable,
        };
        match resource::configuration(&entry.directory, &context) {
            Ok(configuration) => {
                entry.model_version = Some(configuration.model_version);
                let family = if configuration.model_version == 1 {
                    VoiceFamily::Sv2Compatible
                } else {
                    VoiceFamily::Sv2
                };
                entry.family = Some(family);
                let mut missing = configuration.models.missing(configuration.model_version);
                if missing.contains(&"时长") {
                    match super::super::bundled::duration() {
                        Ok(Some(_)) => {
                            missing.retain(|role| *role != "时长");
                            entry.model_dependencies.push(
                                "Duration：本地编译嵌入（容器及资源标识已校验）".to_owned(),
                            );
                        }
                        Err(error) => entry.model_dependencies.push(format!(
                            "内嵌 Duration 不兼容：{error}"
                        )),
                        Ok(None) => {}
                    }
                }
                if configuration.version != version.to_string() {
                    entry.availability =
                        VoiceAvailability::Unsupported("声库配置版本与目录版本不一致".to_owned());
                } else if !missing.is_empty() {
                    entry.availability = VoiceAvailability::Unsupported(format!(
                        "缺少本地{}模型，当前安装内容无法在本机合成",
                        missing.join("、")
                    ));
                }
                entry.display_name = configuration.name;
                entry.vendor = configuration.vendor;
                entry.languages = configuration.languages;
                if let Some(display) = metadata.lookup(&entry.display_name, &entry.vendor) {
                    entry.genres.clone_from(&display.genres);
                    entry.pitch_range.clone_from(&display.pitch_range);
                    // Product metadata names the voice's native languages, not every
                    // language the inference model can synthesize.
                    if !display.languages.is_empty() {
                        entry.languages.clone_from(&display.languages);
                    }
                    entry.icon_path.clone_from(&display.icon_path);
                }
            }
            Err(error) => {
                entry.availability = VoiceAvailability::Unsupported(error.to_string());
            }
        }
        catalog.entries.push(entry);
    }

    Ok(())
}

fn has_dnni_models(directory: &Path) -> std::io::Result<bool> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        if item.file_type()?.is_file()
            && item.path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dnni"))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
