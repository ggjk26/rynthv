use super::{
    super::{context, family_name, language_name, text, weak},
    feedback,
};
use crate::{
    app::App,
    synthesis::voice::{VoiceAvailability, VoiceCatalogEntry},
};
use eframe::egui::*;

pub(super) fn show(app: &mut App, ui: &mut Ui, entry: &VoiceCatalogEntry) {
    let lang = app.preferences.language;
    ui.spacing_mut().item_spacing.y = 10.0;
    let selection = entry.selection();
    text(ui, format!("{}: {}", lang.text("声库类型", "Voice type", "歌声タイプ"), family_name(entry, lang)));
    if entry.database_id.starts_with("svflat:") {
        weak(
            ui,
            lang.text(
                "已识别 SV Flat 声库元数据。可检查模型依赖，但原生 Flat 合成与导出尚未实现。",
                "SV Flat metadata is recognized. Model dependencies can be inspected, but native Flat synthesis and export are not implemented.",
                "SV Flat のメタデータを認識しました。モデルの依存関係は確認できますが、Flat の歌声合成と書き出しは未対応です。",
            ),
        );
    }
    if let Some(range) = &entry.pitch_range {
        weak(ui, format!("{}  {range}", lang.text("推荐音域", "Recommended range", "推奨音域")));
    }
    if let Some(config) = app
        .synthesis
        .inventory(&selection)
        .and_then(|voice| voice.configuration.as_ref())
    {
        text(
            ui,
            format!(
                "{}: {}",
                lang.text("支持语言", "Supported languages", "対応言語"),
                config
                    .supported_languages
                    .iter()
                    .map(|s| language_name(s, lang))
                    .collect::<Vec<_>>()
                    .join("、")
            ),
        );
        if !config.styles.is_empty() {
            text(ui, format!("{}: {}", lang.text("唱法", "Vocal styles", "歌唱スタイル"), config.styles.join(" · ")));
        }
    } else if !entry.languages.is_empty() {
        text(
            ui,
            format!(
                "{}: {}",
                lang.text("语言", "Languages", "言語"),
                entry
                    .languages
                    .iter()
                    .map(|s| language_name(s, lang))
                    .collect::<Vec<_>>()
                    .join("、")
            ),
        );
    }
    if !entry.styles.is_empty() {
        text(ui, format!("{}: {}", lang.text("可识别唱法", "Recognized styles", "検出されたスタイル"), entry.styles.join(" · ")));
    }
    if !entry.model_dependencies.is_empty() {
        text(ui, lang.text("模型依赖（仅文件识别，不代表能够合成）：", "Model dependencies (file detection only; synthesis is not guaranteed):", "モデル依存関係（ファイル検出のみで、合成可能とは限りません）："));
        for dependency in &entry.model_dependencies {
            weak(ui, dependency);
        }
    }
    if let VoiceAvailability::Unsupported(reason) = &entry.availability {
        super::super::error_label(ui, reason);
    }
    if entry.database_id.starts_with("svflat:") {
        // A concise, copyable report makes mismatched Flat model installs
        // diagnosable without reading or exporting protected model payloads.
        let mut report = format!(
            "RynthV SV Flat diagnostics\nName: {}\nVersion: {}\nID: {}\nSource: {}\n",
            entry.display_name,
            entry.version_label.as_deref().unwrap_or("-"),
            entry.database_id,
            entry.directory.display(),
        );
        for dependency in &entry.model_dependencies {
            report.push_str("Model: ");
            report.push_str(dependency);
            report.push('\n');
        }
        if let VoiceAvailability::Unsupported(reason) = &entry.availability {
            report.push_str("Synthesis: unsupported\nDetails: ");
            report.push_str(reason);
        }
        if ui
            .button(lang.text(
                "复制 SV Flat 诊断报告",
                "Copy SV Flat diagnostic report",
                "SV Flat 診断レポートをコピー",
            ))
            .clicked()
        {
            ui.ctx().copy_text(report);
        }
    }
    if app.synthesis.inventory(&selection).is_some()
        || app.synthesis.voice_error(&selection).is_some()
    {
        feedback::status(app, ui, &selection);
        if app.synthesis.voice_error(&selection).is_none() {
            context::controls(app, ui, &selection);
        }
    }
    feedback::diagnostics(app, ui, entry);
}
