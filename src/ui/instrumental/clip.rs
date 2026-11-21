use super::AudioDrag;
use crate::{
    app::App,
    model::*,
    ui::{theme::*, timeline::*, track_color::TrackColors},
};
use eframe::egui::*;

pub fn show(app: &mut App, ui: &mut Ui, body: Rect, track: &Track) {
    let lang = app.preferences.language;
    let row = ui.interact(body, Id::new(("audio-row", track.id)), Sense::click());
    row.context_menu(|ui| {
        if ui.button(lang.text("导入 / 替换音频…", "Import / replace audio…", "音声をインポート／置換…")).clicked() {
            ui.close();
            app.import_audio(Some(track.id), ui.ctx());
        }
    });
    let Some(clip) = track.audio_clip() else {
        let response = ui.interact(body, Id::new(("empty-audio", track.id)), Sense::click());
        let loading = app
            .audio_import
            .as_ref()
            .is_some_and(|import| import.track == track.id);
        ui.painter().text(
            body.left_center() + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            if loading {
                lang.text("正在导入音频…", "Importing audio…", "音声を読み込み中…")
            } else {
                lang.text("双击导入伴奏音频…", "Double-click to import instrumental audio…", "ダブルクリックで伴奏音声をインポート…")
            },
            FontId::proportional(12.0),
            MUTED,
        );
        if response.double_clicked() {
            app.import_audio(Some(track.id), ui.ctx());
        }
        response.context_menu(|ui| {
            if ui.button(lang.text("导入音频…", "Import audio…", "音声をインポート…")).clicked() {
                ui.close();
                app.import_audio(Some(track.id), ui.ctx());
            }
        });
        return;
    };
    let time = &app.editor.project.time;
    let x = |seconds| {
        x_at(
            body,
            &app.view.arrangement,
            beats(time.position_at(seconds)),
        )
    };
    let region = Rect::from_min_max(
        pos2(x(clip.onset_seconds), body.top() + 1.0),
        pos2(x(clip.end_seconds()), body.bottom() - 1.0),
    );
    let visible = region.intersect(body).intersect(ui.clip_rect());
    if !visible.is_positive() {
        return;
    }
    let colors = TrackColors::new(track);
    let fill = colors.note;
    let painter = ui.painter().with_clip_rect(body.intersect(ui.clip_rect()));
    painter.rect_filled(region, 3, fill);
    let painter = painter.with_clip_rect(visible);
    let error = app.synthesis.instrumentals.error(&clip.path);
    let asset = app.synthesis.instrumentals.asset(&clip.path);
    let title = if error.is_some() {
        format!("{} · {}", clip.name(), lang.text("加载失败", "Load failed", "読み込み失敗"))
    } else if asset.is_none() {
        format!("{} · {}", clip.name(), lang.text("加载中…", "Loading…", "読み込み中…"))
    } else {
        clip.name().to_owned()
    };
    painter.text(
        region.min + vec2(6.0, 4.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(11.0),
        Color32::WHITE,
    );
    if let Some(asset) = asset {
        let height = (region.height() - 21.0) / asset.waveforms.len() as f32;
        let pixel = ui.ctx().pixels_per_point().recip();
        for (channel, wave) in asset.waveforms.iter().enumerate() {
            let center = region.top() + 20.0 + height * (channel as f32 + 0.5);
            let mut mesh = Mesh::default();
            for column in 0..(visible.width() / pixel).ceil() as usize {
                let left = visible.left() + column as f32 * pixel;
                let right = (left + pixel).min(visible.right());
                let seconds = |x| {
                    time.seconds_at(ticks(beat_at(body, &app.view.arrangement, x)))
                        - clip.onset_seconds
                };
                if let Some((min, max)) = wave.peak_range(seconds(left), seconds(right)) {
                    let top = center - max.clamp(-1.0, 1.0) * height * 0.46;
                    let bottom = center - min.clamp(-1.0, 1.0) * height * 0.46;
                    mesh.add_colored_rect(
                        Rect::from_min_max(
                            pos2(left, top),
                            pos2(right, bottom.max(top + pixel * 0.4)),
                        ),
                        colors.highlight,
                    );
                }
            }
            painter.add(Shape::mesh(mesh));
        }
    }
    let hover = error.map_or_else(
        || {
            format!(
                "{}\n{:.2} {} · {}",
                clip.path.display(),
                clip.duration_seconds,
                lang.text("秒", "s", "秒"),
                lang.text("拖动调整位置，双击替换音频", "Drag to reposition, double-click to replace audio", "ドラッグで位置を移動、ダブルクリックで音声を置換")
            )
        },
        str::to_owned,
    );
    let response = ui
        .interact(
            visible,
            Id::new(("audio-clip", track.id)),
            Sense::click_and_drag(),
        )
        .on_hover_cursor(CursorIcon::Grab)
        .on_hover_text(hover);
    if response.clicked() || response.secondary_clicked() {
        app.commit_lyric();
    }
    if response.double_clicked() {
        app.import_audio(Some(track.id), ui.ctx());
    }
    if response.drag_started_by(PointerButton::Primary)
        && let Some(pointer) = ui.input(|i| i.pointer.press_origin())
    {
        app.commit_lyric();
        app.editor.begin("移动伴奏");
        app.audio_drag = Some(AudioDrag {
            track: track.id,
            pointer_x: pointer.x,
            beat: beats(app.editor.project.time.position_at(clip.onset_seconds)),
            beat_width: app.view.arrangement.beat_width,
        });
    }
    response.context_menu(|ui| {
        if ui.button(lang.text("替换音频…", "Replace audio…", "音声を置換…")).clicked() {
            ui.close();
            app.import_audio(Some(track.id), ui.ctx());
        }
        if ui.button(lang.text("重新加载音频", "Reload audio", "音声を再読み込み")).clicked() {
            app.reload_audio(track.id, ui.ctx());
            ui.close();
        }
        if ui.button(lang.text("移到工程开头", "Move to project start", "プロジェクトの先頭に移動")).clicked() {
            app.editor.begin("移动伴奏");
            app.editor.move_audio_clip(track.id, 0.0);
            app.editor.finish();
            ui.close();
        }
        if ui.button(lang.text("移除音频", "Remove audio", "音声を削除")).clicked() {
            app.editor.set_audio_clip(track.id, None);
            ui.close();
        }
    });
}
