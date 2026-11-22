use super::{
    VoiceAvailability, VoiceCatalog, VoiceCatalogEntry, metadata::MetadataIndex, resource,
};
use crate::synthesis::voice::{Result, VoiceFamily, format::FormatContext};
use std::{fs, path::Path};
use sha2::Digest;

pub(super) fn scan(
    database: &Path,
    database_id: uuid::Uuid,
    catalog: &mut VoiceCatalog,
    metadata: &MetadataIndex,
    duration_override: Option<&Path>,
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
                    match super::super::shared_duration::selected(duration_override) {
                        Ok(Some((_, source))) => {
                            missing.retain(|role| *role != "时长");
                            entry.model_dependencies.push(format!(
                                "SV2 Duration：{source}（DNNI 资源身份已验证）"
                            ));
                        }
                        Err(error) => entry.model_dependencies.push(format!(
                            "SV2 Duration 文件不可用：{error}"
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

/// Detect directly extracted SV2 voice folders without requiring an
/// installer-generated UUID/version directory. A folder with NOFS/Flat data
/// is not promoted to SV2 solely because it also holds unrelated DNNI files.
pub(super) fn scan_named(
    directory: &Path,
    metadata: &MetadataIndex,
    duration_override: Option<&Path>,
) -> Result<Option<VoiceCatalogEntry>> {
    if !has_dnni_models(directory)? {
        return Ok(None);
    }
    // An extracted Flat package can reference separate DNNI models; it must
    // not be misclassified as a fully-supported SV2 model directory.
    if super::super::nofs::paths(directory)?
        .iter()
        .any(|file| super::flat::is_flat(file) || super::flat::is_flat_package(file))
    {
        return Ok(None);
    }
    let sidecar = super::super::sidecar::load(directory)?;
    let canonical = fs::canonicalize(directory)?;
    let digest = sha2::Sha256::digest(canonical.to_string_lossy().as_bytes());
    let id = format!("sv2-local:{}", super::super::hex(&digest[..12]));
    let context = FormatContext::dnni_v3();
    let folder_name = directory.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let mut entry = VoiceCatalogEntry {
        database_id: id,
        revision: 0,
        version_label: None,
        model_version: None,
        family: None,
        directory: directory.to_owned(),
        display_name: folder_name,
        vendor: String::new(),
        genres: Vec::new(),
        pitch_range: None,
        languages: Vec::new(),
        styles: Vec::new(),
        model_dependencies: Vec::new(),
        icon_path: find_profile(directory),
        availability: VoiceAvailability::Loadable,
    };
    match resource::configuration(directory, &context) {
        Ok(configuration) => {
            entry.model_version = Some(configuration.model_version);
            entry.family = Some(if configuration.model_version == 1 {
                VoiceFamily::Sv2Compatible
            } else {
                VoiceFamily::Sv2
            });
            entry.revision = configuration.version.parse::<u32>().unwrap_or(0);
            entry.version_label = Some(configuration.version);
            entry.display_name = configuration.name;
            entry.vendor = configuration.vendor;
            entry.languages = configuration.languages;
            let mut missing = configuration.models.missing(configuration.model_version);
            if missing.contains(&"时长") {
                match super::super::shared_duration::selected(duration_override) {
                    Ok(Some((_, source))) => {
                        missing.retain(|role| *role != "时长");
                        entry.model_dependencies.push(format!(
                            "SV2 Duration：{source}（DNNI 资源身份已验证）"
                        ));
                    }
                    Err(error) => entry.model_dependencies.push(format!(
                        "SV2 Duration 文件不可用：{error}"
                    )),
                    Ok(None) => {}
                }
            }
            if !missing.is_empty() {
                entry.availability = VoiceAvailability::Unsupported(format!(
                    "缺少本地{}模型，当前安装内容无法在本机合成",
                    missing.join("、")
                ));
            }
            if let Some(sidecar) = sidecar.as_ref() {
                entry.genres.clone_from(&sidecar.metadata.genres);
                entry.pitch_range.clone_from(&sidecar.metadata.pitch_range);
                if !sidecar.metadata.languages.is_empty() {
                    entry.languages.clone_from(&sidecar.metadata.languages);
                }
                entry.model_dependencies.push(format!(
                    "独立声库资源：{}（内部版本号 {}）",
                    sidecar.metadata.dnni_file, sidecar.metadata.version_number
                ));
            } else if let Some(display) = metadata.lookup(&entry.display_name, &entry.vendor) {
                entry.genres.clone_from(&display.genres);
                entry.pitch_range.clone_from(&display.pitch_range);
                if !display.languages.is_empty() {
                    entry.languages.clone_from(&display.languages);
                }
                if entry.icon_path.is_none() {
                    entry.icon_path.clone_from(&display.icon_path);
                }
            }
        }
        Err(error) if error.to_string() == "声库缺少名称配置资源" => {
            // A manually extracted voicebank may keep its metadata in
            // configs.txt/meta.json rather than embed it in the DNNI index.
            // Show an honest unsupported entry for this recognizable layout,
            // not an empty list or a false claim that synthesis can run.
            let config = directory.join("configs.txt");
            let meta = directory.join("meta.json");
            if !config.is_file() || !meta.is_file() {
                // Plain shared runtime-model caches are not voicebanks.
                return Ok(None);
            }
            entry.model_dependencies.push(
                "已找到 configs.txt 和 meta.json；尚未验证外部配置结构".to_owned()
            );
            entry.availability = VoiceAvailability::Unsupported(
                "已识别独立 SV2 声库文件夹，但 DNNI 内缺少可读取的声库配置；需要解析 configs.txt/meta.json 并确定模型依赖后才能合成".to_owned()
            );
        }
        Err(error) => {
            entry.availability = VoiceAvailability::Unsupported(
                format!("DNNI 声库配置无法读取：{error}")
            );
        }
    }
    Ok(Some(entry))
}

fn find_profile(directory: &Path) -> Option<std::path::PathBuf> {
    let exact = directory.join("profile.png");
    if exact.is_file() {
        return Some(exact);
    }
    fs::read_dir(directory).ok()?.take(512).flatten()
        .find(|entry| entry.file_name().eq_ignore_ascii_case("profile.png")
            && entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
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
