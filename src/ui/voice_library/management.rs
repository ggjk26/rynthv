mod cards;
mod catalog;
mod details;
mod feedback;
mod flat_probe;
mod header;
mod toolbar;

use super::{context, error_label, paths, text, weak};
use crate::{
    app::App,
    ui::{child, theme::*},
};
use eframe::egui::*;
use std::{collections::BTreeMap, path::PathBuf};

const SURFACE: Color32 = Color32::from_rgb(27, 27, 27);
const CARD: Color32 = Color32::from_rgb(38, 38, 38);
const ACCENT: Color32 = Color32::from_rgb(120, 214, 24);
const INK: Color32 = Color32::from_rgb(242, 242, 242);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Sort {
    #[default]
    Default,
    Name,
    NameReverse,
    Version,
}
impl Sort {
    fn label(self, lang: crate::ui::settings::UiLanguage) -> &'static str {
        match self {
            Self::Default => lang.text("默认", "Default", "デフォルト"),
            Self::Name => lang.text("名称 A–Z", "Name A–Z", "名前 A–Z"),
            Self::NameReverse => lang.text("名称 Z–A", "Name Z–A", "名前 Z–A"),
            Self::Version => lang.text("版本最新", "Newest version", "新しいバージョン順"),
        }
    }
}
#[derive(Clone, Default)]
struct State {
    query: String,
    sort: Sort,
    language: String,
    vendor: String,
    versions: BTreeMap<String, u32>,
    expanded: Option<String>,
    root: Option<PathBuf>,
    sv1_root: Option<PathBuf>,
    flat_root: Option<PathBuf>,
    flat_models: Option<PathBuf>,
}

pub fn dialogs(app: &mut App, ctx: &Context) {
    let id = Id::new("voice-manager-state");
    if app.voice_manager {
        let mut state = ctx
            .data_mut(|data| data.get_temp::<State>(id))
            .unwrap_or_default();
        if state.root != app.synthesis.library.settings.databases
            || state.sv1_root != app.synthesis.library.settings.sv1_databases
            || state.flat_root != app.synthesis.library.settings.flat_databases
            || state.flat_models != app.synthesis.library.settings.flat_models
        {
            state = State {
                root: app.synthesis.library.settings.databases.clone(),
                sv1_root: app.synthesis.library.settings.sv1_databases.clone(),
                flat_root: app.synthesis.library.settings.flat_databases.clone(),
                flat_models: app.synthesis.library.settings.flat_models.clone(),
                ..Default::default()
            };
        }
        show(app, ctx, &mut state);
        ctx.data_mut(|data| data.insert_temp(id, state));
    } else {
        ctx.data_mut(|data| data.remove::<State>(id));
    }
    context::dialog(app, ctx);
}

fn show(app: &mut App, ctx: &Context, state: &mut State) {
    let size = (ctx.content_rect().size() - vec2(32.0, 40.0))
        .min(vec2(1120.0, 650.0))
        .max(vec2(260.0, 240.0));
    let mut close = false;
    let modal = Modal::new(Id::new("voice-manager"))
        .backdrop_color(Color32::from_black_alpha(105))
        .frame(
            Frame::new()
                .fill(SURFACE)
                .corner_radius(5)
                .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50))),
        )
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let head = Rect::from_min_size(rect.min, vec2(rect.width(), header::HEIGHT));
            close = header::show(app, ui, head);
            ui.painter().hline(
                head.x_range(),
                head.bottom(),
                Stroke::new(1.0, Color32::from_rgb(20, 20, 20)),
            );
            let margin = (rect.width() * 0.08).clamp(16.0, 90.0);
            let body = Rect::from_min_max(
                pos2(rect.left() + margin, head.bottom() + 26.0),
                rect.max - vec2(margin, 16.0),
            );
            let mut content = child(ui, body, "voice-store-content");
            ScrollArea::vertical()
                .id_salt("voice-store-scroll")
                .auto_shrink([false, false])
                .show(&mut content, |ui| {
                    ui.set_width(ui.available_width());
                    toolbar::show(app, ui, state);
                    ui.add_space(16.0);
                    if app.synthesis.library.is_scanning() {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            text(ui, app.preferences.language.text("正在扫描歌声…", "Scanning voicebanks…", "歌声データベースを検索中…"));
                        });
                        return;
                    }
                    if let Some(error) = app.synthesis.library.settings_error() {
                        error_label(ui, app.preferences.language.text("无法读取声库列表", "Cannot read voicebank list", "歌声一覧を読み込めません"));
                        ui.collapsing(app.preferences.language.text("查看原因", "Show details", "詳細を表示"), |ui| error_label(ui, error));
                        ui.menu_button(app.preferences.language.text("选择声库目录…", "Choose voice folder…", "歌声フォルダーを選択…"), |ui| {
                            paths::directory_menu(app, ui)
                        });
                        return;
                    }
                    if let Some(error) = app.synthesis.library.catalog_error() {
                        error_label(
                            ui,
                            if app
                                .synthesis
                                .library
                                .catalog()
                                .is_some_and(|catalog| !catalog.entries.is_empty())
                            {
                                app.preferences.language.text("部分声库目录无法读取", "Some voice folders could not be read", "一部の歌声フォルダーを読み込めません")
                            } else {
                                app.preferences.language.text("无法读取声库目录", "Unable to read voice folder", "歌声フォルダーを読み込めません")
                            },
                        );
                        ui.collapsing(app.preferences.language.text("查看原因", "Show details", "詳細を表示"), |ui| error_label(ui, error));
                        ui.menu_button(app.preferences.language.text("选择声库目录…", "Choose voice folder…", "歌声フォルダーを選択…"), |ui| {
                            paths::directory_menu(app, ui)
                        });
                        ui.add_space(12.0);
                    }
                    let Some(catalog) = app.synthesis.library.catalog_snapshot() else {
                        empty(app, ui, state);
                        return;
                    };
                    let lang = app.preferences.language;
                    feedback::scan_warnings(ui, &catalog, lang);
                    let current = app.voice_selection();
                    let entries = catalog::entries(&catalog, state, current.as_ref(), lang);
                    if entries.is_empty() {
                        empty(app, ui, state);
                    } else {
                        ui.weak(format!(
                            "{} {}",
                            entries.len(),
                            lang.text("个声库", "voicebanks", "件の歌声")
                        ));
                        ui.spacing_mut().item_spacing.y = 16.0;
                        for entry in entries {
                            cards::show(
                                app,
                                ui,
                                entry,
                                state,
                                catalog::is_selected(entry, current.as_ref()),
                            );
                        }
                    }
                });
        });
    if close || modal.should_close() {
        app.voice_manager = false;
    }
}

fn empty(app: &mut App, ui: &mut Ui, state: &mut State) {
    Frame::new()
        .fill(CARD)
        .corner_radius(13)
        .inner_margin(28)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(100.0);
            if !state.query.is_empty() || !state.language.is_empty() || !state.vendor.is_empty() {
                text(
                    ui,
                    RichText::new(app.preferences.language.text("没有匹配的歌声", "No matching voices", "一致する歌声がありません"))
                        .font(section_title_font())
                        .size(16.0),
                );
                weak(ui, app.preferences.language.text("换一个关键词，或清除筛选条件。", "Try a different search or clear your filters.", "検索語を変えるか、絞り込みを解除してください。"));
                if ui.button(app.preferences.language.text("清除搜索与筛选", "Clear search and filters", "検索と絞り込みをクリア")).clicked() {
                    state.query.clear();
                    state.language.clear();
                    state.vendor.clear();
                }
            } else {
                text(
                    ui,
                    RichText::new(app.preferences.language.text("没有可显示的声库", "No voicebanks found", "歌声データベースが見つかりません"))
                        .font(section_title_font())
                        .size(16.0),
                );
                weak(ui, app.preferences.language.text("选择本机 databases 目录，扫描已安装的歌声。", "Choose a local databases directory to scan installed voices.", "ローカルの databases フォルダーを指定してインストール済みの歌声を検索します。"));
                ui.menu_button(app.preferences.language.text("选择声库目录…", "Choose voice folder…", "歌声フォルダーを選択…"), |ui| paths::directory_menu(app, ui));
            }
        });
}
