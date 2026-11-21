mod toggle;

use super::{
    child,
    icons::{Icon, IconButton},
    theme::*,
};
use crate::app::App;
use eframe::egui::*;

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let rect = ui.max_rect();
    let controls = Rect::from_center_size(
        pos2(rect.center().x, rect.center().y - 1.0),
        vec2(520.0, 30.0),
    );
    let mut bar = child(ui, controls, "transport-buttons");
    bar.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let button = |icon, label| {
            IconButton::new(icon, label)
                .size(vec2(32.0, 30.0))
                .icon_size(21.0)
        };
        let transport = &mut app.editor.transport;
        if ui
            .add(button(
                if transport.playing {
                    Icon::Pause
                } else {
                    Icon::Play
                },
                lang.text("播放 / 暂停（Space）", "Play / Pause (Space)", "再生／一時停止（Space）"),
            ))
            .clicked()
        {
            transport.toggle();
        }
        if ui.add(button(Icon::Stop, lang.text("停止（Ctrl+Space）", "Stop (Ctrl+Space)", "停止（Ctrl+Space）"))).clicked() {
            transport.stop();
        }
        ui.add_enabled(false, button(Icon::Record, lang.text("录制（尚未接入）", "Record (not available yet)", "録音（未対応）")));
        if ui.add(button(Icon::Rewind, lang.text("跳到开头", "Jump to start", "先頭へ移動"))).clicked() {
            transport.position = 0;
        }
        if ui.add(button(Icon::Forward, lang.text("跳到结尾", "Jump to end", "末尾へ移動"))).clicked() {
            transport.position = app.editor.project.end();
        }
        if ui
            .add(
                button(Icon::Loop, lang.text("显示循环标记 / 循环播放", "Show loop markers / loop playback", "ループ範囲表示／ループ再生"))
                    .selected(transport.looping)
                    .background(BACKGROUND),
            )
            .clicked()
        {
            transport.looping = !transport.looping;
        }
        ui.add_space(22.0);
        let position = transport.position;
        let time = &app.editor.project.time;
        let meter = time.meter_at(time.bar_at(position).0);
        let meter_index = meter.index;
        if display(
            ui,
            &format!("{}/{}", meter.numerator, meter.denominator),
            46.0,
        )
        .on_hover_text(lang.text("编辑当前拍号", "Edit current time signature", "現在の拍子を編集"))
        .clicked()
        {
            super::time_edit::open(app, super::time_edit::Target::Meter(meter_index));
        }
        ui.add_space(1.0);
        if app.bpm_edit.is_some() {
            super::tempo_input::show(app, ui);
        } else if display(
            ui,
            &format!("{:.2}", app.editor.project.time.bpm_at(position)),
            super::tempo_input::WIDTH,
        )
        .on_hover_text(lang.text("编辑当前曲速", "Edit current tempo", "現在のテンポを編集"))
        .clicked()
        {
            super::tempo_input::open(app, position, ui.ctx());
        }
        ui.add_space(1.0);
        let metronome = app.synthesis.metronome_enabled();
        if toggle::show(
            ui,
            Icon::Metronome,
            lang.text("节拍器：播放时打拍，仅用于试听", "Metronome: click during playback (preview only)", "メトロノーム：再生中のクリック（試聴専用）"),
            metronome,
            38.0,
            21.0,
        )
        .clicked()
        {
            app.synthesis.set_metronome(!metronome);
        }
        ui.add_space(16.0);
        let seconds = app.editor.project.time.seconds_at(position).max(0.0);
        display(
            ui,
            &format!(
                "{:02}:{:02}:{:02}",
                (seconds / 60.0) as u64,
                seconds as u64 % 60,
                (seconds.fract() * 100.0) as u64
            ),
            92.0,
        );
        ui.add_space(1.0);
        if toggle::show(
            ui,
            Icon::Return,
            lang.text("暂停播放时返回起播位置", "Return to playback start when pausing", "一時停止で再生開始位置へ戻る"),
            app.editor.transport.return_on_pause,
            34.0,
            20.0,
        )
        .clicked()
        {
            app.editor.transport.return_on_pause = !app.editor.transport.return_on_pause;
        }
    });
    Icon::Brand.paint(
        ui,
        Rect::from_center_size(pos2(rect.right() - 23.0, rect.center().y), vec2(27.0, 27.0)),
        Color32::WHITE,
    );
}
fn display(ui: &mut Ui, text: &str, width: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    ui.painter().rect_filled(rect, 3, PANEL);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::new(18.0, FontFamily::Name("Transport".into())),
        Color32::from_rgb(155, 152, 163),
    );
    response
}
