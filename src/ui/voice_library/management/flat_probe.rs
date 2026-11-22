use crate::{
    app::App,
    synthesis::voice::{VoiceCatalogEntry, flat::probe::StageState},
    ui::settings::UiLanguage,
};
use eframe::egui::*;

pub(super) fn show(app: &mut App, ui: &mut Ui, entry: &VoiceCatalogEntry) {
    let lang = app.preferences.language;
    let library = &app.synthesis.library;
    ui.add_space(8.0);
    if library.flat_probe_path() == Some(entry.directory.as_path()) {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(lang.text(
                "正在校验此声库的 Flat 模型文件…",
                "Verifying Flat model files for this voice…",
                "この歌声の Flat モデルファイルを検証中…",
            ));
        });
    } else {
        let response = ui
            .add_enabled(
                !library.flat_probe_running(),
                Button::new(lang.text(
                    "验证 Flat DNNI 模型",
                    "Verify Flat DNNI models",
                    "Flat DNNI を検証",
                )),
            )
            .on_disabled_hover_text(lang.text(
                "等待当前后台模型校验结束",
                "Waiting for the current model check to finish",
                "現在のモデル検証が完了するまでお待ちください",
            ));
        if response.clicked()
            && let Err(error) = app.synthesis.inspect_flat_models(&entry.directory)
        {
            app.error = Some(error);
        }
    }
    let Some(probe) = app.synthesis.library.flat_probe_for(&entry.directory) else {
        return;
    };
    let report = match probe {
        Ok(report) => report,
        Err(error) => {
            super::super::error_label(ui, lang.editor_message(error));
            return;
        }
    };
    ui.label(format!(
        "{} {}/{}   ·   {} {}",
        lang.text("已定位模型", "Models located", "検出モデル"),
        report.located,
        report.stages.len(),
        lang.text(
            "匹配 SV2 阶段",
            "Matching SV2 stages",
            "一致する SV2 ステージ"
        ),
        report.native_resource_identified,
    ));
    for stage in &report.stages {
        let (label, color) = stage_status(&stage.state, lang);
        ui.collapsing(
            RichText::new(format!("{} · {label}", stage.role)).color(color),
            |ui| {
                ui.monospace(&stage.id);
                if let Some(path) = &stage.path {
                    ui.small(path.display().to_string());
                }
                ui.small(&stage.details);
            },
        );
    }
    if ui
        .button(lang.text(
            "复制完整模型预检报告",
            "Copy full model preflight report",
            "モデル検証レポートをコピー",
        ))
        .clicked()
        && let Ok(text) = serde_json::to_string_pretty(report)
    {
        ui.ctx().copy_text(text);
    }
}

fn stage_status(state: &StageState, lang: UiLanguage) -> (&'static str, Color32) {
    use crate::ui::theme::{GREEN, MUTED};
    let red = Color32::from_rgb(232, 143, 117);
    let amber = Color32::from_rgb(237, 191, 119);
    match state {
        StageState::Missing => (
            lang.text("缺少文件", "Missing file", "ファイルがありません"),
            MUTED,
        ),
        StageState::FileUnreadable => (lang.text("无法读取", "Unreadable", "読み込み不可"), red),
        StageState::UnsupportedFormat => (
            lang.text(
                "当前解析器不支持",
                "Unsupported by this reader",
                "現在のリーダーは非対応",
            ),
            amber,
        ),
        StageState::InvalidNativeContainer => (
            lang.text(
                "完整性校验未通过",
                "Integrity check did not pass",
                "整合性チェック未通過",
            ),
            amber,
        ),
        StageState::StageResourceNotFound => (
            lang.text(
                "容器已验证，未匹配阶段",
                "Container verified, stage unmatched",
                "コンテナー検証済み、ステージ不一致",
            ),
            amber,
        ),
        StageState::NativeResourceFound => (
            lang.text(
                "容器已验证，匹配 SV2 阶段",
                "Container verified, SV2 stage matched",
                "コンテナー検証済み、SV2 ステージ一致",
            ),
            GREEN,
        ),
    }
}
