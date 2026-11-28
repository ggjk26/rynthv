//! Conditions and resource roles for Flat's native DDS pipeline.
use super::{FlatVoice, Result, VoiceError, inline};
use crate::synthesis::voice::{embedded::types, hex};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

#[derive(Serialize)]
pub(crate) struct FlatRenderProfile {
    pub(crate) acoustic: Vec<f32>,
    pub(crate) variants: Vec<(String, Vec<f32>)>,
    pub(crate) f0: Vec<f32>,
    pub(crate) duration: Vec<f32>,
}

pub(in crate::synthesis::voice) struct StageFile<'a> {
    pub role: &'static str,
    pub root: [u8; 16],
    pub path: &'a Path,
}

pub(in crate::synthesis::voice) const STAGES: [(&str, &str, [u8; 16]); 4] = [
    ("duration", "duration", types::DURATION_TYPE_ID),
    ("f0", "f0", types::F0_WRAPPER_TYPE_ID),
    ("acoustic", "timbre", types::TIMBRE_TYPE_ID),
    ("vocoder", "vocoder", types::VOCODER_TYPE_ID),
];

pub(in crate::synthesis::voice) fn database_id(path: &Path) -> Result<String> {
    let canonical = fs::canonicalize(path)?;
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    Ok(format!("svflat:{}", hex(&digest[..12])))
}

impl FlatVoice {
    pub(in crate::synthesis::voice) fn render_profile(&self) -> Result<FlatRenderProfile> {
        let controls = self.conditioning(None)?;
        let required = |value: Option<Vec<f32>>, field| {
            value.ok_or_else(|| VoiceError::new(format!("Flat 声库缺少 {field} 参数")))
        };
        let mut names = HashSet::with_capacity(self.styles.len());
        for style in &self.styles {
            if !names.insert(&style.name) {
                return Err(VoiceError::new(format!(
                    "Flat 唱法名称重复：{}",
                    style.name
                )));
            }
        }
        let acoustic = controls.acoustic_style;
        let variants = self
            .styles
            .iter()
            .filter(|style| style.name != controls.style_name)
            .map(|style| {
                // DDS uses deltas from the neutral row. A named mode at
                // full strength reaches the corresponding Flat vector.
                let offset = style
                    .vector
                    .iter()
                    .zip(&acoustic)
                    .map(|(value, base)| value - base)
                    .collect();
                (style.name.clone(), offset)
            })
            .collect();
        Ok(FlatRenderProfile {
            acoustic,
            variants,
            f0: required(controls.f0_style, "f0_style")?,
            duration: required(controls.duration_style, "duration_style")?,
        })
    }

    pub(in crate::synthesis::voice) fn render_models<'a>(
        &'a self,
        timing: Option<&'a Path>,
    ) -> Result<Vec<StageFile<'a>>> {
        STAGES
            .into_iter()
            .map(|(role, stage, root)| {
                let model = self
                    .models
                    .iter()
                    .find(|model| model.role == role)
                    .ok_or_else(|| VoiceError::new(format!("Flat 清单未声明 {role} 模型")))?;
                let path = model.resolved_path(timing).ok_or_else(|| {
                    VoiceError::new(format!(
                        "未找到 Flat {role} 模型 {}，请检查 Flat 模型目录",
                        model.id
                    ))
                })?;
                Ok(StageFile {
                    role: stage,
                    root,
                    path,
                })
            })
            .collect()
    }

    pub(in crate::synthesis::voice) fn check_render_files(
        &self,
        timing: Option<&Path>,
    ) -> Result<()> {
        self.render_profile()?;
        for stage in self.render_models(timing)? {
            inline::check_header(stage.path)
                .map_err(|error| VoiceError::new(format!("Flat {}：{error}", stage.role)))?;
        }
        Ok(())
    }
}
