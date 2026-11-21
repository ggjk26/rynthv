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
    text(ui, format!("{}: {}", lang.text("声库类型", "Voice type", "歌声タイプ"), family_name(entry)));
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
                    .map(|s| language_name(s))
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
                    .map(|s| language_name(s))
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
