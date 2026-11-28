//! Commit automatic pronunciation only after the current voice has produced
//! a matching phoneme snapshot. Do not guess pronunciation from lyrics here.
use super::App;
use crate::{
    model::{EntityId, Group, GroupRef},
    synthesis::{plan::OwnerId, rendered::PhonemeSnapshot},
};
use std::collections::BTreeMap;

impl App {
    pub(crate) fn matching_phonemes(
        &self,
        reference: &GroupRef,
        group: &Group,
    ) -> Option<&PhonemeSnapshot> {
        let owner = OwnerId(reference.id);
        if !self.editor.is_editing()
            && let Some(snapshot) = self.synthesis.phonemes_current(owner, self.editor.revision)
        {
            return Some(snapshot);
        }
        let snapshot = self.synthesis.phonemes(owner)?;
        snapshot
            .input
            .matches_project(
                &self.editor.project,
                self.editor.current_track(),
                reference,
                group,
            )
            .then_some(snapshot)
    }

    pub(super) fn freeze_rendered_phonemes(&mut self) {
        if self.editor.is_editing()
            || self.editor.transport.playing
            || self.gesture.is_some()
            || self.group_gesture.is_some()
            || self.lyric.is_some()
            || self.phoneme_text.is_some()
        {
            return;
        }
        let Some(reference) = self.editor.active_ref() else {
            return;
        };
        let Some(group) = self.editor.group() else {
            return;
        };
        let Some(snapshot) = self.matching_phonemes(reference, group) else {
            return;
        };
        let missing = group
            .notes
            .iter()
            .filter(|note| note.phonemes.trim().is_empty());
        if !missing.clone().any(|note| {
            snapshot
                .data
                .label(note.id)
                .is_some_and(|text| !text.is_empty())
        }) {
            return;
        }
        // A linked group shared by multiple instances may have different
        // singer overrides. Do not bake one instance's phonemes into the
        // shared group, as that would silently change the other singers.
        let shared = self.editor.project.tracks.iter()
            .flat_map(|track| track.references())
            .filter(|item| item.group_id == reference.group_id)
            .count() > 1;
        if shared {
            return;
        }
        let updates: BTreeMap<EntityId, String> = missing
            .filter_map(|note| {
                let text = snapshot.data.label(note.id)?;
                (!text.is_empty()).then(|| (note.id, text.to_owned()))
            })
            .collect();
        self.editor.begin("固定首次渲染音素");
        if let Some(group) = self.editor.group_mut() {
            for note in &mut group.notes {
                if let Some(text) = updates.get(&note.id) {
                    note.phonemes.clone_from(text);
                }
            }
        }
        self.editor.finish();
    }
}
