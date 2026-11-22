use std::path::Path;

use super::{error_label, text, weak};
use crate::{app::App, synthesis::settings::{VoiceDirectory, DurationModelSlot}, ui::settings::UiLanguage};
use eframe::egui::*;

pub fn settings(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    text(ui, RichText::new(lang.text("歌声资源", "Voice resources", "歌声リソース")).strong());
    if app.synthesis.library.invalid_settings() {
        if let Some(error) = app.synthesis.library.settings_error() {
            error_label(ui, error);
        }
        weak(ui, lang.text(
            "声库设置文件无法读取。为了避免覆盖原有路径，已暂停保存声库设置。可备份原文件并重新选择目录。",
            "Voice resource settings could not be read. Saving is disabled to protect the original paths. Back up the file and select new folders to recover.",
            "歌声設定ファイルを読み込めません。元のパスを保護するため設定保存を停止しました。バックアップ後にフォルダーを再選択できます。",
        ));
        if ui.button(lang.text(
            "备份损坏配置并重置声库路径",
            "Back up invalid settings and reset voice paths",
            "破損した設定をバックアップして歌声パスを初期化",
        )).clicked() {
            match app.synthesis.recover_invalid_voice_settings() {
                Ok(backup) => app.status = format!(
                    "{} {}",
                    lang.text("原配置已备份：", "Original settings backed up:", "元の設定をバックアップしました："),
                    backup.display(),
                ),
                Err(error) => app.error = Some(error),
            }
        }
        return;
    }
    if ui.button(lang.text(
        "检测并应用本机 SV1 / SV2 / Flat 声库目录",
        "Detect and apply installed SV1 / SV2 / Flat folders",
        "インストール済み SV1 / SV2 / Flat フォルダーを検出して適用",
    )).clicked() {
        match app.synthesis.use_installed_voice_roots() {
            Ok(count) => app.status = format!(
                "{} ({count})",
                lang.text("已设置声库目录并重新扫描", "Voicebank roots applied and rescanned", "歌声フォルダーを設定して再スキャンしました"),
            ),
            Err(error) => app.error = Some(error),
        }
    }
    weak(ui, lang.text(
        "从 Documents 检查三个独立安装位置；仅应用存在的目录，并清除旧版 roots 遗留路径。",
        "Checks the three separate installation locations under Documents, applies only existing folders, and clears stale legacy roots.",
        "Documents 内の３つの独立したインストール先を調べ、存在するフォルダーだけを設定し旧 roots を消去します。",
    ));
    voice_root_editor(app, ui, VoiceDirectory::Sv2,
        lang.text("SV2 声库根目录（Studio 2 / databases）",
            "SV2 root (Studio 2 / databases)",
            "SV2 フォルダー（Studio 2 / databases）"));
    if ui
        .add_enabled(
            !app.synthesis.library.is_scanning()
                && app.synthesis.library.settings.has_databases(),
            Button::new(lang.text("重新扫描所有声库", "Rescan all voices", "すべての歌声を再スキャン")),
        )
        .clicked()
    {
        app.synthesis.rescan_voices();
    }
    // Show imported roots from older synthesis.json layouts so the paths
    // actually being searched remain visible to the user.
    if app.synthesis.library.settings.roots.is_some() {
        let legacy_paths: Vec<_> = app.synthesis.library.settings.database_roots()
            .filter(|(label, _)| *label == "Legacy")
            .map(|(_, path)| path.to_path_buf())
            .collect();
        if !legacy_paths.is_empty() {
            ui.add_space(8.0);
            text(ui, lang.text(
                "从旧版配置 roots 读取的声库目录",
                "Voicebank folders loaded from legacy roots",
                "旧設定 roots から読み込んだ歌声フォルダー",
            ));
            for path in &legacy_paths {
                path_label(ui, Some(path), lang);
            }
        }
    }
    ui.add_space(10.0);
    voice_root_editor(app, ui, VoiceDirectory::Sv1,
        lang.text("SV1 原生声库根目录（Studio / databases）",
            "Native SV1 root (Studio / databases)",
            "SV1 フォルダー（Studio / databases）"));
    ui.add_space(10.0);
    voice_root_editor(app, ui, VoiceDirectory::Flat,
        lang.text("SV Flat 声库根目录（OPSV / databases）",
            "SV Flat root (OPSV / databases)",
            "SV Flat フォルダー（OPSV / databases）"));
    ui.add_space(10.0);
    voice_root_editor(app, ui, VoiceDirectory::FlatModels,
        lang.text("Flat 模型目录（可选，仅依赖扫描）",
            "Flat models (optional, dependencies only)",
            "Flat モデル（任意、依存関係のみ）"));
    ui.add_space(12.0);
    text(ui, RichText::new(lang.text(
        "时长模型（.dnni）",
        "Duration models (.dnni)",
        "Duration / Timing モデル (.dnni)",
    )).strong());
    weak(ui, lang.text(
        "SV2 Duration 用于声库缺少独立 Duration 阶段时的合成；Flat Timing 仅用于依赖检查，不能直接替代 SV2 Duration。",
        "SV2 Duration can fill a missing SV2 synthesis stage. Flat Timing is metadata-only and cannot replace SV2 Duration.",
        "SV2 Duration は SV2 歌声の不足モデルを補います。Flat Timing は依存関係の検査専用で SV2 Duration の代わりにはなりません。",
    ));
    duration_model_editor(app, ui, DurationModelSlot::Sv2,
        lang.text("SV2 时长模型 DNNI（可选）",
            "SV2 Duration DNNI (optional)",
            "SV2 Duration DNNI（任意）"));
    ui.add_space(8.0);
    duration_model_editor(app, ui, DurationModelSlot::Flat,
        lang.text("Flat Timing Model DNNI（仅检查）",
            "Flat Timing DNNI (inspection only)",
            "Flat Timing DNNI（検査のみ）"));
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
        lang.text("也可直接粘贴路径并点击应用。SV1、SV2 与 Flat 分别使用独立的 databases 目录；保存后立即重新扫描。", "Paste a path and click Apply, or browse. SV1, SV2 and Flat use separate databases directories; saving rescans automatically.", "パスを貼り付けて適用、または参照できます。SV1、SV2、Flat は別々の databases フォルダーを使用し、保存後に再スキャンします。"),
    );
}

/// A model file is a single DNNI archive, not a database folder.
fn duration_model_editor(
    app: &mut App,
    ui: &mut Ui,
    slot: DurationModelSlot,
    title: &str,
) {
    let lang = app.preferences.language;
    let (key, path) = match slot {
        DurationModelSlot::Sv2 =>
            ("sv2-duration", app.synthesis.library.settings.sv2_duration_model.clone()),
        DurationModelSlot::Flat =>
            ("flat-timing", app.synthesis.library.settings.flat_timing_model.clone()),
    };
    let id = Id::new(("duration-file-editor", key));
    let source = path.as_ref().map_or_else(String::new, |file| file.to_string_lossy().into_owned());
    let mut draft = ui.ctx().data_mut(|data| {
        let old = data.get_temp::<String>(id.with("source"));
        if old.as_deref() != Some(source.as_str()) {
            data.insert_temp(id.with("source"), source.clone());
            data.insert_temp(id.with("draft"), source.clone());
        }
        data.get_temp::<String>(id.with("draft")).unwrap_or(source.clone())
    });
    ui.label(RichText::new(title).strong());
    let mut apply = false;
    let mut browse = false;
    let mut clear = false;
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 170.0).max(130.0);
        let response = ui.add(
            TextEdit::singleline(&mut draft)
                .id(id.with("edit"))
                .desired_width(width)
                .hint_text(lang.text(
                    "输入或粘贴 .dnni 文件路径",
                    "Paste the .dnni file path",
                    ".dnni ファイルパスを入力・貼り付け",
                )),
        );
        apply = ui.button(lang.text("应用", "Apply", "適用")).clicked()
            || (response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)));
        browse = ui.button(lang.text("选择文件…", "Choose file…", "ファイルを選択…")).clicked();
        clear = ui.add_enabled(path.is_some(), Button::new(
            lang.text("清除", "Clear", "解除")
        )).clicked();
    });
    ui.ctx().data_mut(|data| data.insert_temp(id.with("draft"), draft.clone()));
    if apply {
        let clean = draft.trim().trim_matches('"').trim();
        let value = if clean.is_empty() { None }
            else { Some(std::path::PathBuf::from(clean)) };
        apply_duration(app, slot, value);
    } else if browse {
        let root = path.as_deref().and_then(Path::parent);
        if let Some(file) = folder_dialog(root)
            .add_filter("DNNI", &["dnni"])
            .pick_file()
        {
            apply_duration(app, slot, Some(file));
        }
    } else if clear {
        apply_duration(app, slot, None);
    }
    if path.is_some_and(|file| !file.is_file()) {
        ui.colored_label(
            Color32::from_rgb(230, 150, 116),
            lang.text(
                "已保存的 DNNI 文件不存在",
                "Saved DNNI file was not found",
                "保存済みの DNNI ファイルが存在しません",
            ),
        );
    }
}

fn apply_duration(
    app: &mut App,
    slot: DurationModelSlot,
    path: Option<std::path::PathBuf>,
) {
    match app.synthesis.set_duration_file(slot, path) {
        Ok(()) => {
            app.status = app.preferences.language.text(
                "时长模型已保存，正在重新检查声库",
                "Duration model saved; rescanning voices",
                "Duration モデルを保存し、歌声を再検査中",
            ).to_owned();
        }
        Err(error) => app.error = Some(error),
    }
}

/// Dedicated editable path field: typing and pasting work even when the OS
/// folder picker cannot be opened from a nested voice-manager popup.
fn voice_root_editor(
    app: &mut App,
    ui: &mut Ui,
    kind: VoiceDirectory,
    title: &str,
) {
    let lang = app.preferences.language;
    let (key, path) = match kind {
        VoiceDirectory::Sv2 => ("sv2", app.synthesis.library.settings.databases.clone()),
        VoiceDirectory::Sv1 => ("sv1", app.synthesis.library.settings.sv1_databases.clone()),
        VoiceDirectory::Flat => ("flat", app.synthesis.library.settings.flat_databases.clone()),
        VoiceDirectory::FlatModels => ("flat-models", app.synthesis.library.settings.flat_models.clone()),
    };
    let id = Id::new(("voice-root-edit", key));
    let source = path.as_ref().map_or_else(String::new, |path| path.to_string_lossy().into_owned());
    let mut draft = ui.ctx().data_mut(|data| {
        let old = data.get_temp::<String>(id.with("source"));
        if old.as_deref() != Some(source.as_str()) {
            data.insert_temp(id.with("source"), source.clone());
            data.insert_temp(id.with("draft"), source.clone());
        }
        data.get_temp::<String>(id.with("draft")).unwrap_or(source.clone())
    });
    ui.label(RichText::new(title).strong());
    let mut apply = false;
    let mut browse = false;
    let mut clear = false;
    ui.horizontal(|ui| {
        let available = (ui.available_width() - 170.0).max(130.0);
        let response = ui.add(
            TextEdit::singleline(&mut draft)
                .id(id.with("edit"))
                .desired_width(available)
                .hint_text(lang.text("输入或粘贴 databases 文件夹路径",
                    "Enter or paste the databases folder path",
                    "databases のフォルダーパスを入力・貼り付け")),
        );
        apply = ui.button(lang.text("应用", "Apply", "適用")).clicked()
            || (response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)));
        browse = ui.button(lang.text("浏览…", "Browse…", "参照…")).clicked();
        clear = ui.add_enabled(path.is_some(),
            Button::new(lang.text("清除", "Clear", "解除"))).clicked();
    });
    ui.ctx().data_mut(|data| data.insert_temp(id.with("draft"), draft.clone()));
    if apply {
        let cleaned = draft.trim().trim_matches('"').trim();
        let path = if cleaned.is_empty() { None } else { Some(std::path::PathBuf::from(cleaned)) };
        apply_root(app, kind, path);
    } else if browse {
        pick_root(app, kind);
    } else if clear {
        apply_root(app, kind, None);
    }
    if let Some(path) = path {
        if !path.is_dir() {
            ui.colored_label(
                Color32::from_rgb(230, 150, 116),
                lang.text("已保存的目录不存在或不可访问",
                    "Saved directory does not exist or is not accessible",
                    "保存済みのフォルダーが存在しないかアクセスできません"),
            );
        }
    }
}

fn apply_root(app: &mut App, kind: VoiceDirectory, path: Option<std::path::PathBuf>) {
    match app.synthesis.set_voice_directory(kind, path) {
        Ok(()) => {
            app.status = app.preferences.language.text(
                "声库目录已保存，正在重新扫描",
                "Voice directory saved; rescanning",
                "歌声フォルダーを保存し、再スキャン中",
            ).to_owned();
        }
        Err(error) => app.error = Some(error),
    }
}

fn pick_root(app: &mut App, kind: VoiceDirectory) {
    let path = match kind {
        VoiceDirectory::Sv2 => app.synthesis.library.settings.databases.as_deref(),
        VoiceDirectory::Sv1 => app.synthesis.library.settings.sv1_databases.as_deref(),
        VoiceDirectory::Flat => app.synthesis.library.settings.flat_databases.as_deref(),
        VoiceDirectory::FlatModels => app.synthesis.library.settings.flat_models.as_deref(),
    };
    if let Some(path) = folder_dialog(path).pick_folder() {
        apply_root(app, kind, Some(path));
    }
}

pub(super) fn choose_databases(app: &mut App) {
    pick_root(app, VoiceDirectory::Sv2);
}

pub(super) fn choose_sv1_databases(app: &mut App) {
    pick_root(app, VoiceDirectory::Sv1);
}

pub(super) fn choose_flat_databases(app: &mut App) {
    pick_root(app, VoiceDirectory::Flat);
}

pub(super) fn choose_flat_models(app: &mut App) {
    pick_root(app, VoiceDirectory::FlatModels);
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
