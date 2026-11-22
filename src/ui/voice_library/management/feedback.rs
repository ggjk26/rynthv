use super::super::{context, error_label, localized_diagnostic, text, weak};
use crate::{
    app::App,
    synthesis::{
        runtime::VoiceState,
        voice::{VoiceCatalog, VoiceCatalogEntry},
    },
    ui::theme::*,
};
use eframe::egui::*;
pub(super) fn status(
    app: &mut App,
    ui: &mut Ui,
    selection: &crate::synthesis::settings::VoiceSelection,
) {
    let lang = app.preferences.language;
    match app.synthesis.voice_state(selection) {
        VoiceState::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                weak(ui, lang.text("正在准备歌声，请稍候…", "Preparing voice…", "歌声を準備中…"));
            });
        }
        VoiceState::Loaded(_) if app.synthesis.voice_error(selection).is_none() => {
            text(ui, RichText::new(lang.text("已加载", "Loaded", "読み込み完了")).color(GREEN));
            weak(ui, lang.text("编辑音符后自动合成，按空格键试听。", "Synthesis updates after editing notes. Press Space to preview.", "ノート編集後に自動合成します。Space キーで試聴できます。"));
        }
        _ => {}
    }
    let error = app.synthesis.voice_error(selection).map(str::to_owned);
    if let Some(error) = error {
        error_label(
            ui,
            if matches!(app.synthesis.voice_state(selection), VoiceState::Failed(_)) {
                lang.text("歌声加载失败", "Voice loading failed", "歌声の読み込みに失敗しました")
            } else {
                lang.text("合成环境准备失败", "Synthesis backend initialization failed", "合成環境の準備に失敗しました")
            },
        );
        ui.collapsing(lang.text("查看错误详情", "Show error details", "エラーの詳細を表示"), |ui| error_label(ui, localized_diagnostic(&error, lang)));
        if app
            .synthesis
            .inventory(selection)
            .is_some_and(|voice| voice.configuration.is_some())
        {
            context::controls(app, ui, selection);
        }
    }
}

pub(super) fn diagnostics(app: &App, ui: &mut Ui, entry: &VoiceCatalogEntry) {
    let lang = app.preferences.language;
    weak(ui, lang.text("声库目录", "Voice directory", "歌声フォルダー"));
    text(ui, entry.directory.display().to_string());
    if let Some(voice) = app.synthesis.inventory(&entry.selection()) {
        weak(
            ui,
            format!(
                "{} {} · {} {} · {} {} · {} {}",
                voice.files.len(), lang.text("个文件", "files", "ファイル"),
                voice.resource_count, lang.text("个资源", "resources", "リソース"),
                voice.graph_count, lang.text("棵模块树", "module trees", "モジュールツリー"),
                voice.tensor_count, lang.text("个张量", "tensors", "テンソル"),

            ),
        );
        if let Some(prepared) = voice.preparation() {
            let mut affine = 0;
            let mut unary = 0;
            let mut other = 0;
            for tree in prepared.summary().files.iter().flat_map(|file| &file.trees) {
                affine += tree.modules.affine_nodes;
                unary += tree.modules.unary_nodes;
                other += tree.modules.unsupported_nodes;
            }
            weak(
                ui,
                format!("{} {affine} {}, {unary} {}; {other} {}", lang.text("已装配", "Prepared", "構築済み"), lang.text("个线性层", "affine layers", "線形層"), lang.text("个逐元素模块", "elementwise modules", "要素ごとのモジュール"), lang.text("个模块由阶段执行器处理或尚未装配", "modules handled by stage executors or not yet prepared", "個のモジュールはステージ実行器が処理、または未構築です")),
            );
        }
        if let Some(elapsed) = app.synthesis.load_elapsed(&entry.selection()) {
            weak(ui, format!("{} {:.2} {}", lang.text("加载耗时", "Load time", "読み込み時間"), elapsed.as_secs_f64(), lang.text("秒", "s", "秒")));
        }
    }
}

pub(super) fn scan_warnings(
    ui: &mut Ui,
    catalog: &VoiceCatalog,
    lang: crate::ui::settings::UiLanguage,
) {
    if !catalog.warnings.is_empty() {
        ui.collapsing(
            format!("{} ({})", lang.text("扫描提示", "Scan warnings", "スキャンの警告"), catalog.warnings.len()),
            |ui| {
                for warning in &catalog.warnings {
                    weak(ui, localized_diagnostic(warning, lang));
                }
            },
        );
    }
}
