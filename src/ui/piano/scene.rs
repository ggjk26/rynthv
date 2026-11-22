use crate::editor::GroupEdit;
use crate::{
    app::App,
    model::*,
    ui::{
        group_edit,
        group_geometry::*,
        theme::*,
        track_color::{TrackColors, foreground},
    },
};
use eframe::egui::*;
use std::collections::HashSet;

pub fn references(app: &App) -> Vec<GroupRef> {
    let mut references: Vec<_> = app
        .editor
        .current_track()
        .visible_references()
        .cloned()
        .collect();
    references.sort_by_key(|reference| app.editor.reference == Some(reference.id));
    references
}

pub struct NoteHit {
    pub reference: EntityId,
    pub note: EntityId,
    pub rect: Rect,
    pub onset: i64,
    pub end: i64,
}

pub fn hit_note(app: &App, canvas: Rect, pos: Pos2) -> Option<NoteHit> {
    let hit_in_group = |reference: &GroupRef| {
        if !clip_rect(reference, canvas, &app.view.piano).contains(pos) {
            return None;
        }
        let group = app.editor.project.group(&reference.group_id)?;
        group.notes.iter().rev().find_map(|note| {
            let visible = visible_note_rect(
                note,
                reference,
                canvas,
                &app.view.piano,
                app.view.top_pitch,
                app.view.row_height,
            )?;
            visible.contains(pos).then(|| NoteHit {
                reference: reference.id,
                note: note.id,
                rect: super::canvas::note_rect(
                    note,
                    reference.blick_offset,
                    reference.pitch_offset,
                    canvas,
                    app,
                ),
                onset: note.onset,
                end: note.onset.saturating_add(note.duration),
            })
        })
    };
    let track = app.editor.current_track();
    app.editor.active_ref().and_then(hit_in_group).or_else(|| {
        track
            .visible_references()
            .rev()
            .filter(|reference| app.editor.reference != Some(reference.id))
            .find_map(hit_in_group)
    })
}

fn overlapping_notes(notes: &[Note]) -> HashSet<EntityId> {
    let mut intervals: Vec<_> = notes.iter().filter(|note| note.duration > 0).collect();
    intervals.sort_unstable_by_key(|note| note.onset);
    let mut previous_end = i64::MIN;
    let mut overlaps = HashSet::new();
    for (index, note) in intervals.iter().enumerate() {
        let end = note.onset.saturating_add(note.duration);
        // An earlier note overlaps if its maximum end reaches this onset.
        // A later note overlaps if the very next onset precedes this end.
        if end > note.onset
            && (previous_end > note.onset
                || intervals
                    .get(index + 1)
                    .is_some_and(|next| next.onset < end))
        {
            overlaps.insert(note.id);
        }
        previous_end = previous_end.max(end);
    }
    overlaps
}

pub fn draw_notes(app: &App, ui: &Ui, canvas: Rect, references: &[GroupRef]) {
    let colors = TrackColors::new(app.editor.current_track());
    let drawing_pitch = app.piano_tool.is_pitch();
    for reference in references {
        let clip = clip_rect(reference, canvas, &app.view.piano).intersect(ui.clip_rect());
        if !clip.is_positive() {
            continue;
        }
        let Some(group) = app.editor.project.group(&reference.group_id) else {
            continue;
        };
        let active = app.editor.reference == Some(reference.id);
        let painter = ui.painter().with_clip_rect(clip);
        let overlaps = std::cell::OnceCell::new();
        for note in &group.notes {
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
            let r = super::canvas::note_rect(
                note,
                reference.blick_offset,
                reference.pitch_offset,
                canvas,
                app,
            );
            let selected = active && !drawing_pitch && app.editor.selected.contains(&note.id);
            let color = if note.muted() {
                MUTED.gamma_multiply(0.4)
            } else if selected {
                colors.accent
            } else if active {
                colors.note
            } else {
                colors.inactive
            };
            let color = if drawing_pitch {
                color.gamma_multiply(0.45)
            } else {
                color
            };
            let corner_radius = if note.musical_type == NoteMode::Rap.id() {
                r.height() / 2.0
            } else {
                2.0
            };
            painter.rect_filled(r, corner_radius, color);
            if !drawing_pitch {
                painter.rect_stroke(
                    r,
                    corner_radius,
                    Stroke::new(
                        if selected { 1.5 } else { 0.7 },
                        if selected {
                            Color32::WHITE
                        } else {
                            colors
                                .accent
                                .gamma_multiply(if active { 0.7 } else { 0.35 })
                        },
                    ),
                    StrokeKind::Inside,
                );
                if selected {
                    for x in [r.left() + 1.0, r.right() - 1.0] {
                        painter.vline(x, r.y_range(), Stroke::new(1.0, Color32::WHITE));
                    }
                }
                if note.muted() {
                    painter.hline(
                        r.left()..=r.right(),
                        r.bottom() - 1.0,
                        Stroke::new(2.0, Color32::from_rgb(241, 90, 92)),
                    );
                }
                if overlaps
                    .get_or_init(|| overlapping_notes(&group.notes))
                    .contains(&note.id)
                {
                    let size = r.width().min(r.height()).min(10.0);
                    painter.add(Shape::convex_polygon(
                        vec![
                            r.right_top(),
                            pos2(r.right() - size, r.top()),
                            pos2(r.right(), r.top() + size),
                        ],
                        Color32::from_rgb(243, 170, 67),
                        Stroke::NONE,
                    ));
                }
            }
            if r.width() <= 10.0 || note.lyrics.is_empty() {
                continue;
            }
            // Short notes hide text instead of letting lyrics spill into
            // adjacent note rectangles. Long notes center their labels.
            let ink = if drawing_pitch || (note.muted() && active) {
                TEXT
            } else if active {
                foreground(color)
            } else {
                MUTED
            };
            let label =
                painter.layout_no_wrap(note.lyrics.clone(), FontId::proportional(12.0), ink);
            if label.size().x + 10.0 <= r.width() && label.size().y <= r.height() {
                painter.galley(
                    pos2(
                        r.center().x - label.size().x * 0.5,
                        r.center().y - label.size().y * 0.5,
                    ),
                    label,
                    ink,
                );
            }
        }
    }
}
pub fn headers(app: &mut App, ui: &mut Ui, canvas: Rect, references: &[GroupRef]) {
    let track = app.editor.track;
    let colors = TrackColors::new(app.editor.current_track());
    let drawing_pitch = app.piano_tool.is_pitch();
    for reference in references {
        let r = group_rect(reference, canvas, &app.view.piano);
        let color = if app.editor.reference == Some(reference.id) {
            colors.accent
        } else {
            colors.note
        };
        let header_clip = Rect::from_min_max(pos2(canvas.left(), canvas.top() - 20.0), canvas.max)
            .intersect(ui.clip_rect());
        let painter = ui.painter().with_clip_rect(header_clip);
        if !r.intersects(canvas) {
            continue;
        }
        painter.rect_stroke(r, 2, Stroke::new(1.0, color), StrokeKind::Inside);
        let name = app
            .editor
            .project
            .group(&reference.group_id)
            .map(|g| g.name.as_str())
            .unwrap_or("");
        let name = if name == "main" {
            app.preferences
                .language
                .text("主音符组", "Main group", "メイングループ")
        } else {
            name
        };
        let ink = foreground(color);
        let text = painter.layout_no_wrap(name.into(), FontId::proportional(12.0), ink);
        if let Some(tag) = label_rect(r, canvas, text.size().x + 12.0) {
            painter.rect_filled(
                tag,
                CornerRadius {
                    nw: 3,
                    ne: 3,
                    sw: 0,
                    se: 0,
                },
                color,
            );
            painter.with_clip_rect(tag.intersect(header_clip)).galley(
                pos2(tag.left() + 6.0, tag.center().y - text.size().y / 2.0),
                text,
                ink,
            );
            let response = ui.interact(
                tag,
                Id::new(("group-label", reference.id)),
                if drawing_pitch {
                    Sense::click()
                } else {
                    Sense::click_and_drag()
                },
            );
            if response.clicked() || response.secondary_clicked() {
                app.commit_lyric();
                app.editor.select_reference(track, reference.id);
            }
            if !drawing_pitch
                && response.drag_started_by(PointerButton::Primary)
                && reference.blick_absolute_end >= 0
                && let Some(origin) = ui.input(|i| i.pointer.press_origin())
            {
                group_edit::start(
                    app,
                    reference,
                    track,
                    origin,
                    GroupEdit::Move,
                    app.view.piano.beat_width,
                    false,
                );
            }
            response.context_menu(|ui| group_edit::context_menu(app, ui, reference));
        }
        if !drawing_pitch && reference.blick_absolute_end >= 0 {
            for (side, x, kind) in [
                ("left", r.left(), GroupEdit::Left),
                ("right", r.right(), GroupEdit::Right),
            ] {
                if x < canvas.left() || x > canvas.right() {
                    continue;
                }
                let edge =
                    Rect::from_min_max(pos2(x - 3.0, canvas.top()), pos2(x + 3.0, canvas.bottom()))
                        .intersect(canvas);
                let response = ui
                    .interact(
                        edge,
                        Id::new(("piano-group-edge", reference.id, side)),
                        Sense::drag(),
                    )
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);
                if response.drag_started_by(PointerButton::Primary)
                    && let Some(origin) = ui.input(|i| i.pointer.press_origin())
                {
                    group_edit::start(
                        app,
                        reference,
                        track,
                        origin,
                        kind,
                        app.view.piano.beat_width,
                        false,
                    );
                }
            }
        }
    }
    if app.group_gesture.as_ref().is_some_and(|g| !g.arrangement) {
        group_edit::update(app, ui.ctx(), None);
    }
}
