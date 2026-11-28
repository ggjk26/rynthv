//! Model-family contracts shared by full rendering and progressive preparation.
use crate::synthesis::{
    rendered::PitchTrack,
    score::PreparedPhrase,
    stages::{f0::PitchResult, sv1_f0::Sv1F0Stage, sv1_timbre::Sv1TimbreNetwork},
    voice::{PreparedVoice, Result, VoiceError, VoiceFamily},
};

pub(super) fn required_identity(identity: Option<u64>) -> Result<u64> {
    identity.ok_or_else(|| VoiceError::new("This voice requires its model identity condition"))
}

pub(super) fn dds_pitch(voice: &PreparedVoice) -> Result<Sv1F0Stage<'_>> {
    match voice.family {
        VoiceFamily::Sv1 => voice.sv1_pitch(),
        VoiceFamily::Sv2Compatible => Sv1F0Stage::prepare(voice.model("f0")?),
        VoiceFamily::Sv2 => Err(VoiceError::new(
            "SV2 voice does not use the DDS pitch program",
        )),
    }
}

pub(super) fn dds_timbre(voice: &PreparedVoice) -> Result<Sv1TimbreNetwork<'_>> {
    match voice.family {
        VoiceFamily::Sv1 => voice.sv1_timbre(),
        VoiceFamily::Sv2Compatible => Sv1TimbreNetwork::prepare(voice.model("timbre")?),
        VoiceFamily::Sv2 => Err(VoiceError::new(
            "SV2 voice does not use the DDS acoustic program",
        )),
    }
}

pub(super) fn pitch_track(
    voice: &PreparedVoice,
    phrase: &PreparedPhrase,
    prediction: PitchResult,
) -> Result<PitchTrack> {
    match voice.family {
        VoiceFamily::Sv1 | VoiceFamily::Sv2Compatible => {
            let frames = dds_pitch(voice)?.note_frames(&phrase.notes)?;
            PitchTrack::from_phrase_with_frames(phrase, prediction, &frames)
        }
        VoiceFamily::Sv2 => PitchTrack::from_phrase(phrase, prediction),
    }
}
