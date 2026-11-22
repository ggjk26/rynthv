//! Read-only Flat model preflight with container checks and optional
//! preparation of readable inline DDS F0 and timbre graphs.
use super::{FlatVoice, MAX_STAGE_BYTES, acoustic::inline_acoustic_tree, pitch::inline_f0_tree};
use crate::synthesis::stages::{sv1_f0::Sv1F0Stage, sv1_timbre::Sv1TimbreNetwork};
use crate::synthesis::voice::{
    archive::{Archive, ResourceId},
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
    InvalidNativeContainer,
    InlineF0GraphReady,
    InlineAcousticGraphReady,
    InvalidInlineGraph,
    StageResourceNotFound,
    NativeResourceFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlatStageCheck {
    pub role: String,
    pub id: String,
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
    /// Experimental graphs do not establish end-to-end Flat synthesis.
    pub synthesis_supported: bool,
    pub next_step: String,
}

impl FlatVoice {
    /// Opt-in, potentially expensive inspection of locally installed stages.
    /// Call from a CLI command or worker, never directly in an egui frame.
    pub fn preflight(&self, timing_override: Option<&Path>) -> FlatPreflight {
        let stages: Vec<_> = self.models.iter().map(|model| {
            let mut path = model.path.clone();
            if model.role == "duration" {
                if let Some(override_path) = timing_override {
                    let matching = override_path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .is_some_and(|stem| stem.eq_ignore_ascii_case(&model.id));
                    if matching
                        && override_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dnni"))
                        && override_path.is_file()
                    {
                        path = Some(override_path.to_path_buf());
                    }
                }
            }
            probe_file(model.role, &model.id, path)
        }).collect();
        let located = stages.iter().filter(|stage| stage.path.is_some()).count();
        let native_resource_identified = stages.iter()
            .filter(|stage| stage.state == StageState::NativeResourceFound)
            .count();
        FlatPreflight {
            voice: self.name.clone(),
            version: self.version.clone(),
            stages,
            located,
            native_resource_identified,
            synthesis_supported: false,
            next_step: "Flat 的 F0 和声学实验可使用可读的内联图。整曲输出仍需接通 Flat 时长模型与声码器，并支持所用 DNNI 容器。".to_owned(),
        }
    }
}

fn probe_file(role: &str, id: &str, path: Option<PathBuf>) -> FlatStageCheck {
    let mut check = FlatStageCheck {
        role: role.to_owned(),
        id: id.to_owned(),
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
    let mut magic = [0; 8];
    if let Err(error) = File::open(&path)
        .and_then(|mut file| file.read_exact(&mut magic))
    {
        check.state = StageState::FileUnreadable;
        check.details = format!("无法读取 DNNI 文件头：{error}");
        return check;
    }
    if matches!(role, "f0" | "acoustic")
        && magic[..4] == DNNI_V3_MAGIC[..4]
        && matches!(u32::from_le_bytes(magic[4..].try_into().unwrap()), 1 | 2)
    {
        check.container = format!("Inline DNNI v{}", magic[4]);
        let prepared = if role == "f0" {
            inline_f0_tree(&path)
                .and_then(|tree| Sv1F0Stage::prepare_native(&tree, 0.0).map(|_| ()))
                .map(|()| (StageState::InlineF0GraphReady, "--pitch-score"))
        } else {
            inline_acoustic_tree(&path)
                .and_then(|tree| Sv1TimbreNetwork::prepare_native(&tree).map(|_| ()))
                .map(|()| (StageState::InlineAcousticGraphReady, "--acoustic-score"))
        };
        match prepared {
            Ok((state, argument)) => {
                check.state = state;
                check.details =
                    format!("DDS {role} 计算图已装配，使用 {argument} 可验证显式时长乐谱");
            }
            Err(error) => {
                check.state = StageState::InvalidInlineGraph;
                check.details = format!("内联 DDS {role} 计算图装配失败：{error}");
            }
        }
        return check;
    }
    if magic != DNNI_V3_MAGIC {
        check.state = StageState::UnsupportedFormat;
        check.container = format!("未知/非原生 v3 格式：{:02X?}", magic);
        check.details = "目前的 Rust DNNI v3 读取器不能识别该容器；需要针对实际 Flat 模型格式实现读取，不意味着文件损坏".to_owned();
        return check;
    }
    check.container = "DNNI v3".to_owned();
    let archive = match Archive::load(&path, &FormatContext::dnni_v3())
        .and_then(|archive| {
            archive.verify_all()?;
            Ok(archive)
        })
    {
        Ok(archive) => archive,
        Err(error) => {
            check.state = StageState::InvalidNativeContainer;
            check.details = format!(
                "文件头符合 DNNI v3，但无法通过当前原生解析器的完整性校验：{error}；可能采用不同的 Flat 容器参数"
            );
            return check;
        }
    };
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
