//! Commit automatic pronunciation only after the current voice has produced
//! a matching phoneme snapshot. Do not guess pronunciation from lyrics here.
use super::App;
use crate::{model::EntityId, synthesis::plan::OwnerId};
use std::collections::BTreeMap;

impl App {
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
        let Some(snapshot) = self.synthesis.phonemes(OwnerId(reference.id)) else {
            return;
        };
        if !snapshot.input.matches_project(
            &self.editor.project,
            self.editor.current_track(),
            reference,
            group,
        ) {
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
        let mut phones: BTreeMap<EntityId, Vec<(usize, String)>> = BTreeMap::new();
        for phone in snapshot.data.phones.iter() {
            phones.entry(phone.source.note_id).or_default()
                .push((phone.source.phone_index, phone.token.clone()));
        }
        let updates: BTreeMap<EntityId, String> = group.notes.iter()
            .filter(|note| note.phonemes.trim().is_empty())
            .filter_map(|note| {
                let tokens = phones.get(&note.id)?;
                if tokens.is_empty() {
                    return None;
                }
                let mut tokens = tokens.clone();
                tokens.sort_unstable_by_key(|(index, _)| *index);
                let text = tokens.into_iter()
                    .map(|(_, token)| token)
                    .collect::<Vec<_>>()
                    .join(" ");
                (!text.is_empty()).then_some((note.id, text))
            })
            .collect();
        if updates.is_empty() {
            return;
        }
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
