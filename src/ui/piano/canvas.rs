use super::{
    Gesture,
    gestures::{pitch_at, start_gesture, update_gesture},
    lyrics::lyric,
    scene,
};
use crate::{
    app::App,
    model::*,
    ui::{theme::*, timeline::*},
};
use eframe::egui::*;
use std::collections::BTreeSet;

pub fn note_rect(note: &Note, offset: i64, pitch_offset: i32, rect: Rect, app: &App) -> Rect {
    let x = x_at(rect, &app.view.piano, beats(note.onset + offset));
    let y = rect.top()
        + (app.view.top_pitch - (note.pitch + pitch_offset) as f32) * app.view.row_height;
    Rect::from_min_size(
        pos2(x, y + 1.0),
        vec2(
            (beats(note.duration) * app.view.piano.beat_width as f64) as f32,
            app.view.row_height - 2.0,
        ),
    )
}
pub fn show(app: &mut App, ui: &mut Ui) {
    let all = ui.max_rect();
    if ui.rect_contains_pointer(all) && ui.input(|input| input.pointer.any_pressed()) {
        crate::ui::lanes::parameter_deactivate(app);
        crate::ui::phonemes::phoneme_deactivate(app);
    }
    let phoneme_dragging = crate::ui::phonemes::phoneme_dragging(app);
    ui.painter().rect_filled(all, 0, PANEL);
    let ruler = Rect::from_min_max(
        pos2(all.left() + GUTTER, all.top()),
        pos2(all.right(), all.top() + 38.0),
    );
    let canvas = Rect::from_min_max(
        pos2(ruler.left(), ruler.bottom() + 20.0),
        all.max - vec2(0.0, 6.0),
    );
    if canvas.height() < 8.0 {
        return;
    }
    ui.ctx().data_mut(|data| data.insert_temp(Id::new("piano-canvas-width"), canvas.width()));
    let pan = app.view_pan.drag(
        ui,
        crate::ui::view_pan::Target::Piano,
        Rect::from_min_max(all.min, canvas.max - vec2(6.0, 0.0)),
        app.gesture.is_none() && app.group_gesture.is_none() && !phoneme_dragging,
    );
    app.view.piano.start =
        (app.view.piano.start - pan.x as f64 / app.view.piano.beat_width as f64).max(0.0);
    let response = ui.interact(canvas, Id::new("piano-canvas"), Sense::click_and_drag());
    let vertical = if phoneme_dragging
        || matches!(app.gesture, Some(Gesture::Pitch(_) | Gesture::Anchor(_)))
    {
        0.0
    } else if app.view_pan.is_active() {
        pan.y
    } else {
        scroll(ui, &response, canvas, &mut app.view.piano)
    };
    app.view.top_pitch = (app.view.top_pitch + vertical / app.view.row_height).clamp(12.0, 127.0);
    // Explicit wheel/pan activity takes priority over playback following.
    let center_pause = Id::new("piano-auto-pitch-pause");
    let now = ui.input(|input| input.time);
    if vertical.abs() > f32::EPSILON || app.view_pan.is_active() {
        ui.ctx().data_mut(|data| data.insert_temp(center_pause, now + 1.2));
    }
    let pause_until = ui.ctx().data_mut(|data| data.get_temp::<f64>(center_pause)).unwrap_or(0.0);
    if app.view.auto_pitch_center
        && app.editor.transport.playing
        && now >= pause_until
        && app.gesture.is_none()
        && app.group_gesture.is_none()
        && app.audio_drag.is_none()
        && !phoneme_dragging
        && !app.view_pan.is_active()
        && let Some(median) = median_playback_pitch(app)
    {
        let centered = median as f32 + canvas.height() / (2.0 * app.view.row_height);
        let target = centered.clamp(12.0, 127.0);
        // Keep the current view while the median pitch stays inside a
        // three-semitone dead zone. Follow larger movement smoothly.
        let delta = target - app.view.top_pitch;
        if delta.abs() > 3.0 {
            app.view.top_pitch = (app.view.top_pitch + delta * 0.18).clamp(12.0, 127.0);
            ui.ctx().request_repaint();
        }
    }
    let follow_id = Id::new("piano-followed-playhead-position");
    let previous = ui.ctx().data_mut(|d| d.get_temp::<i64>(follow_id));
    let paused_seek = !app.editor.transport.playing
        && previous.is_some_and(|old| old != app.editor.transport.position);
    let follow = app.editor.transport.playing || paused_seek;
    follow_playhead(
        ui,
        Rect::from_min_max(ruler.min, canvas.max),
        &mut app.view.piano,
        app.editor.transport.position,
        follow
            && app.gesture.is_none()
            && app.group_gesture.is_none()
            && app.audio_drag.is_none()
            && !phoneme_dragging
            && !app.view_pan.is_active(),
    );
    if !ui.input(|i| i.pointer.any_down()) && !app.view_pan.is_active() {
        ui.ctx().data_mut(|d| d.insert_temp(follow_id, app.editor.transport.position));
    }
    super::background::draw(app, ui, canvas);
    super::keyboard::show(app, ui, all, canvas);
    super::scale_guide::draw_tonic_lines(app, ui, canvas);
    grid(
        ui,
        Rect::from_min_max(pos2(ruler.left(), ruler.bottom()), canvas.max),
        &app.view.piano,
        &app.editor.project.time,
        app.view.step(),
    );
    loop_region::show(
        ui,
        Id::new("piano-ruler"),
        ruler,
        &app.view.piano,
        &mut app.editor.transport,
        loop_region::RulerOptions {
            marker_step: app.view.snap.then_some(app.view.step()),
            seek_step: app.view.snap.then_some(app.view.step()),
            enabled: !app.view_pan.is_active()
                && app.gesture.is_none()
                && app.group_gesture.is_none()
                && app.audio_drag.is_none()
                && !phoneme_dragging,
        },
        app.preferences.language,
    );
    draw_ruler(
        ui,
        ruler,
        &app.view.piano,
        &app.editor.project.time,
        app.view.step(),
    );
    let groups = scene::snapshot(app);
    if !app.editor.current_track().is_instrumental()
        && app.group_gesture.is_none()
        && !app.view_pan.is_active()
        && !phoneme_dragging
    {
        if app.piano_tool == super::Tool::PitchDraw {
            super::pitch_edit::interaction(app, ui, &response, canvas);
        } else if app.piano_tool == super::Tool::PitchAnchor {
            super::pitch_anchor::interaction(app, ui, &response, canvas);
        } else {
            interaction(app, ui, &response, canvas, &groups);
        }
    }
    let groups = scene::snapshot(app);
    super::background_notes::draw(app, ui, canvas);
    super::rendered::waveforms(app, ui, canvas, &groups);
    scene::draw_notes(app, ui, canvas, &groups);
    super::rendered::phonemes(app, ui, canvas, &groups);
    super::rendered::text_editor(app, ui, canvas);
    super::rendered::pitch(app, ui, canvas, &groups);
    super::pitch_edit::draw(app, ui, canvas, &groups);
    super::pitch_anchor::draw(app, ui, canvas);
    super::retakes::show(app, ui, canvas, &groups);
    scene::headers(app, ui, canvas, &groups);
    if let Some(Gesture::Select { origin, .. }) = &app.gesture
        && let Some(p) = ui.input(|i| i.pointer.interact_pos())
    {
        let r = Rect::from_two_pos(*origin, p).intersect(canvas);
        ui.painter().rect_filled(r, 0, GREEN.gamma_multiply(0.12));
        ui.painter()
            .rect_stroke(r, 0, Stroke::new(1.0, GREEN), StrokeKind::Inside);
    }
    playhead(
        ui,
        Rect::from_min_max(ruler.min, canvas.max),
        ruler.bottom(),
        &app.view.piano,
        app.editor.transport.position,
    );
    if !app.piano_tool.is_pitch()
        && let Some(reference) = app.editor.active_ref().cloned()
    {
        lyric(app, ui, canvas, &reference);
    }
    crate::ui::scrollbars::horizontal(
        ui,
        "piano-scrollbar",
        Rect::from_min_max(
            pos2(canvas.left() + 2.0, all.bottom() - 6.0),
            all.max - vec2(8.0, 0.0),
        ),
        &mut app.view.piano,
        beats(app.editor.project.end()) + 32.0,
    );
    let before_scrollbar = app.view.top_pitch;
    crate::ui::scrollbars::vertical(
        ui,
        Rect::from_min_max(pos2(canvas.right() - 6.0, canvas.top()), canvas.max),
        &mut app.view.top_pitch,
        app.view.row_height,
    );
    if (app.view.top_pitch - before_scrollbar).abs() > f32::EPSILON {
        let now = ui.input(|input| input.time);
        ui.ctx().data_mut(|data| {
            data.insert_temp(Id::new("piano-auto-pitch-pause"), now + 1.2)
        });
    }
}

/// Prefer notes near the playback cursor; fall back to the current visible
/// group when the cursor is between phrases. Using a median ignores outliers.
fn median_playback_pitch(app: &App) -> Option<i32> {
    let reference = app.editor.active_ref()?;
    let cursor = app.editor.transport.position;
    // Follow only notes that are actually sounding at the transport cursor.
    let mut pitches: Vec<i32> = app.editor.notes().iter()
        .filter_map(|note| {
            let (start, end) = reference.note_range(note)?;
            (start <= cursor && cursor < end)
                .then_some(note.pitch + reference.pitch_offset)
        })
        .collect();
    // In a rest, keep the current vertical viewport rather than jumping to
    // unrelated notes in the group.
    if pitches.is_empty() {
        return None;
    }
    pitches.sort_unstable();
    Some(pitches[pitches.len() / 2])
}
fn interaction(
    app: &mut App,
    ui: &mut Ui,
    response: &Response,
    canvas: Rect,
    groups: &[scene::SceneGroup],
) {
    let lang = app.preferences.language;
    let hit = response
        .interact_pointer_pos()
        .and_then(|p| scene::hit_note(app, groups, canvas, p));
    if let Some(p) = ui.input(|i| i.pointer.hover_pos())
        && let Some((r, n)) = scene::hit_note(app, groups, canvas, p)
    {
        let rect = note_rect(&n, r.blick_offset, r.pitch_offset, canvas, app);
        if (p.x - rect.left()).abs() < 6.0 || (p.x - rect.right()).abs() < 6.0 {
            response
                .clone()
                .on_hover_cursor(CursorIcon::ResizeHorizontal);
        }
    }
    if response.double_clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        app.commit_lyric();
        if super::rendered::begin_text_edit(app, canvas, groups, p) {
            // Editing a rendered phoneme label takes precedence over creating
            // a note at the same double-click location.
        } else if let Some((reference, note)) = hit {
            app.editor.select_reference(app.editor.track, reference.id);
            app.editor.selected = BTreeSet::from([note.id]);
            app.edit_lyric();
        } else {
            app.editor.begin(app.preferences.language.text("创建音符", "Create note", "ノートを作成"));
            let at = ticks(app.view.snap_beat(beat_at(canvas, &app.view.piano, p.x))).max(0);
            if app
                .editor
                .add_note(at, QUARTER, pitch_at(app, canvas, p.y))
                .is_none()
            {
                app.status = lang.text("请在音符组范围内创建音符，或在编排中插入新的音符组", "Create notes inside a group, or insert a new group in the arrangement", "グループ内にノートを作成するか、アレンジに新しいグループを追加してください").into();
            }
            app.editor.finish();
        }
    } else if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        app.commit_lyric();
        if let Some((reference, note)) = hit {
            app.editor.select_reference(app.editor.track, reference.id);
            if ui.input(|i| i.modifiers.ctrl || i.modifiers.shift) {
                if !app.editor.selected.remove(&note.id) {
                    app.editor.selected.insert(note.id);
                }
            } else {
                app.editor.selected = BTreeSet::from([note.id]);
            }
        } else {
            let at = ticks(beat_at(canvas, &app.view.piano, p.x));
            if let Some(id) = app.editor.group_at(at) {
                app.editor.select_reference(app.editor.track, id);
            } else {
                app.editor.select_track(app.editor.track);
            }
            app.editor.selected.clear();
        }
    }
    if response.secondary_clicked()
        && let Some(p) = response.interact_pointer_pos()
        && let Some((reference, note)) = scene::hit_note(app, groups, canvas, p)
    {
        app.commit_lyric();
        app.editor.select_reference(app.editor.track, reference.id);
        if !app.editor.selected.contains(&note.id) {
            app.editor.selected = BTreeSet::from([note.id]);
        }
    }
    response.context_menu(|ui| {
        let selected = !app.editor.selected.is_empty();
        if ui
            .add_enabled(selected, Button::new(lang.text("填入歌词…", "Fill lyrics…", "歌詞を入力…")).shortcut_text("Ctrl+L"))
            .clicked()
        {
            crate::ui::fill_lyrics::open(app);
            ui.close();
        }
        if ui.add_enabled(selected, Button::new(lang.text("静音音符", "Mute notes", "ノートをミュート"))).clicked() {
            app.editor.mute_selected();
            ui.close();
        }
        ui.separator();
        crate::ui::retakes::commands(app, ui);
        ui.separator();
        if ui
            .add_enabled(selected, Button::new(lang.text("复制", "Copy", "コピー")).shortcut_text("Ctrl+C"))
            .clicked()
        {
            crate::ui::chrome::edit::copy(app, ui.ctx());
            ui.close();
        }
        if ui
            .add_enabled(selected, Button::new(lang.text("剪切", "Cut", "切り取り")).shortcut_text("Ctrl+X"))
            .clicked()
        {
            crate::ui::chrome::edit::cut(app, ui.ctx());
            ui.close();
        }
        if ui
            .add_enabled(
                crate::ui::chrome::edit::can_paste(app),
                Button::new(lang.text("粘贴", "Paste", "貼り付け")).shortcut_text("Ctrl+V"),
            )
            .on_hover_text(lang.text("在播放游标处粘贴音符", "Paste notes at the playhead", "再生位置にノートを貼り付け"))
            .clicked()
        {
            crate::ui::chrome::edit::paste(app, ui.ctx());
            ui.close();
        }
        if ui
            .add_enabled(selected, Button::new(lang.text("删除所选音符", "Delete selected notes", "選択したノートを削除")))
            .clicked()
        {
            app.editor.delete_selected();
            ui.close();
        }
    });
    if response.drag_started_by(PointerButton::Primary) {
        start_gesture(app, ui, canvas, groups);
    }
    if response.dragged_by(PointerButton::Primary) {
        update_gesture(app, ui, canvas);
    }
    if response.drag_stopped_by(PointerButton::Primary) {
        app.editor.finish();
        app.gesture = None;
    }
}
