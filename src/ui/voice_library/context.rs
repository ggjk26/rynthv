mod fields;

use super::{error_label, text, weak};
use crate::{
    app::App,
    synthesis::{
        runtime::{ContextImportPhase, ContextImportSource},
        voice::session_identity,
    },
};
use eframe::egui::*;

pub(super) fn controls(
    app: &mut App,
    ui: &mut Ui,
    selection: &crate::synthesis::settings::VoiceSelection,
) {
    let lang = app.preferences.language;
    if !app
        .synthesis
        .library
        .entry(selection)
        .and_then(|entry| entry.family)
        .is_some_and(|family| family.requires_identity())
    {
        return;
    }
    if let Some(result) = &app.synthesis.context_import.feedback {
        match result {
            Ok(message) => text(ui, message),
            Err(error) => error_label(ui, error),
        }
    }
    let label = if app.synthesis.needs_context_import(selection) {
        lang.text("导入模型上下文…", "Import model context…", "モデルコンテキストをインポート…")
    } else {
        lang.text("重新导入上下文…", "Reimport context…", "コンテキストを再インポート…")
    };
    if ui
        .add_enabled(
            app.synthesis.can_import_context(selection),
            Button::new(label),
        )
        .on_hover_text(
            lang.text("从本地会话文件、上下文文件、手动输入或原编辑器进程重新导入此歌声的模型上下文", "Import voice model context from a session file, JSON file, manual value or a running editor.", "セッションファイル、JSON、手動入力、または起動中のエディターからモデルコンテキストをインポートします。"),
        )
        .on_disabled_hover_text(lang.text("当前声库不可用，或正在扫描声库、导入上下文", "Voice unavailable, or scan/import in progress", "歌声が利用できないか、スキャン・インポート中です"))
        .clicked()
    {
        app.synthesis.open_context_import(selection.clone());
    }
}

pub(super) fn dialog(app: &mut App, ctx: &Context) {
    let lang = app.preferences.language;
    let Some(dialog) = &mut app.synthesis.context_import.dialog else {
        return;
    };
    let mut refresh = false;
    let mut import = false;
    let mut cancel = false;
    let mut edited = false;
    let mut source = dialog.source;
    let importing = dialog.phase == ContextImportPhase::Importing;
    let modal = Modal::new(Id::new("voice-model-context-import")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.spacing_mut().item_spacing.y = 8.0;
        text(ui, RichText::new(lang.text("导入模型上下文", "Import model context", "モデルコンテキストのインポート")).strong().size(16.0));
        text(ui, format!("{}: {}", lang.text("当前声库", "Current voice", "現在の歌声"), dialog.voice_name));
        weak(
            ui,
            if dialog.model_version == 1 {
                lang.text("版本 1 兼容歌声", "Version 1-compatible voice", "バージョン1互換歌声")
            } else {
                lang.text("版本 2 歌声", "Version 2 voice", "バージョン2の歌声")
            },
        );
        ui.add_enabled_ui(!importing, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(session_identity::supports_session_import(), |ui| {
                    ui.selectable_value(
                        &mut source,
                        ContextImportSource::SessionFile,
                        lang.text("本地会话文件", "Local session file", "ローカルセッションファイル"),
                    )
                    .on_disabled_hover_text(
                        lang.text("导入本地会话文件仅支持 Windows x64；当前平台可导入上下文文件或手动输入", "Session-file import is available only on Windows x64. Use a context file or manual input on this platform.", "セッションファイルのインポートは Windows x64 のみ対応します。この環境ではコンテキストファイルか手動入力を使用してください。"),
                    );
                });
                ui.add_enabled_ui(session_identity::supports_online_import(), |ui| {
                    ui.selectable_value(&mut source, ContextImportSource::Online, lang.text("从原版获取", "Read from original editor", "元のエディターから取得"))
                        .on_disabled_hover_text(
                            lang.text("从原版获取仅支持 Windows x64；当前平台可导入上下文文件或手动输入", "Reading from original editor is only supported on Windows x64; use a context file or manual input here.", "元のエディターからの取得は Windows x64 のみ対応します。コンテキストファイルか手動入力を使用してください。"),
                        );
                });
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut source, ContextImportSource::File, lang.text("上下文文件", "Context file", "コンテキストファイル"));
                ui.selectable_value(&mut source, ContextImportSource::Manual, lang.text("手动输入", "Manual input", "手動入力"));
            });
        });
        ui.separator();
        ui.add_enabled_ui(!importing, |ui| {
            edited = fields::show(ui, dialog, lang);
        });
        if edited {
            dialog.error = None;
        }
        if importing {
            ui.horizontal(|ui| {
                ui.spinner();
                text(ui, lang.text("正在导入，完成后自动重新加载此歌声…", "Importing; this voice will reload afterward…", "インポート中です。完了後に歌声を再読み込みします…"));
            });
        } else {
            weak(ui, lang.text("仅更新此歌声的上下文，完成后自动重新加载。", "Only this voice's context will be changed. The voice will reload after import.", "この歌声のコンテキストのみ更新し、完了後に再読み込みします。"));
        }
        if let Some(error) = &dialog.error {
            error_label(ui, error);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if dialog.source == ContextImportSource::Online {
                refresh = ui
                    .add_enabled(
                        dialog.phase == ContextImportPhase::Ready,
                        Button::new(lang.text("刷新进程", "Refresh processes", "プロセスを更新")),
                    )
                    .clicked();
            }
            cancel = ui.add_enabled(!importing, Button::new(lang.text("取消", "Cancel", "キャンセル"))).clicked();
            import = ui
                .add_enabled(dialog.can_import(), Button::new(lang.text("导入", "Import", "インポート")))
                .clicked();
        });
    });
    cancel |= !importing
        && modal.is_top_modal
        && !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
    let source_changed = source != dialog.source;
    if edited {
        app.synthesis.clear_context_import_error();
    }
    if cancel {
        app.synthesis.cancel_context_import();
    } else if source_changed {
        app.synthesis.set_context_import_source(source);
    } else if import {
        app.synthesis.import_context();
    } else if refresh {
        app.synthesis.refresh_context_sources();
    }
}
