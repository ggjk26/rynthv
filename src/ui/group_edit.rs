use crate::{app::App, editor::GroupEdit, model::*};
use eframe::egui::*;

pub struct GroupGesture {
    pub original: GroupRef,
    pub source_track: EntityId,
    pub origin: Pos2,
    pub kind: GroupEdit,
    pub beat_width: f32,
    pub arrangement: bool,
}
pub fn start(
    app: &mut App,
    original: &GroupRef,
    track: EntityId,
    origin: Pos2,
    kind: GroupEdit,
    beat_width: f32,
    arrangement: bool,
) {
    app.commit_lyric();
    app.editor.select_reference(track, original.id);
    app.editor.begin(match kind {
        GroupEdit::Move => "移动音符组",
        _ => "调整音符组范围",
    });
    app.group_gesture = Some(GroupGesture {
        original: original.clone(),
        source_track: track,
        origin,
        kind,
        beat_width,
        arrangement,
    });
}
pub fn update(app: &mut App, ctx: &Context, target_track: Option<EntityId>) {
    let Some(drag) = &app.group_gesture else {
        return;
    };
    let Some(pointer) = ctx.input(|i| i.pointer.interact_pos()) else {
        return;
    };
    let raw = ticks((pointer.x - drag.origin.x) as f64 / drag.beat_width as f64);
    let anchor = match drag.kind {
        GroupEdit::Right => drag.original.visible_end(),
        _ => drag.original.blick_absolute_begin,
    };
    let delta = if app.view.snap && !ctx.input(|i| i.modifiers.alt) {
        ticks(app.view.snap_beat(beats(anchor.saturating_add(raw)))) - anchor
    } else {
        raw
    };
    let target = if matches!(drag.kind, GroupEdit::Move) {
        target_track.unwrap_or(app.editor.track)
    } else {
        drag.source_track
    };
    let minimum = if app.view.snap {
        ticks(app.view.step())
    } else {
        ticks(0.01)
    };
    app.editor
        .edit_group(&drag.original, target, drag.kind, delta, minimum);
}
pub fn finish(app: &mut App) {
    app.editor.finish();
    app.group_gesture = None;
}
pub fn context_menu(app: &mut App, ui: &mut Ui, reference: &GroupRef) {
    let lang = app.preferences.language;
    let voice_options = app.synthesis.library.catalog().map(|catalog| {
        catalog.entries.iter()
            .filter(|entry| entry.availability.is_loadable())
            .map(|entry| (
                entry.database_id.clone(),
                entry.display_name.clone(),
                entry.revision,
            ))
            .collect::<Vec<_>>()
    }).unwrap_or_default();
    // The context menu may outlive the selection that opened it.
    // Resolve the owner from the reference itself, not the current editor track.
    let Some(track_id) = app.editor.project.tracks.iter()
        .find(|track| track.references().any(|item| item.id == reference.id))
        .map(|track| track.id)
    else {
        return;
    };
    let reference_id = reference.id;
    ui.menu_button(lang.text("设置此音符组声库", "Set group voice", "グループの歌声を設定"), |ui| {
        if ui.button(lang.text("继承音轨声库", "Inherit track voice", "トラックの歌声を継承")).clicked() {
            app.commit_lyric();
            app.commit_phoneme_text();
            if let Err(error) = app.editor.set_reference_voice(track_id, reference_id, None) {
                app.error = Some(error);
            }
            ui.close();
        }
        ui.separator();
        if voice_options.is_empty() {
            ui.label(lang.text("没有可用声库，请先配置声库路径", "No usable voices; configure voice paths first", "利用できる歌声がありません。歌声フォルダーを設定してください"));
        }
        for (id, name, revision) in &voice_options {
            if ui.button(format!("{name} · {revision}")).clicked() {
                app.commit_lyric();
                app.commit_phoneme_text();
                if let Err(error) = app.editor.set_reference_voice(
                    track_id, reference_id, Some((id, name, *revision))
                ) {
                    app.error = Some(error);
                }
                ui.close();
            }
        }
    });
    if ui.button(lang.text("重命名音符组…", "Rename group…", "グループ名を変更…")).clicked() {
        if let Some(group) = app.editor.project.group(&reference.group_id) {
            app.rename = Some(super::rename::Rename::group(reference.id, &group.name));
        }
        ui.close();
    }
    let ordinary = app.editor.project.tracks.iter()
        .find(|track| track.id == track_id)
        .is_some_and(|track| track.groups.iter().any(|item| item.id == reference_id));
    if ui
        .add_enabled(ordinary, Button::new(lang.text("删除音符组", "Delete group", "グループを削除")))
        .clicked()
    {
        app.editor.select_reference(track_id, reference_id);
        app.editor.delete_group(reference_id);
        ui.close();
    }
}
