//! Six-child duration program for SV1 NOFS models.
//!
//! Its conditioning is a stored model vector. It has neither a session identity
//! projection nor the impulse heads of the newer duration programs.

mod configuration;
mod encoding;
mod frontend;
mod postprocess;
pub(crate) use postprocess::AcousticClock;

use crate::synthesis::voice::{PreparedTree, Result, VoiceError};

use super::duration::{DurationPrediction, DurationRecord, DurationResult};
use configuration::Configuration;

pub struct Sv1DurationStage<'a> {
    tree: &'a PreparedTree,
    children: [usize; 6],
    configuration: Configuration,
    style: &'a [f32],
}

pub struct Sv1DurationResult {
    pub result: DurationResult,
    /// Derived record targets after stop/semivowel time borrowing. The F0
    /// note clock consumes these values rather than the original score times.
    pub record_seconds: Vec<f64>,
}

impl<'a> Sv1DurationStage<'a> {
    pub fn prepare(tree: &'a PreparedTree) -> Result<Self> {
        Self::prepare_style(tree, None)
    }

    pub(crate) fn prepare_style(tree: &'a PreparedTree, style: Option<&'a [f32]>) -> Result<Self> {
        let (configuration, children) = configuration::validate(tree)?;
        let constant = tree
            .tensor(children[5])?
            .vector()
            .ok_or_else(|| VoiceError::new("SV1 duration constant is not a float vector"))?;
        let style = style.unwrap_or(constant);
        if style.len() != constant.len() || style.iter().any(|value| !value.is_finite()) {
            return Err(VoiceError::new("Flat duration_style 不匹配 DDS 时长模型的输入维度"));
        }
        Ok(Self {
            tree,
            children,
            configuration,
            style,
        })
    }

    pub fn predict_explicit(&self, records: &[DurationRecord]) -> Result<Sv1DurationResult> {
        let phones = frontend::prepare(self.tree, self.children[0], records)?;
        let scalars = if phones.is_empty() {
            Vec::new()
        } else {
            let features =
                encoding::encode(self.tree, &self.children, self.configuration, &phones, self.style)?;
            let scalars = self.tree.execute_node(self.children[1], &features)?;
            if scalars.len() != phones.len() || scalars.iter().any(|value| !value.is_finite()) {
                return Err(VoiceError::new(
                    "SV1 duration must produce one finite scalar per phone",
                ));
            }
            scalars
        };
        let (durations, record_seconds) =
            postprocess::execute(self.tree, self.children[0], records, &scalars)?;
        let mut result = DurationResult::new(
            records,
            DurationPrediction {
                phones,
                scalars,
                impulse_values: Vec::new(),
                impulse_positions: Vec::new(),
                default_impulse_values: Vec::new(),
                default_impulse_positions: Vec::new(),
            },
            durations.clone(),
            durations,
        )?;
        result.supports_timing = false;
        result.supports_strength = false;
        Ok(Sv1DurationResult {
            result,
            record_seconds,
        })
    }
}
