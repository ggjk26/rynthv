mod artwork;
mod context;
mod language;
mod management;
mod panel;
mod parameters;
mod paths;
mod selector;
mod widgets;

use super::theme::*;
use crate::{
    app::App,
    synthesis::voice::{VoiceCatalogEntry, VoiceFamily},
};
use eframe::egui::*;

pub use management::dialogs;
pub use panel::show;
pub(crate) use parameters::end_frame as end_parameter_frame;
pub use paths::settings;
pub use selector::popup;

fn selected_entry(app: &App) -> Option<&VoiceCatalogEntry> {
    app.synthesis.library.entry(&app.voice_selection()?)
}

fn family_name(entry: &VoiceCatalogEntry, lang: crate::ui::settings::UiLanguage) -> &'static str {
    if entry.database_id.starts_with("svflat:") {
        // Flat JSON voices and extracted SFPK binary packages have different
        // resource layouts. The catalog keeps both metadata-only for now.
        return if matches!(&entry.availability,
            crate::synthesis::voice::VoiceAvailability::Unsupported(reason)
                if reason.starts_with("Flat SFPK")
                    || reason.starts_with("Flat config.json")
        ) {
            "SV Flat · SFPK"
        } else {
            "SV Flat · NOFS-JSON"
        };
    }
    match entry.family {
        Some(VoiceFamily::Sv1) => lang.text("SV1 原生", "Native SV1", "SV1 ネイティブ"),
        Some(VoiceFamily::Sv2Compatible) => lang.text("SV2 · 版本 1 兼容", "SV2 · version 1 compatible", "SV2 · バージョン1互換"),
        Some(VoiceFamily::Sv2) => "SV2",
        None => lang.text("未知格式", "Unknown format", "不明な形式"),
    }
}

fn text(ui: &mut Ui, value: impl Into<WidgetText>) {
    ui.add(Label::new(value).selectable(false).wrap());
}

fn weak(ui: &mut Ui, value: impl Into<String>) {
    text(ui, RichText::new(value).color(MUTED));
}

fn error_label(ui: &mut Ui, value: impl Into<String>) {
    text(
        ui,
        RichText::new(value).color(Color32::from_rgb(218, 142, 126)),
    );
}

fn language_name(language: &str, lang: crate::ui::settings::UiLanguage) -> &str {
    match language {
        "mandarin" | "mandarin-xsampa" | "zh" | "ZH" => lang.text("普通话", "Mandarin", "中国語（普通話）"),
        "english" | "english-arpabet" | "en" | "EN" => lang.text("英语", "English", "英語"),
        "japanese" | "japanese-romaji" | "ja" | "JA" => lang.text("日语", "Japanese", "日本語"),
        "spanish" | "spanish-xsampa" | "es" | "ES" => lang.text("西班牙语", "Spanish", "スペイン語"),
        "cantonese" | "cantonese-xsampa" | "yue" | "YUE" => lang.text("粤语", "Cantonese", "広東語"),
        "korean" | "korean-xsampa" | "ko" | "KO" => lang.text("韩语", "Korean", "韓国語"),
        other => other,
    }
}
