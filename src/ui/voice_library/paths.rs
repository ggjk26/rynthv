use std::path::Path;

use super::{error_label, text, weak};
use crate::{app::App, ui::settings::UiLanguage};
use eframe::egui::*;

pub fn settings(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    text(ui, RichText::new(lang.text("歌声资源", "Voice resources", "歌声リソース")).strong());
    text(ui, lang.text("SV2 声库根目录（databases）", "SV2 voicebank root (databases)", "SV2 歌声ルート（databases）"));
    path_label(ui, app.synthesis.library.settings.databases.as_deref(), lang);
    ui.horizontal(|ui| {
        if ui.button(lang.text("选择 SV2 databases…", "Choose SV2 databases…", "SV2 databases を選択…")).clicked() {
            choose_databases(app);
        }
        if ui
            .add_enabled(
                !app.synthesis.library.is_scanning()
                    && app.synthesis.library.settings.has_databases(),
                Button::new(lang.text("重新扫描", "Rescan", "再スキャン")),
            )
            .clicked()
        {
            app.synthesis.rescan_voices();
        }
    });
    ui.add_space(8.0);
    text(ui, lang.text("SV1 原生声库根目录", "Native SV1 voicebank root", "SV1 ネイティブ歌声のルート"));
    path_label(ui, app.synthesis.library.settings.sv1_databases.as_deref(), lang);
    if ui.button(lang.text("选择 SV1 databases…", "Choose SV1 databases…", "SV1 databases を選択…")).clicked() {
        choose_sv1_databases(app);
    }
    ui.add_space(8.0);
    text(ui, lang.text("SV Flat 声库根目录（已解包的 NOFS-JSON 或 info.nofs）", "SV Flat root (extracted NOFS-JSON or info.nofs)", "SV Flat ルート（展開済み NOFS-JSON / info.nofs）"));
    path_label(ui, app.synthesis.library.settings.flat_databases.as_deref(), lang);
    if ui.button(lang.text("选择 Flat databases…", "Choose Flat databases…", "Flat databases を選択…")).clicked() {
        choose_flat_databases(app);
    }
    ui.add_space(8.0);
    text(ui, lang.text("Flat 模型根目录（可选，仅用于依赖扫描）", "Flat model root (optional; dependency scan only)", "Flat モデルルート（任意、依存関係の確認のみ）"));
    path_label(ui, app.synthesis.library.settings.flat_models.as_deref(), lang);
    if ui.button(lang.text("选择 Flat models…", "Choose Flat models…", "Flat models を選択…")).clicked() {
        choose_flat_models(app);
    }
    ui.add_space(8.0);
    text(ui, lang.text("可选外部发音词典（不设置则使用内置普通话 / 日语 / 英语词典）", "External pronunciation dictionaries (optional; Mandarin, Japanese and English are built in)", "外部発音辞書（任意、中国語・日本語・英語は内蔵）"));
    path_label(ui, app.synthesis.dictionaries(), lang);
    ui.horizontal(|ui| {
        if ui.button(lang.text("选择词典目录…", "Choose dictionary folder…", "辞書フォルダーを選択…")).clicked() {
            choose_dictionaries(app);
        }
        if app.synthesis.dictionaries().is_some()
            && ui.button(lang.text("恢复内置词典", "Use built-in dictionary", "内蔵辞書に戻す")).clicked()
        {
            app.synthesis.use_builtin_dictionaries();
        }
    });
    if let Some(error) = app.synthesis.library.settings_error() {
        error_label(ui, error);
    }
    weak(
        ui,
        lang.text("声库目录需要手动选择；普通话、日语和英语词典随程序提供，其他语言需外部词典或显式音素。", "Voicebank folders must be selected manually. Mandarin, Japanese and English dictionaries are built in; other languages need an external dictionary or explicit phonemes.", "歌声フォルダーは手動選択が必要です。中国語・日本語・英語の辞書は内蔵されています。他の言語は外部辞書か音素の指定が必要です。"),
    );
}

pub(super) fn choose_databases(app: &mut App) {
    if let Some(path) =
        folder_dialog(app.synthesis.library.settings.databases.as_deref()).pick_folder()
    {
        app.synthesis.set_databases(path);
    }
}

pub(super) fn choose_sv1_databases(app: &mut App) {
    if let Some(path) =
        folder_dialog(app.synthesis.library.settings.sv1_databases.as_deref()).pick_folder()
    {
        app.synthesis.set_sv1_databases(path);
    }
}

pub(super) fn choose_flat_databases(app: &mut App) {
    if let Some(path) =
        folder_dialog(app.synthesis.library.settings.flat_databases.as_deref()).pick_folder()
    {
        app.synthesis.set_flat_databases(path);
    }
}

pub(super) fn choose_flat_models(app: &mut App) {
    if let Some(path) =
        folder_dialog(app.synthesis.library.settings.flat_models.as_deref()).pick_folder()
    {
        app.synthesis.set_flat_models(path);
    }
}

pub(super) fn directory_menu(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    if ui.button(lang.text("SV2 声库目录…", "SV2 voicebank folder…", "SV2 歌声フォルダー…")).clicked() {
        ui.close();
        choose_databases(app);
    }
    if ui.button(lang.text("SV1 原生声库目录…", "Native SV1 folder…", "SV1 ネイティブフォルダー…")).clicked() {
        ui.close();
        choose_sv1_databases(app);
    }
    if ui.button(lang.text("SV Flat 声库目录…", "SV Flat voicebank folder…", "SV Flat 歌声フォルダー…")).clicked() {
        ui.close();
        choose_flat_databases(app);
    }
}

pub(super) fn choose_dictionaries(app: &mut App) {
    if let Some(path) = folder_dialog(app.synthesis.dictionaries()).pick_folder() {
        app.synthesis.set_dictionaries(path);
    }
}

fn path_label(ui: &mut Ui, path: Option<&Path>, lang: UiLanguage) {
    match path {
        Some(path) => weak(ui, path.display().to_string()),
        None => weak(ui, lang.text("未指定", "Not specified", "未指定")),
    }
}

fn folder_dialog(path: Option<&Path>) -> rfd::FileDialog {
    let dialog = rfd::FileDialog::new();
    match path {
        Some(path) => dialog.set_directory(path),
        None => dialog,
    }
}
