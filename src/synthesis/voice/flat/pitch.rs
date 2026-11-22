//! Experimental Flat F0 inference with a readable inline DDS graph.
use super::{
    FlatVoice, Result, VoiceError, inline,
    score::{FlatTimedScore, PreparedScore},
};
use crate::synthesis::{
    stages::{f0::PitchResult, sv1_f0::Sv1F0Stage},
    voice::{PreparedTree, embedded::types::F0_WRAPPER_TYPE_ID},
};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FlatPitch {
    pub model_id: String,
    pub start_seconds: f64,
    pub frame_seconds: f64,
    pub pitch_midi: Vec<f32>,
}

impl FlatVoice {
    pub fn predict_inline_f0(&self, input: &FlatTimedScore, phonesets: &Path) -> Result<FlatPitch> {
        let mut score = PreparedScore::new(self, input, phonesets)?;
        let (model_id, pitch) = self.run_inline_f0(&mut score)?;
        Ok(FlatPitch {
            model_id,
            start_seconds: input.score.start_seconds,
            frame_seconds: pitch.frame_seconds,
            pitch_midi: pitch.pitch,
        })
    }

    pub(super) fn run_inline_f0(&self, score: &mut PreparedScore) -> Result<(String, PitchResult)> {
        let (model_id, path) = inline::required_model(self, "f0")?;
        let style = self
            .f0_style
            .as_deref()
            .filter(|style| style.len() == 32)
            .ok_or_else(|| {
                VoiceError::new("Flat F0 inference requires the voice's 32-value f0_style")
            })?;
        let tree = inline_f0_tree(path)?;
        // Bind the voice's F0 style to the DDS bank's base style slot. The
        // graph's flag projection remains active.
        let stage = Sv1F0Stage::prepare_native_style(&tree, 0.0, Some(style))?;
        score.verify_mappings(|language, token, mapped| {
            stage.verify_phone_mapping(language, token, mapped)
        })?;
        for note in &mut score.notes {
            note.frame_seconds = stage.frame_seconds();
        }
        let pitch = stage.predict_native(
            &score.notes,
            &score.phones,
            &score.phone_seconds,
            &score.pitch_seeds,
            None,
        )?;
        Ok((model_id.to_owned(), pitch))
    }
}

pub(super) fn inline_f0_tree(path: &Path) -> Result<PreparedTree> {
    inline::load(path, F0_WRAPPER_TYPE_ID, "F0")
}
