//! Optional user-supplied duration model, embedded only in local builds.
//! Never ship third-party model weights in source control or default releases.
use std::sync::{Arc, OnceLock};

use super::{
    Result, VoiceError,
    archive::{Archive, ResourceId},
    format::FormatContext,
};

const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rynthv-duration.dnni"));
const DURATION_ID: ResourceId = ResourceId([
    0x13, 0xbb, 0x78, 0x56, 0x76, 0xc6, 0x81, 0x36,
    0x76, 0xf7, 0x51, 0x71, 0x46, 0xd4, 0x72, 0xbc,
]);

static VALIDATED: OnceLock<Result<Option<Arc<Archive>>>> = OnceLock::new();

/// Validate format, integrity and stage identity before making a bundled
/// duration model available. Flat timing_model is not assumed equivalent.
pub(super) fn duration() -> Result<Option<Arc<Archive>>> {
    VALIDATED
        .get_or_init(|| {
            if BYTES.is_empty() {
                return Ok(None);
            }
            let archive = Archive::from_bytes(
                Arc::<[u8]>::from(BYTES),
                &FormatContext::dnni_v3(),
            )?;
            if archive.index(DURATION_ID).is_none() {
                return Err(VoiceError::new(
                    "内嵌模型缺少 SV2 Duration 资源；Flat timing_model 不能直接当作 SV2 Duration 使用",
                ));
            }
            archive.verify_all()?;
            Ok(Some(Arc::new(archive)))
        })
        .clone()
}
