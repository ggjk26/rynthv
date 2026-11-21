use super::{super::language_name, CARD, INK, Sort, State};
use crate::{
    app::App,
    ui::{
        child,
        icons::{Icon, IconButton},
        theme::*,
    },
};
use eframe::egui::*;
use std::collections::BTreeSet;

pub(super) fn show(app: &mut App, ui: &mut Ui, state: &mut State) {
    ui.spacing_mut().interact_size.y = 36.0;
    ui.visuals_mut().widgets.inactive.bg_fill = CARD;
    ui.visuals_mut().widgets.inactive.weak_bg_fill = CARD;
    ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::same(12);
    let wide = ui.available_width() >= 670.0;
    if wide {
        ui.horizontal(|ui| {
            search(ui, &mut state.query, 230.0, app.preferences.language);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                actions(app, ui, state)
            });
        });
    } else {
        search(ui, &mut state.query, ui.available_width(), app.preferences.language);
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| actions(app, ui, state));
    }
}

fn search(ui: &mut Ui, query: &mut String, width: f32, lang: crate::ui::settings::UiLanguage) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 40.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 5, Color32::from_rgb(43, 43, 43));
    let center = rect.left_center() + vec2(19.0, -1.0);
    ui.painter()
        .circle_stroke(center, 5.5, Stroke::new(1.8, TEXT));
    ui.painter().line_segment(
        [center + vec2(4.0, 4.0), center + vec2(9.0, 9.0)],
        Stroke::new(1.8, TEXT),
    );
    let has_text = !query.is_empty();
    let mut input = child(
        ui,
        Rect::from_min_max(
            rect.min + vec2(35.0, 0.0),
            rect.max - vec2(if has_text { 30.0 } else { 8.0 }, 0.0),
        ),
        ui.id().with("voice-search"),
    );
    input.add_sized(
        input.max_rect().size(),
        TextEdit::singleline(query)
            .frame(Frame::NONE)
            .font(FontId::proportional(14.0))
            .hint_text(lang.text("搜索歌声", "Search voices", "歌声を検索"))
            .vertical_align(Align::Center)
            .margin(vec2(0.0, 9.0)),
    );
    if has_text {
        let mut clear = child(
            ui,
            Rect::from_center_size(rect.right_center() - vec2(16.0, 0.0), Vec2::splat(24.0)),
            "clear-voice-search",
        );
        if clear
            .add(
                IconButton::new(Icon::Close, lang.text("清除搜索", "Clear search", "検索をクリア"))
                    .size(Vec2::splat(24.0))
                    .icon_size(12.0),
            )
            .clicked()
        {
            query.clear();
        }
    }
}

fn filters(app: &App, ui: &mut Ui, state: &mut State) {
    let lang = app.preferences.language;
    let count = usize::from(!state.language.is_empty()) + usize::from(!state.vendor.is_empty());
    let filter = ui.add(
        Button::new(if count == 0 {
            lang.text("筛选", "Filter", "絞り込み").into()
        } else {
            format!("{} · {count}", lang.text("筛选", "Filter", "絞り込み"))
        })
        .min_size(vec2(76.0, 36.0))
        .corner_radius(14),
    );
    Popup::from_toggle_button_response(&filter)
        .width(240.0)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                let mut languages = BTreeSet::new();
                let mut vendors = BTreeSet::new();
                if let Some(catalog) = app.synthesis.library.catalog() {
                    for entry in &catalog.entries {
                        languages.extend(entry.languages.iter().cloned());
                        if !entry.vendor.is_empty() {
                            vendors.insert(entry.vendor.clone());
                        }
                    }
                }
                ui.label(RichText::new(lang.text("筛选歌声", "Filter voices", "歌声を絞り込む")).strong().color(INK));
                ui.weak(lang.text("声库语言", "Voice language", "歌声の言語"));
                ComboBox::from_id_salt("manager-language-filter")
                    .width(ui.available_width())
                    .selected_text(if state.language.is_empty() {
                        lang.text("全部语言", "All languages", "すべての言語")
                    } else {
                        language_name(&state.language, lang)
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.language, String::new(), lang.text("全部语言", "All languages", "すべての言語"));
                        for language in languages {
                            let label = language_name(&language, lang).to_owned();
                            ui.selectable_value(&mut state.language, language, label);
                        }
                    });
                ui.weak(lang.text("厂商", "Vendor", "メーカー"));
                ComboBox::from_id_salt("manager-vendor-filter")
                    .width(ui.available_width())
                    .selected_text(if state.vendor.is_empty() {
                        lang.text("全部厂商", "All vendors", "すべてのメーカー")
                    } else {
                        &state.vendor
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.vendor, String::new(), lang.text("全部厂商", "All vendors", "すべてのメーカー"));
                        for vendor in vendors {
                            ui.selectable_value(&mut state.vendor, vendor.clone(), vendor);
                        }
                    });
                if ui.button(lang.text("清除筛选", "Clear filters", "絞り込みを解除")).clicked() {
                    state.language.clear();
                    state.vendor.clear();
                }
            });
        });
    if count > 0 {
        filter.on_hover_text(lang.text("已启用筛选", "Filters active", "絞り込み中")).highlight();
    }
}

fn actions(app: &mut App, ui: &mut Ui, state: &mut State) {
    let reverse = ui.layout().main_dir() == Direction::RightToLeft;
    if reverse {
        filters(app, ui, state);
        sorting(ui, state, true, app.preferences.language);
        refresh(app, ui);
    } else {
        refresh(app, ui);
        sorting(ui, state, false, app.preferences.language);
        filters(app, ui, state);
    }
}

fn sorting(ui: &mut Ui, state: &mut State, reverse: bool, lang: crate::ui::settings::UiLanguage) {
    if !reverse {
        ui.label(RichText::new(lang.text("排序方式", "Sort by", "並び替え")).color(MUTED));
    }
    ComboBox::from_id_salt("voice-manager-sort")
        .width(90.0)
        .selected_text(state.sort.label(lang))
        .show_ui(ui, |ui| {
            for sort in [Sort::Default, Sort::Name, Sort::NameReverse, Sort::Version] {
                ui.selectable_value(&mut state.sort, sort, sort.label(lang));
            }
        });
    if reverse {
        ui.label(RichText::new(lang.text("排序方式", "Sort by", "並び替え")).color(MUTED));
    }
}

fn refresh(app: &mut App, ui: &mut Ui) {
    if ui
        .add_enabled(
            !app.synthesis.library.is_scanning() && app.synthesis.library.settings.has_databases(),
            Button::new(app.preferences.language.text("刷新", "Refresh", "更新")).min_size(vec2(52.0, 36.0)),
        )
        .clicked()
    {
        app.synthesis.rescan_voices();
    }
}
