mod export;

use super::controls::*;
use crate::{
    app::App,
    model::{Extra, Track},
    ui::{theme::*, track_color::TrackColors},
};
use eframe::egui::*;

pub(super) fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let mut config = app.editor.project.render_config.clone();
    let mut tracks: Vec<_> = app
        .editor
        .project
        .tracks
        .iter()
        .map(track_enabled)
        .collect();
    let exporting = app.synthesis.is_exporting();
    let editing = app.editor.is_editing();
    padded(ui, |ui| {
        if editing || exporting {
            ui.disable();
        }
        ui.weak(lang.text("目标文件夹", "Destination folder", "保存先フォルダー"));
        let directory_before = string(&config, "destination");
        let mut directory = directory_before.clone();
        ui.horizontal(|ui| {
            let width = (ui.available_width() - 34.0).max(20.0);
            ui.add_sized(
                vec2(width, 28.0),
                TextEdit::singleline(&mut directory)
                    .vertical_align(Align::Center)
                    .margin(vec2(7.0, 5.0))
                    .background_color(BORDER),
            );
            if folder_button(ui, lang).clicked() {
                let mut picker = rfd::FileDialog::new().set_title(lang.text(
                    "选择导出文件夹",
                    "Select export folder",
                    "書き出し先を選択",
                ));
                if !directory.is_empty() {
                    picker = picker.set_directory(&directory);
                }
                if let Some(path) = picker.pick_folder() {
                    directory = path.display().to_string();
                }
            }
        });
        if directory != directory_before {
            config.insert("destination".into(), directory.into());
        }
        ui.weak(lang.text("文件名称", "File name", "ファイル名"));
        let mut name = string(&config, "filename");
        if field(ui, &mut name).changed() {
            config.insert("filename".into(), name.into());
        }
    });
    section(ui, lang.text("音轨", "Tracks", "トラック"), |ui| {
        if editing || exporting {
            ui.disable();
        }
        let before = config
            .get("exportMixDown")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let mut mix = before;
        Frame::new()
            .fill(BACKGROUND)
            .inner_margin(4)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 3.0;
                track_row(
                    ui,
                    &mut mix,
                    lang.text("混音导出", "Mixdown", "ミックスダウン"),
                    TEXT,
                );
                for (track, enabled) in app.editor.project.tracks.iter().zip(&mut tracks) {
                    track_row(ui, enabled, &track.name, TrackColors::new(track).accent);
                }
            });
        pair(ui, |ui, width| {
            if ui
                .add_sized(
                    vec2(width, 28.0),
                    Button::new(lang.text("全选", "Select all", "すべて選択")),
                )
                .clicked()
            {
                mix = true;
                tracks.fill(true);
            }
            if ui
                .add_sized(
                    vec2(width, 28.0),
                    Button::new(lang.text("取消全选", "Deselect all", "すべて解除")),
                )
                .clicked()
            {
                mix = false;
                tracks.fill(false);
            }
        });
        if mix != before {
            config.insert("exportMixDown".into(), mix.into());
        }
    });
    section(ui, lang.text("格式", "Format", "形式"), |ui| {
        if editing || exporting {
            ui.disable();
        }
        numeric(
            ui,
            &mut config,
            "numChannels",
            lang.text("声道", "Channels", "チャンネル"),
            &[
                (1, lang.text("单声道", "Mono", "モノラル")),
                (2, lang.text("立体声", "Stereo", "ステレオ")),
            ],
            lang,
        );
        ui.weak(lang.text("气声输出", "Aspiration output", "息成分の書き出し"));
        ui.small(lang.text(
            "基于 vocoder 的 primary / secondary 原始分量导出；分离模式的主音与气声分别保留一致的采样时间、音量和声像。",
            "Exports native vocoder primary/secondary components with matching sample timing, gain and pan.",
            "ボコーダーの primary / secondary 成分を同じサンプル時刻、音量、パンで書き出します。",
        ));
        let before = match config
            .get("aspirationFormat")
            .map(serde_json::Value::as_str)
        {
            None | Some(Some("")) => "noAspiration",
            Some(Some(value)) => value,
            Some(None) => "",
        };
        let mut aspiration = before.to_owned();
        combo(
            ui,
            "aspiration-format",
            &mut aspiration,
            &[
                (
                    "noAspiration".to_owned(),
                    lang.text("无（完整混合）", "None (full mix)", "なし（完全ミックス）"),
                ),
                (
                    "separateAspiration".to_owned(),
                    lang.text("分离", "Separate", "分離"),
                ),
                (
                    "onlyMain".to_owned(),
                    lang.text("仅主音", "Primary only", "主成分のみ"),
                ),
                (
                    "onlyAspiration".to_owned(),
                    lang.text("仅气声", "Aspiration only", "息成分のみ"),
                ),
            ],
            lang,
        );
        if aspiration != before {
            config.insert("aspirationFormat".into(), aspiration.into());
        }
        numeric(
            ui,
            &mut config,
            "bitDepth",
            lang.text("位深度", "Bit depth", "ビット深度"),
            &[
                (16, lang.text("16 位", "16-bit", "16ビット")),
                (24, lang.text("24 位", "24-bit", "24ビット")),
                (
                    32,
                    lang.text("32 位浮点", "32-bit float", "32ビット浮動小数点"),
                ),
            ],
            lang,
        );
        numeric(
            ui,
            &mut config,
            "sampleRate",
            lang.text("采样率", "Sample rate", "サンプルレート"),
            &[
                (44100, "44100 Hz"),
                (48000, "48000 Hz"),
                (96000, "96000 Hz"),
            ],
            lang,
        );
    });
    let tracks_changed = app
        .editor
        .project
        .tracks
        .iter()
        .map(track_enabled)
        .ne(tracks.iter().copied());
    if config != app.editor.project.render_config || tracks_changed {
        app.editor
            .begin(lang.text("渲染设置", "Render settings", "レンダリング設定"));
        app.editor.project.render_config = config;
        for (track, enabled) in app.editor.project.tracks.iter_mut().zip(&tracks) {
            if track_enabled(track) != *enabled {
                track
                    .extra
                    .insert("renderEnabled".into(), (*enabled).into());
            }
        }
        app.editor.finish();
        app.synthesis.export_result = None;
    }
    export::show(app, ui, tracks.iter().any(|&enabled| enabled));
    section(
        ui,
        lang.text("合成状态", "Synthesis status", "合成ステータス"),
        |ui| super::super::synthesis::render_status(app, ui),
    );
}

fn track_enabled(track: &Track) -> bool {
    track
        .extra
        .get("renderEnabled")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

fn string(config: &Extra, key: &str) -> String {
    config
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned()
}
fn numeric(
    ui: &mut Ui,
    config: &mut Extra,
    key: &str,
    label: &str,
    values: &[(u64, &str)],
    lang: crate::ui::settings::UiLanguage,
) {
    ui.weak(label);
    let before = config
        .get(key)
        .map_or(values[0].0, |value| value.as_u64().unwrap_or(0));
    let mut value = before;
    combo(ui, key, &mut value, values, lang);
    if value != before {
        config.insert(key.into(), value.into());
    }
}
fn track_row(ui: &mut Ui, checked: &mut bool, name: &str, color: Color32) {
    let row = Frame::new()
        .fill(PANEL)
        .corner_radius(2)
        .inner_margin(Margin {
            left: 12,
            right: 6,
            top: 5,
            bottom: 5,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.checkbox(checked, name);
        });
    ui.painter().rect_filled(
        Rect::from_min_size(row.response.rect.min, vec2(3.0, row.response.rect.height())),
        0,
        color,
    );
}
fn folder_button(ui: &mut Ui, lang: crate::ui::settings::UiLanguage) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    ui.painter().rect(
        rect,
        3,
        if response.hovered() {
            BUTTON_HOVER
        } else {
            BORDER
        },
        Stroke::new(0.6, BUTTON),
        StrokeKind::Inside,
    );
    let c = rect.center();
    let points = [
        (-5.5, -3.0),
        (-5.5, -5.0),
        (-1.0, -5.0),
        (1.0, -3.0),
        (5.5, -3.0),
        (5.5, 5.0),
        (-5.5, 5.0),
        (-5.5, -3.0),
    ];
    ui.painter().add(Shape::line(
        points.into_iter().map(|(x, y)| c + vec2(x, y)).collect(),
        Stroke::new(1.1, MUTED),
    ));
    response.on_hover_text(lang.text(
        "选择目标文件夹",
        "Choose destination folder",
        "書き出し先フォルダーを選択",
    ))
}
