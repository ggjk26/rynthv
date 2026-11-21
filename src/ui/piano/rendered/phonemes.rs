use super::super::{canvas::note_rect, scene::SceneGroup};
use crate::{
    app::App,
    model::EntityId,
    synthesis::{plan::OwnerId, rendered::RenderedPhoneme},
    ui::{
        child,
        group_geometry::{clip_rect, visible_note_rect},
        theme::{CANVAS, MUTED, TEXT},
        track_color::TrackColors,
    },
};
use eframe::egui::*;
use std::collections::BTreeMap;

pub fn draw(app: &App, ui: &Ui, canvas: Rect, groups: &[SceneGroup]) {
    for scene in groups {
        let reference = &scene.reference;
        let Some(snapshot) = app.synthesis.phonemes(OwnerId(reference.id)) else {
            continue;
        };
        let Some(group) = app.editor.project.group(&reference.group_id) else {
            continue;
        };
        if !snapshot.input.matches_project(
            &app.editor.project,
            app.editor.current_track(),
            reference,
            group,
        ) {
            continue;
        }
        let mut phones: BTreeMap<EntityId, Vec<&RenderedPhoneme>> = BTreeMap::new();
        for phone in snapshot.data.phones.iter() {
            phones.entry(phone.source.note_id).or_default().push(phone);
        }
        let painter = ui.painter().with_clip_rect(
            clip_rect(reference, canvas, &app.view.piano).intersect(ui.clip_rect()),
        );
        let color = if app.editor.reference == Some(reference.id) {
            TEXT
        } else {
            MUTED
        };
        for note in &scene.notes {
            if visible_note_rect(
                note,
                reference,
                canvas,
                &app.view.piano,
                app.view.top_pitch,
                app.view.row_height,
            )
            .is_none()
            {
                continue;
            }
            let Some(phones) = phones.get_mut(&note.id) else {
                continue;
            };
            phones.sort_by_key(|phone| phone.source.phone_index);
            let label = phones
                .iter()
                .map(|phone| phone.token.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let rect = note_rect(
                note,
                reference.blick_offset,
                reference.pitch_offset,
                canvas,
                app,
            );
            let bounds = Rect::from_min_max(
                pos2(rect.left(), canvas.top()),
                pos2(rect.right(), rect.top()),
            );
            painter
                .with_clip_rect(bounds.intersect(painter.clip_rect()))
                .text(
                    pos2(rect.left() + 6.0, rect.top() - 3.0),
                    Align2::LEFT_BOTTOM,
                    label,
                    FontId::proportional(12.0),
                    color,
                );
        }
    }
}


/// A double click on a rendered pronunciation label opens its source note,
/// never the empty-canvas note creation handler.
pub(super) fn begin_text_edit(
    app: &mut App,
    canvas: Rect,
    groups: &[SceneGroup],
    pointer: Pos2,
) -> bool {
    let mut candidate = None;
    for scene in groups {
        let reference = &scene.reference;
        let Some(snapshot) = app.synthesis.phonemes(OwnerId(reference.id)) else {
            continue;
        };
        let Some(group) = app.editor.project.group(&reference.group_id) else {
            continue;
        };
        if !snapshot.input.matches_project(
            &app.editor.project,
            app.editor.current_track(),
            reference,
            group,
        ) {
            continue;
        }
        for note in &scene.notes {
            let Some(rect) = visible_note_rect(
                note,
                reference,
                canvas,
                &app.view.piano,
                app.view.top_pitch,
                app.view.row_height,
            ) else {
                continue;
            };
            let label = Rect::from_min_max(
                pos2(rect.left(), (rect.top() - 20.0).max(canvas.top())),
                pos2((rect.left() + 140.0).max(rect.right()).min(canvas.right()), rect.top()),
            );
            if label.contains(pointer)
                && snapshot.data.phones.iter().any(|phone| phone.source.note_id == note.id)
            {
                let mut phones = snapshot.data.phones.iter()
                    .filter(|phone| phone.source.note_id == note.id)
                    .collect::<Vec<_>>();
                phones.sort_by_key(|phone| phone.source.phone_index);
                let text = if note.phonemes.trim().is_empty() {
                    phones.iter().map(|phone| phone.token.as_str())
                        .collect::<Vec<_>>().join(" ")
                } else {
                    note.phonemes.clone()
                };
                candidate = Some((reference.id, note.id, text));
                break;
            }
        }
        if candidate.is_some() {
            break;
        }
    }
    let Some((reference, id, text)) = candidate else {
        return false;
    };
    app.commit_lyric();
    app.commit_phoneme_text();
    app.editor.select_reference(app.editor.track, reference);
    app.phoneme_text = Some(super::super::PhonemeTextEdit {
        id, reference, text, focus: true,
    });
    true
}

/// Inline text editor matches the corresponding rendered note label, uses the
/// active track theme accent as an outline, and widens with the input text.
pub(super) fn text_editor(app: &mut App, ui: &mut Ui, canvas: Rect) {
    let Some(mut edit) = app.phoneme_text.take() else {
        return;
    };
    let Some(reference) = app.editor.active_ref().cloned() else {
        return;
    };
    if reference.id != edit.reference {
        return;
    }
    let Some(note) = app.editor.notes().iter().find(|note| note.id == edit.id).cloned() else {
        return;
    };
    let note_rect = note_rect(
        &note, reference.blick_offset, reference.pitch_offset, canvas, app
    );
    let clip = clip_rect(&reference, canvas, &app.view.piano).intersect(canvas);
    let left = note_rect.left().max(clip.left());
    let bottom = note_rect.top().min(clip.bottom());
    let width = ((edit.text.chars().count() as f32 * 7.5) + 30.0)
        .max(80.0).min((clip.right() - left).max(0.0));
    if width < 30.0 || bottom - clip.top() < 22.0 {
        app.phoneme_text = Some(edit);
        return;
    }
    let bounds = Rect::from_min_size(pos2(left, bottom - 22.0), vec2(width, 22.0));
    let accent = TrackColors::new(app.editor.current_track()).accent;
    let mut popup = child(ui, bounds, ("phoneme-text", edit.id));
    let response = popup.add_sized(
        bounds.size(),
        TextEdit::singleline(&mut edit.text)
            .id(Id::new(("phoneme-text-input", edit.id)))
            .margin(vec2(5.0, 2.0))
            .vertical_align(Align::Center)
            .background_color(CANVAS),
    );
    ui.painter().rect_stroke(
        bounds, 2, Stroke::new(1.5, accent), StrokeKind::Inside
    );
    if edit.focus {
        response.request_focus();
        edit.focus = false;
        ui.ctx().request_repaint();
    }
    let escape = ui.input(|input| input.key_pressed(Key::Escape));
    let confirm = ui.input(|input| input.key_pressed(Key::Enter));
    if escape {
        response.surrender_focus();
    } else if confirm || response.lost_focus() {
        app.editor.set_note_phonemes(edit.id, edit.text);
        response.surrender_focus();
    } else {
        app.phoneme_text = Some(edit);
    }
}
