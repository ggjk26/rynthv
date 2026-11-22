//! Standalone experimental F0 inference for an explicitly supplied inline
//! DDS graph and already-timed phones. The Flat Duration stage owns real timings.
use super::{FlatVoice, MAX_STAGE_BYTES, Result, VoiceError, phones::FlatPhoneTable};
use crate::{
    model::{Language, NoteMode},
    synthesis::{
        score::{ExplicitScore, ScoreNote},
        stages::{
            f0::{TimedPhone, frontend::FrontendNote},
            sv1_f0::Sv1F0Stage,
        },
        voice::{PreparedTree, embedded},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    fs::File,
    io::Read,
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatTimedScore {
    pub score: ExplicitScore,
    /// Seconds for each phone, grouped by its score record.
    pub phone_seconds: Vec<Vec<f64>>,
}

#[derive(Serialize)]
pub struct FlatPitch {
    pub model_id: String,
    pub start_seconds: f64,
    pub frame_seconds: f64,
    pub pitch_midi: Vec<f32>,
}

impl FlatVoice {
    pub fn predict_inline_f0(&self, input: &FlatTimedScore, phonesets: &Path) -> Result<FlatPitch> {
        let model = self
            .models
            .iter()
            .find(|model| model.role == "f0")
            .ok_or_else(|| VoiceError::new("Flat voice does not declare an F0 model"))?;
        let path = model.path.as_deref().ok_or_else(|| {
            VoiceError::new(format!(
                "Flat F0 model {} was not found in the supplied models directory",
                model.id
            ))
        })?;
        let style = self.f0_style.as_deref().ok_or_else(|| {
            VoiceError::new("Flat F0 inference requires the voice's 32-value f0_style")
        })?;
        if style.len() != 32 {
            return Err(VoiceError::new(
                "Flat f0_style must contain 32 float32 values",
            ));
        }
        let tree = inline_f0_tree(path)?;
        // Bind f0_style to the DDS graph's native base-style slot. The existing
        // flag projection still runs. Flat DDS enhancement control is unverified.
        let stage = Sv1F0Stage::prepare_native_style(&tree, 0.0, Some(style))?;
        let score = &input.score;
        if !score.start_seconds.is_finite()
            || score.notes.is_empty()
            || score.notes.len() != input.phone_seconds.len()
        {
            return Err(VoiceError::new(
                "Flat needs one phone timing list per score record",
            ));
        }

        let mut notes = Vec::with_capacity(score.notes.len());
        let mut seeds = Vec::with_capacity(score.notes.len());
        let mut tables = HashMap::new();
        let mut checked = HashSet::new();
        let mut phones = Vec::new();
        let mut phone_seconds = Vec::new();
        for (index, (note, seconds)) in score.notes.iter().zip(&input.phone_seconds).enumerate() {
            validate_timing(note, seconds, index)?;
            for phone in &note.phones {
                let language = Language::parse(&phone.language)
                    .filter(|language| {
                        self.supported_languages
                            .iter()
                            .any(|id| Language::parse(id) == Some(*language))
                    })
                    .ok_or_else(|| {
                        VoiceError::new(format!(
                            "Flat record {index} has an unsupported phoneme language: {}",
                            phone.language
                        ))
                    })?;
                if checked.insert((language, phone.token.as_str())) {
                    let table = match tables.entry(language) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => {
                            entry.insert(FlatPhoneTable::open(phonesets, language)?)
                        }
                    };
                    let mapped = table.mapped(&phone.token).ok_or_else(|| {
                        VoiceError::new(format!(
                            "Flat record {index} contains unknown {} phone {:?}",
                            language.id(),
                            phone.token,
                        ))
                    })?;
                    stage.verify_phone_mapping(language.id(), &phone.token, mapped)?;
                }
                phones.push(TimedPhone {
                    note_index: index,
                    language: language.id().to_owned(),
                    token: phone.token.clone(),
                    // The native DDS stage derives its own frames from seconds.
                    frames: 0,
                });
            }
            phone_seconds.extend(seconds.iter().copied());
            notes.push(FrontendNote {
                phones: note
                    .phones
                    .iter()
                    .map(|phone| phone.token.clone())
                    .collect(),
                pitch: note.pitch,
                pitch_offset: 0.0,
                duration_seconds: note.duration_seconds,
                frame_seconds: stage.frame_seconds(),
                kind: note.kind,
                flag: note.flag,
                tag: note.tag.clone(),
                parts: note.parts,
                part_index: note.part_index,
                appended_durations: Vec::new(),
                performance: note.performance,
            });
            seeds.push(note.retake_seeds.unwrap_or_default().pitch);
        }
        let output = stage.predict_native(&notes, &phones, &phone_seconds, &seeds, None)?;
        Ok(FlatPitch {
            model_id: model.id.clone(),
            start_seconds: score.start_seconds,
            frame_seconds: output.frame_seconds,
            pitch_midi: output.pitch,
        })
    }
}

pub(super) fn inline_f0_tree(path: &Path) -> Result<PreparedTree> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_STAGE_BYTES {
        return Err(VoiceError::new(
            "Flat F0 model must be a regular file of at most 1 GiB",
        ));
    }
    // Check the root before preparing any tensors.
    let mut header = [0; 20];
    file.read_exact(&mut header)?;
    if embedded::root_type(&header).ok() != Some(embedded::types::F0_WRAPPER_TYPE_ID) {
        return Err(VoiceError::new(
            "Flat F0 prediction requires a readable inline DNNI v1/v2 DDS graph; Flat DNNI v3 is not supported by this adapter",
        ));
    }
    let mut data = header.to_vec();
    file.take(MAX_STAGE_BYTES + 1 - header.len() as u64)
        .read_to_end(&mut data)?;
    if data.len() as u64 > MAX_STAGE_BYTES {
        return Err(VoiceError::new("Flat F0 model exceeds the 1 GiB limit"));
    }
    PreparedTree::prepare_embedded(&data)
}

fn validate_timing(note: &ScoreNote, seconds: &[f64], index: usize) -> Result<()> {
    if !note.pitch.is_finite()
        || !(0.0..=127.0).contains(&note.pitch)
        || !note.duration_seconds.is_finite()
        || note.duration_seconds <= 0.0
        || note.parts == 0
        || note.part_index >= note.parts
        || note.kind > 1
        || note.performance.mode != NoteMode::Singing
        || note.phones.is_empty()
        || note.phones.len() != seconds.len()
        || seconds
            .iter()
            .any(|&seconds| !seconds.is_finite() || seconds < 0.0)
    {
        return Err(VoiceError::new(format!(
            "Flat F0 record {index} has invalid pitch, timing or note mode"
        )));
    }
    let duration = note.duration_seconds / f64::from(note.parts);
    if (duration - seconds.iter().sum::<f64>()).abs() > 1e-7 {
        return Err(VoiceError::new(format!(
            "Flat F0 record {index} phone seconds do not fill its {duration} s partition"
        )));
    }
    Ok(())
}
