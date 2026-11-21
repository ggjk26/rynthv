use std::collections::BTreeMap;

use super::{artwork, error_label, family_name, selected_entry, weak};
use crate::{
    app::App,
    synthesis::{
        settings::VoiceSelection,
        voice::{VoiceAvailability, VoiceCatalogEntry, VoiceFamily},
    },
    ui::{icons::Icon, theme::*},
};
use eframe::egui::*;

const POPUP_FILL: Color32 = Color32::from_rgb(61, 59, 64);

pub(super) fn selected_card(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let entry = selected_entry(app);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 60.0), Sense::click());
    ui.painter().rect(
        rect,
        4,
        if response.hovered() { BUTTON } else { BORDER },
        Stroke::new(0.7, BUTTON),
        StrokeKind::Inside,
    );
    let art = Rect::from_min_size(rect.min, Vec2::splat(60.0));
    artwork::paint(ui, art, entry.and_then(|entry| entry.icon_path.as_deref()));
    let label_rect = Rect::from_min_max(
        pos2(art.right() + 8.0, rect.top()),
        rect.max - vec2(26.0, 0.0),
    );
    ui.painter()
        .with_clip_rect(label_rect.intersect(ui.clip_rect()))
        .text(
            label_rect.left_center() - vec2(0.0, if entry.is_some() { 9.0 } else { 0.0 }),
            Align2::LEFT_CENTER,
            entry.map_or_else(
                || app.voice_name().unwrap_or(lang.text("选择歌声", "Select voice", "歌声を選択")),
                |entry| entry.display_name.as_str(),
            ),
            FontId::proportional(13.0),
            TEXT,
        );
    if let Some(entry) = entry {
        ui.painter()
            .with_clip_rect(label_rect.intersect(ui.clip_rect()))
            .text(
                label_rect.left_center() + vec2(0.0, 12.0),
                Align2::LEFT_CENTER,
                family_name(entry),
                FontId::proportional(10.5),
                MUTED,
            );
    }
    Icon::ChevronDown.paint(
        ui,
        Rect::from_center_size(rect.right_center() - vec2(16.0, 0.0), Vec2::splat(12.0)),
        MUTED,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, lang.text("选择歌声", "Select voice", "歌声を選択")));
    popup(app, &response.on_hover_cursor(CursorIcon::Default));
}

/// Shared by the track header and the voice panel; choices apply to this track.
pub fn popup(app: &mut App, anchor: &Response) {
    let lang = app.preferences.language;
    Popup::menu(anchor)
        .width(288.0)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .frame(
            Frame::popup(&anchor.ctx.style_of(Theme::Dark))
                .fill(POPUP_FILL)
                .stroke(Stroke::new(0.8, BUTTON_ACTIVE))
                .corner_radius(5)
                .inner_margin(4),
        )
        .show(|ui| {
            ui.set_width(280.0);
            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);
            header(ui, lang);
            separator(ui);
            if menu_row(ui, lang.text("管理歌声", "Manage voices", "歌声を管理"), Some(Icon::Cloud), true).clicked() {
                app.voice_manager = true;
                ui.close();
            }
            separator(ui);
            if menu_row(ui, lang.text("取消当前音轨歌声", "Remove track voice", "トラックの歌声を解除"), None, app.has_voice()).clicked() {
                app.unload_voice();
                ui.close();
            }
            separator(ui);
            if app.synthesis.library.is_scanning() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    weak(ui, lang.text("正在扫描歌声…", "Scanning voices…", "歌声をスキャン中…"));
                });
            }
            let entries = representative_entries(app);
            if app.synthesis.library.catalog_error().is_some() {
                error_label(
                    ui,
                    if entries.is_empty() {
                        lang.text("无法读取声库目录，请在“管理歌声”中查看原因。", "Cannot read voice directories; check Manage voices for details.", "歌声フォルダーを読み込めません。歌声管理で詳細を確認してください。")
                    } else {
                        lang.text("部分声库目录无法读取，已列出可读取的歌声。", "Some voice directories could not be read; available voices are listed.", "一部の歌声フォルダーを読み込めませんでした。利用可能な歌声のみ表示しています。")
                    },
                );
            }
            if entries.is_empty() && !app.synthesis.library.is_scanning() {
                weak(ui, lang.text("没有可选歌声，请在“管理歌声”中设置 databases 目录。", "No available voices. Set the databases directory in Manage voices.", "利用できる歌声がありません。歌声管理で databases フォルダーを設定してください。"));
            }
            let selected = app.voice_selection();
            let mut choose = None;
            ScrollArea::vertical()
                .id_salt("voice-picker-list")
                .max_height((ui.ctx().content_rect().height() - 180.0).clamp(80.0, 440.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for category in 0..4 {
                        let banks: Vec<_> = entries
                            .iter()
                            .filter(|entry| category_of(entry) == category)
                            .collect();
                        if banks.is_empty() {
                            continue;
                        }
                        if category > 0 {
                            separator(ui);
                            let title = match category {
                                1 => lang.text("SV2 · 版本 1 兼容歌声", "SV2 · Version 1 compatible voices", "SV2 · バージョン1互換歌声"),
                                2 => lang.text("SV1 原生歌声", "Native SV1 voices", "SV1 ネイティブ歌声"),
                                _ => lang.text("其他歌声 · 暂不可用", "Other voices · unavailable", "その他の歌声 · 利用不可"),
                            };
                            menu_row(ui, title, None, false);
                        }
                        for entry in banks {
                            let selection = VoiceSelection {
                                database_id: entry.database_id.clone(),
                                revision: entry.revision,
                            };
                            if card(ui, entry, selected.as_ref() == Some(&selection), lang).clicked() {
                                choose = Some(selection);
                            }
                        }
                    }
                });
            if let Some(selection) = choose {
                app.select_voice(selection);
                ui.close();
            }
        });
}

// A bank appears once; the panel's version selector exposes its revisions.
fn representative_entries(app: &App) -> Vec<&VoiceCatalogEntry> {
    let mut banks: BTreeMap<&str, &VoiceCatalogEntry> = BTreeMap::new();
    let selected = app.voice_selection();
    let rank = |entry: &VoiceCatalogEntry| {
        (
            selected.as_ref().is_some_and(|s| {
                s.database_id == entry.database_id && s.revision == entry.revision
            }),
            entry.availability.is_loadable(),
            entry.revision,
        )
    };
    if let Some(catalog) = app.synthesis.library.catalog() {
        for entry in &catalog.entries {
            let current = banks.entry(&entry.database_id).or_insert(entry);
            if rank(entry) > rank(current) {
                *current = entry;
            }
        }
    }
    let mut entries: Vec<_> = banks.into_values().collect();
    entries.sort_by(|a, b| {
        a.display_name
            .cmp(&b.display_name)
            .then_with(|| a.database_id.cmp(&b.database_id))
    });
    entries
}

fn category_of(entry: &VoiceCatalogEntry) -> u8 {
    match entry.family {
        Some(VoiceFamily::Sv1) => 2,
        Some(VoiceFamily::Sv2Compatible) => 1,
        Some(VoiceFamily::Sv2) if entry.availability.is_loadable() => 0,
        _ => 3,
    }
}

fn header(ui: &mut Ui, lang: crate::ui::settings::UiLanguage) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::hover());
    ui.painter().text(
        rect.left_center() + vec2(8.0, 0.0),
        Align2::LEFT_CENTER,
        lang.text("歌声", "Voices", "歌声"),
        section_title_font(),
        TEXT,
    );
    let origin = rect.right_center() - vec2(24.0, 7.0);
    for row in 0..3 {
        let p = origin + vec2(0.0, row as f32 * 5.0);
        ui.painter()
            .rect_filled(Rect::from_min_size(p, vec2(5.0, 4.0)), 0, ICON_TOOL);
        ui.painter().rect_filled(
            Rect::from_min_size(p + vec2(7.0, 0.0), vec2(9.0, 2.0)),
            0,
            ICON_TOOL,
        );
    }
}

fn menu_row(ui: &mut Ui, label: &str, icon: Option<Icon>, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 32.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 3, BUTTON_HOVER);
    }
    let mut x = rect.left() + 8.0;
    if let Some(icon) = icon {
        icon.paint(
            ui,
            Rect::from_center_size(pos2(x + 10.0, rect.center().y), Vec2::splat(20.0)),
            TEXT,
        );
        x += 28.0;
    }
    ui.painter().text(
        pos2(x, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        if enabled { TEXT } else { MUTED },
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    response.on_hover_cursor(CursorIcon::Default)
}

fn separator(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, BUTTON_ACTIVE),
    );
}

fn card(ui: &mut Ui, entry: &VoiceCatalogEntry, selected: bool, lang: crate::ui::settings::UiLanguage) -> Response {
    let enabled = entry.availability.is_loadable();
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 76.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if selected || (enabled && response.hovered()) {
        ui.painter()
            .rect_filled(rect, 5, if selected { BUTTON_HOVER } else { BUTTON });
    }
    let art = Rect::from_min_size(rect.min + vec2(8.0, 6.0), Vec2::splat(64.0));
    artwork::paint(ui, art, entry.icon_path.as_deref());
    let content = Rect::from_min_max(
        pos2(art.right() + 12.0, rect.top() + 7.0),
        rect.max - vec2(6.0, 4.0),
    );
    let painter = ui
        .painter()
        .with_clip_rect(content.intersect(ui.clip_rect()));
    painter.text(
        content.min,
        Align2::LEFT_TOP,
        &entry.display_name,
        section_title_font(),
        if enabled { TEXT } else { MUTED },
    );
    painter.text(
        content.min + vec2(0.0, 23.0),
        Align2::LEFT_TOP,
        match &entry.availability {
            VoiceAvailability::Loadable => entry.genres.join(", "),
            VoiceAvailability::Unsupported(reason) => reason.clone(),
        },
        FontId::proportional(12.0),
        MUTED,
    );
    let mut x = content.left();
    let mut tags = Vec::new();
    if let Some(range) = &entry.pitch_range {
        tags.push(range.as_str());
    }
    tags.extend(entry.languages.iter().map(|s| language_tag(s)));
    for tag in tags {
        let galley = painter.layout_no_wrap(tag.to_owned(), FontId::proportional(10.5), TEXT);
        let badge = Rect::from_min_size(
            pos2(x, content.top() + 44.0),
            vec2(galley.size().x + 8.0, 18.0),
        );
        painter.rect_filled(badge, 2, BUTTON_ACTIVE);
        painter.galley(
            badge.left_center() + vec2(4.0, -galley.size().y / 2.0),
            galley,
            TEXT,
        );
        x = badge.right() + 4.0;
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, &entry.display_name));
    response
        .on_hover_cursor(CursorIcon::Default)
        .on_hover_text(match &entry.availability {
            VoiceAvailability::Loadable => format!(
                "{} · {} · {} {}\n{}",
                lang.text("版本", "Version", "バージョン"),
                entry.display_name,
                family_name(entry),
                entry.revision,
                entry.vendor
            ),
            VoiceAvailability::Unsupported(reason) => format!("{}\n{reason}", entry.display_name),
        })
}

fn language_tag(language: &str) -> &str {
    match language {
        "mandarin" | "mandarin-xsampa" | "zh" => "ZH",
        "english" | "english-arpabet" | "en" => "EN",
        "japanese" | "japanese-romaji" | "ja" => "JA",
        "spanish" | "spanish-xsampa" | "es" => "ES",
        "cantonese" | "cantonese-xsampa" | "yue" => "YUE",
        "korean" | "korean-xsampa" | "ko" => "KO",
        other => other,
    }
}
