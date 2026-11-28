//! Optional external SV2 Duration model, separate from Flat Timing.
//! Never substitute a Flat model merely because the filename contains
//! "duration": the authenticated DNNI v3 archive must contain the native
//! SV2 Duration resource identifier.
use std::{fs, path::Path, sync::Arc};

use super::{
    Result, VoiceError,
    archive::{Archive, ResourceId},
    config::CONFIGURATION_ID,
    format::FormatContext,
};

const DURATION_ID: ResourceId = ResourceId([
    0x13, 0xbb, 0x78, 0x56, 0x76, 0xc6, 0x81, 0x36,
    0x76, 0xf7, 0x51, 0x71, 0x46, 0xd4, 0x72, 0xbc,
]);
const MAX_EXTERNAL_BYTES: u64 = 1024 * 1024 * 1024;

pub(super) fn load(path: &Path) -> Result<Arc<Archive>> {
    if !path.is_file() || !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dnni")) {
        return Err(VoiceError::new(format!(
            "SV2 时长模型必须是本地 .dnni 文件：{}", path.display()
        )));
    }
    let len = fs::metadata(path)?.len();
    if len == 0 || len > MAX_EXTERNAL_BYTES {
        return Err(VoiceError::new("SV2 时长模型文件为空或超过 1 GiB 检查限制"));
    }
    let archive = Archive::load(path, &FormatContext::dnni_v3())?;
    if archive.index(DURATION_ID).is_none() {
        return Err(VoiceError::new(
            "该文件缺少 SV2 Duration 资源，不能把 Flat Timing 模型直接当作 SV2 时长模型"
        ));
    }
    if archive.index(CONFIGURATION_ID).is_some() {
        return Err(VoiceError::new("SV2 共享时长模型不应包含独立声库配置"));
    }
    archive.verify_all()?;
    Ok(Arc::new(archive))
}

/// A configured external file takes precedence over an optional embedded
/// duration, but only when a voicebank actually lacks its own duration stage.
pub(super) fn selected(path: Option<&Path>) -> Result<Option<(Arc<Archive>, String)>> {
    if let Some(path) = path {
        let archive = load(path)?;
        return Ok(Some((archive, path.display().to_string())));
    }
    super::bundled::duration().map(|archive| {
        archive.map(|archive| (archive, "bundled-duration.dnni".to_owned()))
    })
}
