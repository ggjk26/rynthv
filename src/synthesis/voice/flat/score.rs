use super::{FlatVoice, Result, VoiceError, phones::FlatPhoneTable};
use crate::{
    model::{Language, NoteMode},
    synthesis::{
        score::{ExplicitScore, ScoreNote},
        stages::f0::{TimedPhone, frontend::FrontendNote},
    },
};
use serde::Deserialize;
use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatTimedScore {
    pub score: ExplicitScore,
    /// Seconds for each phone, grouped by its score record.
    pub phone_seconds: Vec<Vec<f64>>,
}

pub(super) struct PreparedScore {
    pub notes: Vec<FrontendNote>,
    pub phones: Vec<TimedPhone>,
    pub phone_seconds: Vec<f64>,
    pub pitch_seeds: Vec<u32>,
    mappings: HashMap<(Language, String), String>,
}

impl PreparedScore {
    pub fn new(voice: &FlatVoice, input: &FlatTimedScore, phonesets: &Path) -> Result<Self> {
        let score = &input.score;
        if !score.start_seconds.is_finite()
            || score.notes.is_empty()
            || score.notes.len() != input.phone_seconds.len()
        {
            return Err(VoiceError::new(
                "Flat needs a finite start time and one phone timing list per score record",
            ));
        }

        let mut notes = Vec::with_capacity(score.notes.len());
        let mut pitch_seeds = Vec::with_capacity(score.notes.len());
        let mut tables = HashMap::new();
        let mut mappings = HashMap::new();
        let mut phones = Vec::new();
        let mut phone_seconds = Vec::new();
        for (index, (note, seconds)) in score.notes.iter().zip(&input.phone_seconds).enumerate() {
            validate_timing(note, seconds, index)?;
            for phone in &note.phones {
                let language = Language::parse(&phone.language)
                    .filter(|language| {
                        voice
                            .supported_languages
                            .iter()
                            .any(|id| Language::parse(id) == Some(*language))
                    })
                    .ok_or_else(|| {
                        VoiceError::new(format!(
                            "Flat record {index} has an unsupported phoneme language: {}",
                            phone.language
                        ))
                    })?;
                if let Entry::Vacant(mapping) = mappings.entry((language, phone.token.clone())) {
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
                    mapping.insert(mapped.to_owned());
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
                frame_seconds: crate::synthesis::score::FRAME_SECONDS,
                kind: note.kind,
                flag: note.flag,
                tag: note.tag.clone(),
                parts: note.parts,
                part_index: note.part_index,
                appended_durations: Vec::new(),
                performance: note.performance,
            });
            pitch_seeds.push(note.retake_seeds.unwrap_or_default().pitch);
        }
        Ok(Self {
            notes,
            phones,
            phone_seconds,
            pitch_seeds,
            mappings,
        })
    }

    pub fn verify_mappings(
        &self,
        mut verify: impl FnMut(&str, &str, &str) -> Result<()>,
    ) -> Result<()> {
        for ((language, token), mapped) in &self.mappings {
            verify(language.id(), token, mapped)?;
        }
        Ok(())
    }
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
            "Flat record {index} has invalid pitch, timing or note mode"
        )));
    }
    let duration = note.duration_seconds / f64::from(note.parts);
    if (duration - seconds.iter().sum::<f64>()).abs() > 1e-7 {
        return Err(VoiceError::new(format!(
            "Flat record {index} phone seconds do not fill its {duration} s partition"
        )));
    }
    Ok(())
}
