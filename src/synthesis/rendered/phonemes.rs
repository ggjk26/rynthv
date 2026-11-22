use crate::{model::EntityId, synthesis::plan::OwnerInput};
use std::{collections::BTreeMap, sync::Arc};

mod prepare;

pub use crate::model::SourcePhone;

/// One source phoneme on the acoustic clock, including its unedited model values.
#[derive(Clone, Debug)]
pub struct RenderedPhoneme {
    pub source: SourcePhone,
    pub token: String,
    pub language: String,
    pub begin_seconds: f64,
    pub end_seconds: f64,
    pub default_begin_seconds: f64,
    pub default_end_seconds: f64,
    pub activity: Option<f64>,
    pub position: Option<f64>,
    pub strength: Option<f64>,
    pub default_activity: Option<f64>,
    pub default_position: Option<f64>,
    pub default_strength: Option<f64>,
    pub supports_timing: bool,
    pub supports_impulse: bool,
}

/// The complete owner result is shared before any PCM block is requested.
#[derive(Clone, Debug)]
pub struct RenderedPhonemeTrack {
    pub phones: Arc<[RenderedPhoneme]>,
    labels: Arc<BTreeMap<EntityId, String>>,
}

impl RenderedPhonemeTrack {
    pub fn label(&self, note: EntityId) -> Option<&str> {
        self.labels.get(&note).map(String::as_str)
    }

    pub fn memory_bytes(&self) -> usize {
        self.phones.iter().fold(
            self.labels.values().fold(
                std::mem::size_of_val(self.phones.as_ref()),
                |bytes, label| {
                    bytes
                        .saturating_add(std::mem::size_of::<(EntityId, String)>())
                        .saturating_add(label.capacity())
                },
            ),
            |bytes, phone| {
                bytes
                    .saturating_add(phone.token.capacity())
                    .saturating_add(phone.language.capacity())
            },
        )
    }
}

/// Source geometry remains attached to timing data while an editor revision changes.
#[derive(Clone, Debug)]
pub struct PhonemeSnapshot {
    pub input: Arc<OwnerInput>,
    pub data: RenderedPhonemeTrack,
}
