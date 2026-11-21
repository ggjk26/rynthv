use super::{language, parameters, selected_entry, selector, weak, widgets};
use crate::{
    app::App,
    synthesis::{runtime::VoiceState, settings::VoiceSelection},
    ui::theme::*,
};
use eframe::egui::*;

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    ui.spacing_mut().item_spacing.y = 0.0;
    Frame::new().inner_margin(16).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 8.0;
        selector::selected_card(app, ui);
        weak(ui, lang.text("版本", "Version", "バージョン"));
        version(app, ui);
        weak(ui, lang.text("预设", "Preset", "プリセット"));
        let preset = app
            .editor
            .active_ref()
            .and_then(|reference| reference.extra.get("voicePresetName"))
            .and_then(serde_json::Value::as_str)
            .filter(|name| !name.is_empty())
            .unwrap_or(lang.text("默认", "Default", "デフォルト"));
        widgets::field(ui, preset, true, lang.text("歌声预设编辑尚未接入合成", "Voice preset editing is not connected to synthesis yet", "歌声プリセット編集はまだ合成に反映されません"));
        status(app, ui);
    });
    section(ui, lang.text("语言", "Language", "言語"), |ui| language::show(app, ui));
    section(ui, lang.text("唱法", "Vocal styles", "歌唱スタイル"), |ui| parameters::styles(app, ui));
    section(ui, lang.text("参数", "Parameters", "パラメーター"), |ui| parameters::scalars(app, ui));
    section(ui, lang.text("调音", "Tuning", "チューニング"), |ui| parameters::tuning(app, ui));
    section(ui, lang.text("调式音阶", "Musical scale", "音階"), |ui| {
        crate::ui::scale::controls::panel(app, ui);
    });
}

fn version(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let Some(selected) = app.voice_selection() else {
        widgets::field(ui, lang.text("未选择歌声", "No voice selected", "歌声が選択されていません"), true, lang.text("先选择歌声", "Select a voice first", "まず歌声を選択してください"));
        return;
    };
    let mut revision = selected.revision;
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 28.0;
        ui.visuals_mut().widgets.inactive.bg_fill = BORDER;
        ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::same(4);
        ComboBox::from_id_salt("voice-revision")
            .width(ui.available_width())
            .selected_text(revision.to_string())
            .show_ui(ui, |ui| {
                if let Some(catalog) = app.synthesis.library.catalog() {
                    for entry in catalog
                        .entries
                        .iter()
                        .filter(|entry| entry.database_id == selected.database_id)
                    {
                        ui.add_enabled_ui(entry.availability.is_loadable(), |ui| {
                            ui.selectable_value(
                                &mut revision,
                                entry.revision,
                                entry.revision.to_string(),
                            );
                        })
                        .response
                        .on_disabled_hover_text(lang.text("此版本尚未支持合成", "Synthesis for this revision is unavailable", "このバージョンの合成は未対応です"));
                    }
                }
            });
    });
    if revision != selected.revision {
        app.select_voice(VoiceSelection {
            database_id: selected.database_id,
            revision,
        });
    }
}

fn status(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let failed = matches!(app.voice_state(), VoiceState::Failed(_))
        || app.synthesis.library.settings_error().is_some()
        || app.voice_error().is_some();
    if matches!(app.voice_state(), VoiceState::Loading) || app.synthesis.library.is_scanning() {
        ui.horizontal(|ui| {
            ui.spinner();
            weak(ui, lang.text("正在加载歌声…", "Loading voice…", "歌声を読み込み中…"));
        });
    } else if failed {
        if ui
            .button(RichText::new(lang.text("加载异常 · 管理歌声…", "Load error · Manage voices…", "読み込みエラー · 歌声を管理…")).color(Color32::from_rgb(218, 142, 126)))
            .clicked()
        {
            app.voice_manager = true;
        }
    } else if app.synthesis.library.catalog_error().is_some() {
        if ui.button(lang.text("目录读取异常 · 管理歌声…", "Directory error · Manage voices…", "フォルダーの読み込みエラー · 歌声を管理…")).clicked() {
            app.voice_manager = true;
        }
    } else if selected_entry(app).is_none() && ui.button(lang.text("管理歌声…", "Manage voices…", "歌声を管理…")).clicked() {
        app.voice_manager = true;
    }
}

fn section(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui)) {
    let y = ui.next_widget_position().y;
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        y,
        Stroke::new(1.0, EDGE),
    );
    Frame::new()
        .inner_margin(Margin {
            left: 16,
            right: 16,
            top: 12,
            bottom: 16,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 12.0;
            CollapsingHeader::new(RichText::new(title).font(section_title_font()))
                .id_salt(("voice-section", title))
                .default_open(true)
                .show_unindented(ui, |ui| {
                    ui.set_width(ui.available_width());
                    content(ui);
                });
        });
}
