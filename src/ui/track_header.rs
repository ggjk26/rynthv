mod source;

use super::{icons::Icon, theme::*};
use crate::{app::App, model::Track};
use eframe::egui::*;

pub fn show(app: &mut App, ui: &mut Ui, rect: Rect, track: &Track) {
    let lang = app.preferences.language;
    let response = ui.interact(rect, Id::new(("track", track.id)), Sense::click());
    if response.clicked() && !track.is_instrumental() {
        app.commit_lyric();
        app.commit_phoneme_text();
        let modifiers = ui.input(|input| input.modifiers);
        if modifiers.shift {
            let tracks = &app.editor.project.tracks;
            let anchor = tracks.iter().position(|candidate| candidate.id == app.editor.track);
            let target = tracks.iter().position(|candidate| candidate.id == track.id);
            if let (Some(anchor), Some(target)) = (anchor, target) {
                for candidate in &tracks[anchor.min(target)..=anchor.max(target)] {
                    if !candidate.is_instrumental() {
                        app.selected_tracks.insert(candidate.id);
                    }
                }
            }
        } else if modifiers.ctrl || modifiers.command {
            if !app.selected_tracks.remove(&track.id) {
                app.selected_tracks.insert(track.id);
            }
        } else {
            app.selected_tracks.clear();
            app.selected_tracks.insert(track.id);
        }
        if app.selected_tracks.contains(&track.id) {
            app.editor.select_track(track.id);
        }
    }
    if response.double_clicked() {
        app.rename = Some(super::rename::Rename::track(track.id, &track.name));
    }
    // Snapshot the catalog before the menu mutably edits the project.
    let available_voices = app.synthesis.library.catalog().map(|catalog| {
        catalog.entries.iter()
            .filter(|entry| entry.availability.is_loadable())
            .map(|entry| (entry.database_id.clone(), entry.display_name.clone(), entry.revision))
            .collect::<Vec<_>>()
    }).unwrap_or_default();
    response.context_menu(|ui| {
        let mut targets = app.selected_tracks.clone();
        if !targets.contains(&track.id) {
            targets.clear();
            targets.insert(track.id);
        }
        // Instrumental rows have no singer and are never added to the bulk set.
        targets.retain(|id| app.editor.project.tracks.iter()
            .any(|candidate| candidate.id == *id && !candidate.is_instrumental()));
        let bulk = targets.len() > 1;
        if !track.is_instrumental() {
            ui.menu_button(if bulk {
                lang.text("设置所选音轨歌声", "Set selected tracks' voices", "選択したトラックの歌声を設定")
            } else {
                lang.text("设置音轨歌声", "Set track voice", "トラックの歌声を設定")
            }, |ui| {
                if ui.button(lang.text("取消音轨歌声", "Remove track voice", "トラックの歌声を解除")).clicked() {
                    app.commit_lyric();
                    app.commit_phoneme_text();
                    if let Err(error) = app.editor.set_tracks_voice(&targets, None) {
                        app.error = Some(error);
                    }
                    ui.close();
                }
                ui.separator();
                if available_voices.is_empty() {
                    ui.label(lang.text("没有可用声库", "No usable voices", "利用できる歌声がありません"));
                }
                for (id, name, revision) in &available_voices {
                    if ui.button(format!("{name} · {revision}")).clicked() {
                        app.commit_lyric();
                        app.commit_phoneme_text();
                        if let Err(error) = app.editor.set_tracks_voice(&targets, Some((id, name, *revision))) {
                            app.error = Some(error);
                        }
                        ui.close();
                    }
                }
            });
            ui.separator();
        }
        if ui.button(lang.text("重命名音轨…", "Rename track…", "トラック名を変更…")).clicked() {
            app.rename = Some(super::rename::Rename::track(track.id, &track.name));
            ui.close();
        }
        if ui.button(lang.text("新增音轨", "Add track", "トラックを追加")).clicked() {
            app.editor.add_track();
            ui.close();
        }
        if ui.button(lang.text("新增伴奏轨…", "Add instrumental track…", "伴奏トラックを追加…")).clicked() {
            let id = app.editor.add_instrumental_track();
            ui.close();
            app.import_audio(Some(id), ui.ctx());
        }
        let index = app
            .editor
            .project
            .tracks
            .iter()
            .position(|t| t.id == track.id)
            .unwrap();
        if ui.add_enabled(index > 0, Button::new(lang.text("上移音轨", "Move track up", "トラックを上へ移動"))).clicked() {
            app.editor.reorder_track(track.id, -1);
            ui.close();
        }
        if ui
            .add_enabled(
                index + 1 < app.editor.project.tracks.len(),
                Button::new(lang.text("下移音轨", "Move track down", "トラックを下へ移動")),
            )
            .clicked()
        {
            app.editor.reorder_track(track.id, 1);
            ui.close();
        }
        ui.separator();
        if ui
            .add_enabled(app.editor.project.tracks.len() > 1, Button::new(lang.text("删除音轨", "Delete track", "トラックを削除")))
            .clicked()
        {
            app.commit_lyric();
            app.editor.delete_track(track.id);
            ui.close();
        }
        ui.separator();
        if let Some(color) = super::track_color::picker(ui, track.disp_color, lang) {
            app.commit_lyric();
            app.editor.set_track_color(track.id, color);
            ui.close();
        }
    });
    if app.selected_tracks.contains(&track.id) && !track.is_instrumental() {
        ui.painter().rect_stroke(rect.shrink(1.0), 3,
            Stroke::new(1.2, super::track_color::TrackColors::new(track).accent), StrokeKind::Inside);
    }
    let text_rect = Rect::from_min_max(
        rect.min + vec2(16.0, 0.0),
        pos2(rect.right() - 95.0, rect.bottom()),
    );
    let painter = ui
        .painter()
        .with_clip_rect(ui.clip_rect().intersect(text_rect));
    painter.text(
        rect.min + vec2(18.0, 17.0),
        Align2::LEFT_CENTER,
        &track.name,
        FontId::proportional(12.0),
        TEXT,
    );
    source::show(app, ui, rect, text_rect, track);
    if rect.width() < 210.0 {
        return;
    }
    for (field, label, center, min, max, reset) in [
        (
            "gainDecibel",
            lang.text("音量", "Volume", "音量"),
            pos2(rect.right() - 79.0, rect.top() + 23.0),
            -48.0,
            12.0,
            0.0,
        ),
        (
            "pan",
            lang.text("声像", "Pan", "パン"),
            pos2(rect.right() - 49.0, rect.top() + 23.0),
            -1.0,
            1.0,
            0.0,
        ),
    ] {
        let old = track
            .mixer
            .get(field)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(reset);
        let (value, response) = knob(ui, (track.id, field), center, old, min, max, reset);
        let response = response.on_hover_text(format!("{label}  {old:.2}\n{}", lang.text("拖动调整，双击复位", "Drag to adjust; double-click to reset", "ドラッグで調整、ダブルクリックでリセット")));
        if response.drag_started() || response.double_clicked() {
            app.editor.begin(lang.text("调整混音", "Adjust mixer", "ミキサー調整"));
        }
        if value != old {
            app.editor
                .project
                .tracks
                .iter_mut()
                .find(|t| t.id == track.id)
                .unwrap()
                .mixer
                .insert(field.into(), value.into());
        }
        if response.drag_stopped() || response.double_clicked() {
            app.editor.finish();
        }
        if field == "pan" {
            Icon::Pan.paint(
                ui,
                Rect::from_center_size(pos2(center.x, rect.top() + 43.0), vec2(13.0, 13.0)),
                Color32::from_rgb(120, 117, 124),
            );
        } else {
            let x = center.x - 6.0;
            for i in 0..6 {
                ui.painter().vline(
                    x + i as f32 * 2.0,
                    rect.top() + 45.0 - (1.0 + i as f32)..=rect.top() + 45.0,
                    Stroke::new(1.0, MUTED.gamma_multiply(0.55)),
                );
            }
        }
    }
    for (field, text, y) in [("mute", "M", 10.0), ("solo", "S", 30.0)] {
        let value = track
            .mixer
            .get(field)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let r = Rect::from_min_size(pos2(rect.right() - 27.0, rect.top() + y), vec2(16.0, 16.0));
        let response = ui.interact(r, Id::new((track.id, field)), Sense::click());
        ui.painter().rect_filled(
            r,
            2,
            if value {
                GREEN
            } else if response.hovered() {
                BUTTON_HOVER
            } else {
                BUTTON
            },
        );
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(9.0),
            if value { CANVAS } else { TEXT },
        );
        if response
            .on_hover_text(if field == "mute" {
                lang.text("音轨静音", "Mute track", "トラックをミュート")
            } else {
                lang.text("音轨独奏", "Solo track", "トラックをソロ")
            })
            .clicked()
        {
            app.editor.begin(lang.text("音轨混音", "Track mixing", "トラックミキシング"));
            app.editor
                .project
                .tracks
                .iter_mut()
                .find(|t| t.id == track.id)
                .unwrap()
                .mixer
                .insert(field.into(), (!value).into());
            app.editor.finish();
        }
    }
}
fn knob(
    ui: &mut Ui,
    id: (u64, &str),
    center: Pos2,
    value: f64,
    min: f64,
    max: f64,
    reset: f64,
) -> (f64, Response) {
    let rect = Rect::from_center_size(center, vec2(21.0, 25.0));
    let response = ui.interact(rect, Id::new(id), Sense::click_and_drag());
    ui.painter().circle_filled(
        center,
        9.5,
        if response.hovered() {
            BUTTON_HOVER
        } else {
            Color32::from_rgb(77, 75, 81)
        },
    );
    let normalized = if value <= reset {
        ((value - reset) / (reset - min).max(1e-6)) * 0.5 + 0.5
    } else {
        ((value - reset) / (max - reset).max(1e-6)) * 0.5 + 0.5
    };
    let angle = (-225.0 + normalized as f32 * 270.0).to_radians();
    let end = center + vec2(angle.cos(), angle.sin()) * 8.0;
    ui.painter()
        .line_segment([center, end], Stroke::new(1.0, MUTED));
    let next = if response.double_clicked() {
        reset
    } else if response.dragged() {
        (value - response.drag_delta().y as f64 * (max - min) / 160.0).clamp(min, max)
    } else {
        value
    };
    (next, response)
}
