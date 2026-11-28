//! Experimental Flat acoustic frames from explicitly timed phones and the
//! same phrase's DDS pitch prediction.
use super::{
    FlatVoice, Result, VoiceError, inline,
    score::{FlatTimedScore, PreparedScore},
};
use crate::synthesis::{
    stages::{
        sv1_duration::AcousticClock,
        sv1_timbre::Sv1TimbreNetwork,
        timbre::{TimbreRequest, records::AcousticNote},
    },
    voice::{PreparedTree, embedded::types::TIMBRE_TYPE_ID},
};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FlatAcoustics {
    pub model_id: String,
    pub f0_model_id: String,
    pub style_name: String,
    pub start_seconds: f64,
    pub frame_seconds: f64,
    pub frames: usize,
    pub channels: usize,
    /// Frame-major DDS acoustic rows.
    pub values: Vec<f32>,
}

impl FlatVoice {
    pub fn predict_inline_acoustics(
        &self,
        input: &FlatTimedScore,
        phonesets: &Path,
        style_name: Option<&str>,
    ) -> Result<FlatAcoustics> {
        let (model_id, path) = inline::required_model(self, "acoustic")?;
        let conditioning = self.conditioning(style_name)?;
        let mut score = PreparedScore::new(self, input, phonesets)?;
        let tree = inline_acoustic_tree(path)?;
        let network = Sv1TimbreNetwork::prepare_native_style(&tree, &conditioning.acoustic_style)?;
        score.verify_mappings(|language, token, mapped| {
            network.verify_phone_mapping(language, token, mapped)
        })?;
        let (f0_model_id, pitch) = self.run_inline_f0(&mut score)?;

        let frame_seconds = network.frame_seconds();
        let mut phones = score.phones;
        let mut modulation: Vec<_> = input
            .score
            .notes
            .iter()
            .map(|note| AcousticNote {
                frames: 0,
                timbre_seed: note.retake_seeds.map(|seeds| seeds.timbre),
                kind: note.kind,
                factor: None,
            })
            .collect();
        let mut clock = AcousticClock::new(frame_seconds);
        for (phone, seconds) in phones.iter_mut().zip(score.phone_seconds) {
            phone.frames = clock.advance(seconds)?;
            modulation[phone.note_index].frames += phone.frames;
        }
        for note in &mut score.notes {
            note.frame_seconds = frame_seconds;
        }
        let output = network.render(
            &TimbreRequest {
                notes: &score.notes,
                phones: &phones,
                midi_pitch: &pitch.pitch,
                note_modulation: &modulation,
                phone_impulses: None,
                frame_controls: None,
                primary_style_weights: None,
                secondary_style_weights: None,
                requested_frame: 0,
            },
            &pitch,
        )?;
        let acoustics = output.acoustics;
        if acoustics.values.iter().any(|value| !value.is_finite()) {
            return Err(VoiceError::new(
                "Flat DDS timbre produced non-finite acoustic frames",
            ));
        }
        Ok(FlatAcoustics {
            model_id: model_id.to_owned(),
            f0_model_id,
            style_name: conditioning.style_name,
            start_seconds: input.score.start_seconds,
            frame_seconds,
            frames: acoustics.frames,
            channels: acoustics.channels,
            values: acoustics.values,
        })
    }
}

pub(super) fn inline_acoustic_tree(path: &Path) -> Result<PreparedTree> {
    inline::load(path, TIMBRE_TYPE_ID, "acoustic")
}
