pub(crate) mod edit;
use super::state::Column;
use crate::{
    app::{App, FileAction, MidiImportMode},
    model::*,
};
use eframe::egui::*;

fn item(ui: &mut Ui, text: &str, shortcut: &str, enabled: bool) -> bool {
    let response = ui.add_enabled(
        enabled,
        Button::new(text)
            .shortcut_text(shortcut)
            .min_size(vec2(180.0, 22.0)),
    );
    if response.clicked() {
        ui.close();
        true
    } else {
        false
    }
}
pub fn menus(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let ctx = ui.ctx().clone();
    MenuBar::new().ui(ui, |ui| {
        ui.spacing_mut().button_padding = vec2(11.0, 4.0);
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.menu_button(lang.text("文件", "File", "ファイル"), |ui| {
            if item(ui, lang.text("新建", "New", "新規"), "Ctrl+N", true) {
                app.request_file(FileAction::New, &ctx);
            }
            if item(ui, lang.text("打开…", "Open…", "開く…"), "Ctrl+O", true) {
                app.request_file(FileAction::Open, &ctx);
            }
            ui.separator();
            if item(ui, lang.text("保存", "Save", "保存"), "Ctrl+S", true) {
                app.save_project(false);
            }
            if item(ui, lang.text("另存为…", "Save as…", "名前を付けて保存…"), "Ctrl+Shift+S", true) {
                app.save_project(true);
            }
            ui.separator();
            if item(ui, lang.text("导入…", "Import…", "インポート…"), "Ctrl+Shift+I", true) {
                app.request_file(FileAction::ImportMidi, &ctx);
            }
            if item(ui, lang.text("作为音轨导入…", "Import as tracks…", "トラックとしてインポート…"), "", true) {
                app.begin_midi_import(MidiImportMode::Tracks, &ctx);
            }
            if item(ui, lang.text("导入音频…", "Import audio…", "オーディオをインポート…"), "", true) {
                app.import_audio(None, &ctx);
            }
            item(ui, lang.text("导出音频", "Export audio", "オーディオを書き出す"), "", false);
        });
        ui.menu_button(lang.text("编辑", "Edit", "編集"), |ui| {
            if item(
                ui,
                &format!("{} {}", lang.text("撤销", "Undo", "元に戻す"), app.editor.undo_label()),
                "Ctrl+Z",
                app.editor.can_undo(),
            ) {
                app.editor.undo();
            }
            if item(ui, lang.text("重做", "Redo", "やり直す"), "Ctrl+Y", app.editor.can_redo()) {
                app.editor.redo();
            }
            ui.separator();
            if item(ui, lang.text("全选", "Select all", "すべて選択"), "Ctrl+A", true) {
                edit::select_all(app);
            }
            if item(ui, lang.text("取消选择", "Deselect", "選択解除"), "Esc", true) {
                edit::clear(app);
            }
            let selected = edit::selected(app);
            if item(ui, lang.text("删除所选内容", "Delete selection", "選択項目を削除"), "Delete", selected) {
                edit::delete(app);
            }
            ui.separator();
            if item(ui, lang.text("复制", "Copy", "コピー"), "Ctrl+C", selected) {
                edit::copy(app, &ctx);
            }
            if item(ui, lang.text("剪切", "Cut", "切り取り"), "Ctrl+X", selected) {
                edit::cut(app, &ctx);
            }
            if item(ui, lang.text("粘贴", "Paste", "貼り付け"), "Ctrl+V", edit::can_paste(app)) {
                edit::paste(app, &ctx);
            }
        });
        ui.menu_button(lang.text("查看", "View", "表示"), |ui| {
            ui.checkbox(&mut app.show_time_tracks, lang.text("显示曲速和拍号轨", "Show tempo and meter tracks", "テンポ・拍子トラックを表示"));
            if ui
                .checkbox(&mut app.view.arrangement_collapsed, "折叠编排")
                .clicked()
            {
                ui.close();
            }
            ui.checkbox(&mut app.view.arrangement_first, lang.text("编排位于上方", "Arrangement above", "アレンジを上に配置"));
            ui.separator();
            for column in Column::PANELS {
                let mut active = app.view.columns.contains(&column);
                if ui.checkbox(&mut active, column.localized_label(lang)).clicked() {
                    app.view.toggle(column);
                }
            }
            ui.separator();
            if item(ui, lang.text("设置", "Settings", "設定"), "", true) {
                app.settings = true;
            }
        });
        ui.menu_button(lang.text("修改", "Modify", "変更"), |ui| {
            let selected = !super::lanes::parameter_active(app)
                && !super::phonemes::phoneme_active(app)
                && app.piano_tool != super::piano::Tool::PitchAnchor
                && !app.editor.selected.is_empty();
            if item(ui, lang.text("填入歌词…", "Fill lyrics…", "歌詞を入力…"), "Ctrl+L", selected) {
                super::fill_lyrics::open(app);
            }
            if item(ui, lang.text("静音音符", "Mute notes", "ノートをミュート"), "Ctrl+M", selected) {
                app.editor.mute_selected();
            }
            ui.separator();
            if item(ui, lang.text("升高八度", "Octave up", "1オクターブ上げる"), "Ctrl+↑", edit::can_transpose(app)) {
                edit::transpose(app, true);
            }
            if item(ui, lang.text("降低八度", "Octave down", "1オクターブ下げる"), "Ctrl+↓", edit::can_transpose(app)) {
                edit::transpose(app, false);
            }
            item(ui, lang.text("智能量化", "Smart quantize", "スマートクオンタイズ"), "", false);
            item(ui, lang.text("解除组链接", "Unlink group", "グループのリンクを解除"), "", false);
        });
        ui.menu_button(lang.text("自动处理", "Automation", "自動処理"), |ui| {
            item(ui, lang.text("装饰所选音符", "Decorate selected notes", "選択ノートを装飾"), "", false);
            ui.add_enabled_ui(
                !super::lanes::parameter_active(app) && !super::phonemes::phoneme_active(app),
                |ui| {
                    super::retakes::commands(app, ui);
                },
            );
            item(ui, lang.text("从音频提取音符", "Extract notes from audio", "音声からノートを抽出"), "", false);
            item(ui, lang.text("检测曲速", "Detect tempo", "テンポを検出"), "", false);
        });
        ui.menu_button(lang.text("项目", "Project", "プロジェクト"), |ui| {
            if item(ui, lang.text("创建音轨", "Add track", "トラックを追加"), "", true) {
                app.editor.add_track();
            }
            if item(
                ui,
                lang.text("在走带处插入新组", "Insert group at playhead", "再生位置にグループを挿入"),
                "",
                !app.editor.current_track().is_instrumental(),
            ) {
                app.editor.add_group();
            }
            if item(ui, lang.text("创建伴奏轨", "Add instrumental track", "伴奏トラックを追加"), "", true) {
                let id = app.editor.add_instrumental_track();
                app.import_audio(Some(id), &ctx);
            }
            item(ui, lang.text("设置歌声", "Set voice", "歌声を設定"), "", false);
        });
        ui.menu_button(lang.text("走带控制", "Transport", "再生操作"), |ui| {
            if item(ui, lang.text("播放 / 暂停", "Play / Pause", "再生／一時停止"), "Space", true) {
                app.editor.transport.toggle();
            }
            if item(ui, lang.text("停止", "Stop", "停止"), "Ctrl+Space", true) {
                app.editor.transport.stop();
            }
            ui.checkbox(&mut app.editor.transport.looping, lang.text("循环播放", "Loop playback", "ループ再生"));
            if item(ui, lang.text("跳到开头", "Go to start", "先頭へ移動"), "", true) {
                app.editor.transport.position = 0;
            }
            if item(ui, lang.text("跳到结尾", "Go to end", "末尾へ移動"), "Ctrl+End", true) {
                app.editor.transport.position = app.editor.project.end();
            }
        });
        ui.menu_button(lang.text("脚本", "Scripts", "スクリプト"), |ui| {
            item(ui, lang.text("重新扫描", "Rescan", "再スキャン"), "", false);
            item(ui, lang.text("打开脚本文件夹", "Open scripts folder", "スクリプトフォルダーを開く"), "", false);
        });
        ui.menu_button(lang.text("其他", "Other", "その他"), |ui| {
            if item(ui, lang.text("关于 RynthV", "About RynthV", "RynthV について"), "", true) {
                app.about = true;
            }
        });
    });
}
pub fn shortcuts(app: &mut App, ctx: &Context) {
    if super::phonemes::phoneme_dragging(app) {
        if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            super::phonemes::phoneme_cancel(app, ctx);
        }
        return;
    }
    if app.editor.has_performance_edit() || app.editor.has_voice_edit() {
        return;
    }
    if app.audio_drag.is_some() {
        return;
    }
    if app.view_pan.is_active() {
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            app.view_pan.cancel(ctx);
        }
        return;
    }
    if !ctx.text_edit_focused()
        && app.lyric.is_none()
        && app.gesture.is_none()
        && app.group_gesture.is_none()
        && edit::clipboard_events(app, ctx)
    {
        return;
    }
    if ctx.egui_wants_keyboard_input() || app.lyric.is_some() {
        return;
    }
    let command = |key, modifiers| ctx.input_mut(|i| i.consume_key(modifiers, key));
    let ctrl = Modifiers::CTRL;
    let shift = Modifiers::CTRL | Modifiers::SHIFT;
    if matches!(app.gesture, Some(super::piano::Gesture::Anchor(_)))
        && super::piano::anchor_keyboard(app, ctx)
    {
        return;
    }
    if app.gesture.is_some() || app.group_gesture.is_some() {
        if command(Key::Escape, Modifiers::NONE) {
            if matches!(app.gesture, Some(super::piano::Gesture::Parameter(_))) {
                super::lanes::parameter_cancel(app, ctx);
                return;
            }
            app.editor.cancel();
            app.gesture = None;
            app.group_gesture = None;
        }
        return;
    }
    if command(Key::S, shift) {
        app.save_project(true);
    } else if command(Key::S, ctrl) {
        app.save_project(false);
    }
    if command(Key::I, shift) {
        app.request_file(FileAction::ImportMidi, ctx);
        return;
    }
    if command(Key::N, ctrl) {
        app.request_file(FileAction::New, ctx);
    }
    if command(Key::O, ctrl) {
        app.request_file(FileAction::Open, ctx);
    }
    if command(Key::Z, shift) || command(Key::Y, ctrl) {
        app.editor.redo();
    } else if command(Key::Z, ctrl) {
        app.editor.undo();
    }
    if super::lanes::parameter_keyboard(app, ctx) {
        return;
    }
    if !super::lanes::parameter_active(app)
        && !super::phonemes::phoneme_active(app)
        && super::piano::anchor_keyboard(app, ctx)
    {
        return;
    }
    if !super::lanes::parameter_active(app)
        && !super::phonemes::phoneme_active(app)
        && command(Key::T, Modifiers::ALT | Modifiers::SHIFT)
    {
        super::retakes::prune(app);
        return;
    } else if !super::lanes::parameter_active(app)
        && !super::phonemes::phoneme_active(app)
        && command(Key::T, Modifiers::ALT)
    {
        super::retakes::create(app, RetakeComponent::All);
        return;
    }
    if command(Key::A, ctrl) {
        edit::select_all(app);
    }
    if command(Key::D, ctrl) {
        edit::duplicate(app);
    }
    if !super::lanes::parameter_active(app)
        && !super::phonemes::phoneme_active(app)
        && app.piano_tool != super::piano::Tool::PitchAnchor
        && command(Key::L, ctrl)
    {
        super::fill_lyrics::open(app);
        return;
    }
    if !super::lanes::parameter_active(app)
        && !super::phonemes::phoneme_active(app)
        && app.piano_tool != super::piano::Tool::PitchAnchor
        && command(Key::M, ctrl)
    {
        app.editor.mute_selected();
    }
    if command(Key::B, ctrl) {
        app.view.toggle(Column::Note);
    }
    if command(Key::ArrowUp, ctrl) {
        edit::transpose(app, true);
    }
    if command(Key::ArrowDown, ctrl) {
        edit::transpose(app, false);
    }
    if command(Key::Delete, Modifiers::NONE) {
        edit::delete(app);
    }
    if command(Key::Escape, Modifiers::NONE) {
        edit::clear(app);
    }
    if command(Key::Num1, Modifiers::ALT) {
        app.piano_tool = super::piano::Tool::NoteSelect;
        app.status.clear();
    }
    if command(Key::Num2, Modifiers::ALT) {
        app.piano_tool = super::piano::Tool::NoteDraw;
        app.status.clear();
    }
    if command(Key::Num3, Modifiers::ALT) {
        app.commit_lyric();
        app.editor.selected.clear();
        app.piano_tool = super::piano::Tool::PitchAnchor;
        app.show_pitch = true;
        app.status.clear();
    }
    if command(Key::Num4, Modifiers::ALT) {
        app.commit_lyric();
        app.piano_tool = super::piano::Tool::PitchDraw;
        app.show_pitch = true;
        app.status.clear();
    }
    if command(Key::Space, ctrl) {
        app.editor.transport.stop();
    } else if command(Key::Space, Modifiers::NONE) {
        app.editor.transport.toggle();
    }
    if command(Key::End, ctrl) {
        app.editor.transport.position = app.editor.project.end();
    }
    for (key, direction) in [(Key::ArrowLeft, -1), (Key::ArrowRight, 1)] {
        if command(key, Modifiers::NONE) {
            app.editor.transport.position =
                (app.editor.transport.position + direction * ticks(app.view.step())).max(0);
        }
    }
}
