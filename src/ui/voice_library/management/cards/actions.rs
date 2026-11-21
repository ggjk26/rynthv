use super::super::{ACCENT, CARD, SURFACE, State, catalog};
use crate::{
    app::App,
    synthesis::{
        runtime::VoiceState,
        voice::{VoiceCatalogEntry, VoiceFamily},
    },
    ui::theme::*,
};
use eframe::egui::*;

pub(super) fn primary(
    app: &mut App,
    ui: &mut Ui,
    entry: &VoiceCatalogEntry,
    state: &mut State,
    current: bool,
    stable: bool,
) {
    let lang = app.preferences.language;
    let key = entry.selection();
    let voice = app.synthesis.voice_state(&key);
    let loading = matches!(voice, VoiceState::Loading);
    let needs_context =
        entry.family != Some(VoiceFamily::Sv1) && app.synthesis.needs_context_import(&key);
    let problem =
        matches!(voice, VoiceState::Loaded(_)) && app.synthesis.voice_error(&key).is_some();
    let loaded = matches!(voice, VoiceState::Loaded(_)) && !problem;
    let label = if !entry.availability.is_loadable() {
        lang.text("不可用", "Unavailable", "利用不可")
    } else if current && loading {
        lang.text("正在加载…", "Loading…", "読み込み中…")
    } else if needs_context {
        lang.text("导入模型上下文…", "Import model context…", "モデルコンテキストをインポート…")
    } else if problem {
        lang.text("处理问题", "View issues", "問題を確認")
    } else if current && loaded {
        lang.text("取消当前音轨歌声", "Remove track voice", "トラックの歌声を解除")
    } else if loaded {
        lang.text("用于当前音轨", "Use for this track", "このトラックで使用")
    } else if current && matches!(voice, VoiceState::Failed(_)) {
        lang.text("重试加载", "Retry loading", "再読み込み")
    } else {
        lang.text("加载歌声", "Load voice", "歌声を読み込む")
    };
    let enabled = stable
        && entry.availability.is_loadable()
        && !(current && loading)
        && !app.editor.current_track().is_instrumental()
        && !app.synthesis.library.is_scanning()
        && app.synthesis.library.settings_error().is_none();
    let outline = loaded || problem;
    let button = Button::new(RichText::new(label).size(13.0).color(if !enabled {
        MUTED
    } else if outline {
        ACCENT
    } else {
        SURFACE
    }))
    .corner_radius(13)
    .fill(if !enabled || outline { CARD } else { ACCENT })
    .stroke(Stroke::new(0.8, if enabled { ACCENT } else { BORDER }));
    if ui
        .add_enabled(enabled, button.min_size(vec2(ui.available_width(), 36.0)))
        .clicked()
    {
        if needs_context {
            app.synthesis.open_context_import(key);
        } else if problem {
            state.expanded = Some(entry.database_id.clone());
        } else if current && loaded {
            app.unload_voice();
        } else if current {
            app.reload_voice();
        } else {
            app.select_voice(catalog::selection(entry));
        }
    }
}

pub(super) fn menu(
    app: &mut App,
    ui: &mut Ui,
    entry: &VoiceCatalogEntry,
    state: &mut State,
    current: bool,
) {
    let lang = app.preferences.language;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 13, BORDER);
    }
    for offset in [-5.0, 0.0, 5.0] {
        ui.painter()
            .circle_filled(rect.center() + vec2(0.0, offset), 1.5, MUTED);
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, lang.text("更多声库操作", "More voice actions", "その他の歌声操作")));
    Popup::menu(&response).show(|ui| {
        if ui
            .button(if state.expanded.as_ref() == Some(&entry.database_id) {
                lang.text("收起详情", "Hide details", "詳細を閉じる")
            } else {
                lang.text("详情与诊断", "Details & diagnostics", "詳細と診断")
            })
            .clicked()
        {
            if state.expanded.as_ref() == Some(&entry.database_id) {
                state.expanded = None;
            } else {
                state.expanded = Some(entry.database_id.clone());
            }
            ui.close();
        }
        if entry.family.is_some_and(VoiceFamily::requires_identity)
            && ui
                .add_enabled(
                    app.synthesis.can_import_context(&entry.selection()),
                    Button::new(lang.text("重新导入上下文…", "Reimport context…", "コンテキストを再インポート…")),
                )
                .on_hover_text(lang.text("重新选择上下文来源，导入成功后重新加载使用此上下文的歌声", "Select a new source. Voices using the context will reload after import.", "ソースを選び直します。インポート後、このコンテキストを使う歌声を再読み込みします。"))
                .on_disabled_hover_text(lang.text("当前声库不可用，或正在扫描声库、导入上下文", "Voice is unavailable, or scanning/import is in progress", "歌声が利用できないか、スキャン・インポート中です"))
                .clicked()
        {
            app.synthesis.open_context_import(entry.selection());
            ui.close();
        }
        if !matches!(
            app.synthesis.voice_state(&entry.selection()),
            VoiceState::Unloaded
        ) {
            if ui
                .add_enabled(
                    !matches!(
                        app.synthesis.voice_state(&entry.selection()),
                        VoiceState::Loading
                    ),
                    Button::new(lang.text("重新加载", "Reload", "再読み込み")),
                )
                .clicked()
            {
                app.synthesis.reload_voice(&entry.selection());
                ui.close();
            }
            if current && ui.button(lang.text("取消当前音轨歌声", "Remove track voice", "トラックの歌声を解除")).clicked() {
                app.unload_voice();
                ui.close();
            }
        }
    });
}
