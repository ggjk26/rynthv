//! Read-only Flat model preflight with container checks and optional
//! preparation of readable inline DDS F0 and timbre graphs.
use super::{FlatVoice, MAX_STAGE_BYTES, inline, legacy, render::STAGES};
use crate::synthesis::stages::{
    sv1_duration::Sv1DurationStage, sv1_f0::Sv1F0Stage, sv1_timbre::Sv1TimbreNetwork,
    vocoder::VocoderStage,
};
use crate::synthesis::voice::{
    Result, VoiceError,
    archive::{Archive, ArchiveLoadError, ResourceId},
    format::FormatContext,
};
use serde::Serialize;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const DNNI_V3_MAGIC: [u8; 8] = [0xff, 0x00, 0xca, 0x7f, 3, 0, 0, 0];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    Missing,
    FileUnreadable,
    UnsupportedFormat,
    InvalidNativeHeader,
    HeaderSignatureRejected,
    InvalidNativeIndex,
    ResourceDigestRejected,
    InlineDdsReady,
    LegacyDecodeRejected,
    InvalidInlineGraph,
    StageResourceNotFound,
    NativeResourceFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlatStageCheck {
    pub role: String,
    pub id: Option<String>,
    pub path: Option<PathBuf>,
    pub state: StageState,
    pub container: String,
    pub details: String,
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlatPreflight {
    pub voice: String,
    pub version: String,
    pub stages: Vec<FlatStageCheck>,
    pub located: usize,
    pub native_resource_identified: usize,
    /// All four DDS programs and the manifest conditions passed preflight.
    pub synthesis_supported: bool,
    pub next_step: String,
}

impl FlatVoice {
    /// Opt-in, potentially expensive inspection of locally installed stages.
    /// Call from a CLI command or worker, never directly in an egui frame.
    pub fn preflight(&self, timing_override: Option<&Path>) -> FlatPreflight {
        let stages: Vec<_> = self.models.iter().map(|model| {
            probe_file(model.role, Some(&model.id), model.resolved_path(timing_override).map(Path::to_owned))
        }).collect();
        let located = stages.iter().filter(|stage| stage.path.is_some()).count();
        let native_resource_identified = stages.iter()
            .filter(|stage| stage.state == StageState::NativeResourceFound)
            .count();
        let synthesis_supported = self.render_profile().is_ok() && STAGES.iter().all(|(role, _, _)| {
            stages.iter().any(|stage| stage.role == *role && stage.state == StageState::InlineDdsReady)
        });
        FlatPreflight {
            voice: self.name.clone(),
            version: self.version.clone(),
            stages,
            located,
            native_resource_identified,
            synthesis_supported,
            next_step: if synthesis_supported {
                "四个 DDS 阶段已通过预检，可加载文本 NOFS-JSON 并渲染乐谱。"
            } else {
                "请检查缺失的模型、容器类型和 NOFS-JSON 条件向量。"
            }.to_owned(),
        }
    }
}

/// Probe an explicitly supplied path. The role is a hypothesis until its
/// graph or stage resource is identified by the reader.
pub fn probe_model_path(role: &str, path: PathBuf) -> Result<FlatStageCheck> {
    if known_native_stage_id(role).is_none() {
        return Err(VoiceError::new(format!("不支持的模型角色：{role}")));
    }
    Ok(probe_file(role, None, Some(path)))
}

fn probe_file(role: &str, id: Option<&str>, path: Option<PathBuf>) -> FlatStageCheck {
    let mut check = FlatStageCheck {
        role: role.to_owned(),
        id: id.map(str::to_owned),
        path: path.clone(),
        state: StageState::Missing,
        container: "not checked".to_owned(),
        details: "未找到已安装的模型文件".to_owned(),
        bytes: None,
    };
    let Some(path) = path else { return check; };

    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            check.state = StageState::FileUnreadable;
            check.details = "模型路径不是普通文件".to_owned();
            return check;
        }
        Err(error) => {
            check.state = StageState::FileUnreadable;
            check.details = format!("无法读取文件属性：{error}");
            return check;
        }
    };
    check.bytes = Some(metadata.len());
    if metadata.len() < 8 || metadata.len() > MAX_STAGE_BYTES {
        check.state = StageState::UnsupportedFormat;
        check.details = "文件过短或超过 1 GiB 安全检查限制".to_owned();
        return check;
    }
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(error) => {
            check.state = StageState::FileUnreadable;
            check.details = format!("无法打开模型文件：{error}");
            return check;
        }
    };
    let mut magic = [0; 8];
    if let Err(error) = file.read_exact(&mut magic) {
        check.state = StageState::FileUnreadable;
        check.details = format!("无法读取 DNNI 文件头：{error}");
        return check;
    }
    let version = inline::version(&magic);
    let legacy_key = (version == Some(3) && legacy::plausible_length(metadata.len()))
        .then(legacy::key);
    let mut legacy_failure = None;
    if let Some((_, stage, root)) = STAGES.into_iter().find(|(name, _, _)| *name == role) {
        if matches!(version, Some(1 | 2)) || legacy_key.as_ref().is_some_and(Result::is_ok) {
            check.container = if version == Some(3) {
                "Flat legacy DNNI v3".into()
            } else {
                format!("Inline DNNI v{}", magic[4])
            };
            let result = inline::load(&path, root, role).and_then(|tree| {
                match stage {
                    "duration" => Sv1DurationStage::prepare(&tree).map(|_| ()),
                    "f0" => Sv1F0Stage::prepare_native(&tree, 0.0).map(|_| ()),
                    "timbre" => Sv1TimbreNetwork::prepare_native(&tree).map(|_| ()),
                    _ => VocoderStage::prepare(&tree).map(|_| ()),
                }
            });
            match result {
                Ok(()) => {
                    check.state = StageState::InlineDdsReady;
                    check.details = format!("DDS {role} 阶段已装配");
                    return check;
                }
                Err(error) if version != Some(3) => {
                    check.state = StageState::InvalidInlineGraph;
                    check.details = error.to_string();
                    return check;
                }
                Err(error) => legacy_failure = Some(error),
            }
        }
    }
    if magic != DNNI_V3_MAGIC {
        check.state = StageState::UnsupportedFormat;
        check.container = format!("未知/非原生 v3 格式：{:02X?}", magic);
        check.details = "目前的 Rust DNNI v3 读取器不能识别该容器；需要针对实际 Flat 模型格式实现读取，不意味着文件损坏".to_owned();
        return check;
    }
    check.container = "DNNI v3".to_owned();
    let mut bytes = magic.to_vec();
    if let Err(error) = file
        .take(MAX_STAGE_BYTES - magic.len() as u64 + 1)
        .read_to_end(&mut bytes)
    {
        check.state = StageState::FileUnreadable;
        check.details = format!("无法读取模型文件：{error}");
        return check;
    }
    check.bytes = Some(bytes.len() as u64);
    if bytes.len() as u64 > MAX_STAGE_BYTES {
        check.state = StageState::UnsupportedFormat;
        check.details = "文件超过 1 GiB 检查限制".to_owned();
        return check;
    }
    let archive = match Archive::from_bytes_checked(bytes.into(), &FormatContext::dnni_v3()) {
        Ok(archive) => archive,
        Err(error) => {
            let (state, details) = match error {
                ArchiveLoadError::Header(error) => (StageState::InvalidNativeHeader, error),
                ArchiveLoadError::Signature(error) => (StageState::HeaderSignatureRejected, error),
                ArchiveLoadError::Index(error) => (StageState::InvalidNativeIndex, error),
            };
            if let Some(legacy_error) = legacy_failure {
                check.state = StageState::LegacyDecodeRejected;
                check.container = "DNNI v3".into();
                check.details = legacy_error.to_string();
            } else {
                check.state = state;
                check.details = details.to_string();
                if let Some(Err(key_error)) = legacy_key {
                    check.details.push_str(&format!("。{key_error}"));
                }
            }
            return check;
        }
    };
    if let Err(error) = archive.verify_all() {
        check.state = StageState::ResourceDigestRejected;
        check.details = error.to_string();
        return check;
    }
    let Some(role_id) = known_native_stage_id(role) else {
        check.state = StageState::StageResourceNotFound;
        check.details = "当前原生内核尚无此 Flat 模型角色的映射；容器已通过验证".to_owned();
        return check;
    };
    if archive.index(ResourceId(role_id.to_le_bytes())).is_some() {
        check.state = StageState::NativeResourceFound;
        check.details = "容器及资源摘要已校验，并发现同名 SV2 阶段资源；尚未验证 Flat 专属模型计算图和条件输入".to_owned();
    } else {
        check.state = StageState::StageResourceNotFound;
        check.details = "容器摘要已验证，但未发现对应的原生 SV2 阶段资源；不能直接交给现有 SV2 推理内核".to_owned();
    }
    check
}

fn known_native_stage_id(role: &str) -> Option<u128> {
    match role {
        "acoustic" => Some(0xa3d15b952e72382824a13092648ea34e),
        "duration" => Some(0xbc72d4467151f7763681c6765678bb13),
        "f0" => Some(0x34620edf25443fa373c2ee09ae6de624),
        "vocoder" => Some(0xa95bea71e057b1ae3b7924462d6ca9aa),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_model_does_not_claim_synthesis_supported() {
        let voice = FlatVoice {
            name: "test".into(), version: "200".into(), vendor: "".into(),
            language: "japanese".into(), supported_languages: vec![], phoneset: None,
            models: vec![super::super::FlatModelReference {
                role: "duration", id: "7b0ea690ada94b4484b50d9d64a21cae".into(),
                installed: false, path: None,
            }],
            styles: vec![], pitch: None, timing: None,
            f0_style: None, duration_style: None, notes: vec![],
        };
        let preflight = voice.preflight(None);
        assert!(!preflight.synthesis_supported);
        assert_eq!(preflight.located, 0);
        assert_eq!(preflight.stages[0].state, StageState::Missing);
    }
}
