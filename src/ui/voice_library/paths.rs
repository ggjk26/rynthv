use std::path::Path;

use super::{error_label, text, weak};
use crate::app::App;
use eframe::egui::*;

pub fn settings(app: &mut App, ui: &mut Ui) {
    text(ui, RichText::new("歌声资源").strong());
    text(ui, "SV2 声库根目录（databases）");
    path_label(ui, app.synthesis.library.settings.databases.as_deref());
    ui.horizontal(|ui| {
        if ui.button("选择 SV2 databases…").clicked() {
            choose_databases(app);
        }
        if ui
            .add_enabled(
                !app.synthesis.library.is_scanning()
                    && app.synthesis.library.settings.has_databases(),
                Button::new("重新扫描"),
            )
            .clicked()
        {
            app.synthesis.rescan_voices();
        }
    });
    ui.add_space(8.0);
    text(ui, "SV1 原生声库根目录");
    path_label(ui, app.synthesis.library.settings.sv1_databases.as_deref());
    if ui.button("选择 SV1 databases…").clicked() {
        choose_sv1_databases(app);
    }
    ui.add_space(8.0);
    text(ui, "SV Flat 声库根目录（已解包的 NOFS-JSON 或 info.nofs）");
    path_label(ui, app.synthesis.library.settings.flat_databases.as_deref());
    if ui.button("选择 Flat databases…").clicked() {
        choose_flat_databases(app);
    }
    ui.add_space(8.0);
    text(ui, "Flat 模型根目录（可选，仅用于依赖扫描）");
    path_label(ui, app.synthesis.library.settings.flat_models.as_deref());
    if ui.button("选择 Flat models…").clicked() {
        choose_flat_models(app);
    }
    ui.add_space(8.0);
    text(ui, "可选外部发音词典（不设置则使用内置普通话 / 日语词典）");
    path_label(ui, app.synthesis.dictionaries());
    ui.horizontal(|ui| {
        if ui.button("选择词典目录…").clicked() {
            choose_dictionaries(app);
        }
        if app.synthesis.dictionaries().is_some()
            && ui.button("恢复内置词典").clicked()
        {
            app.synthesis.use_builtin_dictionaries();
        }
    });
    if let Some(error) = app.synthesis.library.settings_error() {
        error_label(ui, error);
    }
    weak(
        ui,
        "声库目录需要手动选择；普通话和日语词典随程序提供，其他语言需外部词典或显式音素。",
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
    if ui.button("SV2 声库目录…").clicked() {
        ui.close();
        choose_databases(app);
    }
    if ui.button("SV1 原生声库目录…").clicked() {
        ui.close();
        choose_sv1_databases(app);
    }
    if ui.button("SV Flat 声库目录…").clicked() {
        ui.close();
        choose_flat_databases(app);
    }
}

pub(super) fn choose_dictionaries(app: &mut App) {
    if let Some(path) = folder_dialog(app.synthesis.dictionaries()).pick_folder() {
        app.synthesis.set_dictionaries(path);
    }
}

fn path_label(ui: &mut Ui, path: Option<&Path>) {
    match path {
        Some(path) => weak(ui, path.display().to_string()),
        None => weak(ui, "未指定"),
    }
}

fn folder_dialog(path: Option<&Path>) -> rfd::FileDialog {
    let dialog = rfd::FileDialog::new();
    match path {
        Some(path) => dialog.set_directory(path),
        None => dialog,
    }
}
