pub(crate) mod edit;
use super::state::Column;
use crate::{
    app::{App, FileAction, MidiImportMode},
    model::*,
};
use eframe::egui::*;


/// Interpret historical undo labels in the current interface language even
/// when the user switches locales after performing an edit.
fn localized_undo_label<'a>(lang: crate::ui::settings::UiLanguage, label: &'a str) -> &'a str {
    match label {
        "删除音符" | "Delete notes" | "ノートを削除" => lang.text("删除音符", "Delete notes", "ノートを削除"),
        "移调" | "Transpose notes" | "ノートを移調" => lang.text("移调", "Transpose notes", "ノートを移調"),
        "静音音符" | "Mute notes" | "ノートをミュート" => lang.text("静音音符", "Mute notes", "ノートをミュート"),
        "创建音轨" | "Create track" | "トラックを作成" => lang.text("创建音轨", "Create track", "トラックを作成"),
        "修改音轨颜色" | "Change track color" | "トラックの色を変更" => lang.text("修改音轨颜色", "Change track color", "トラックの色を変更"),
        "编辑音符音素" | "Edit note phonemes" | "ノート音素を編集" => lang.text("编辑音符音素", "Edit note phonemes", "ノート音素を編集"),
        "填入歌词" | "Fill lyrics" | "歌詞を入力" => lang.text("填入歌词", "Fill lyrics", "歌詞を入力"),
        "插入音符组" | "Insert note group" | "ノートグループを挿入" => lang.text("插入音符组", "Insert note group", "ノートグループを挿入"),
        "删除音符组" | "Delete note group" | "ノートグループを削除" => lang.text("删除音符组", "Delete note group", "ノートグループを削除"),
        "重命名音符组" | "Rename note group" | "ノートグループ名を変更" => lang.text("重命名音符组", "Rename note group", "ノートグループ名を変更"),
        "重命名音轨" | "Rename track" | "トラック名を変更" => lang.text("重命名音轨", "Rename track", "トラック名を変更"),
        "删除音轨" | "Delete track" | "トラックを削除" => lang.text("删除音轨", "Delete track", "トラックを削除"),
        "调整音轨顺序" | "Reorder tracks" | "トラックを並び替え" => lang.text("调整音轨顺序", "Reorder tracks", "トラックを並び替え"),
        "编辑音素" | "Edit phonemes" | "音素を編集" => lang.text("编辑音素", "Edit phonemes", "音素を編集"),
        "切换音符语言" | "Change note language" | "ノートの言語を変更" => lang.text("切换音符语言", "Change note language", "ノートの言語を変更"),
        "切换音符模式" | "Change note mode" | "ノートモードを変更" => lang.text("切换音符模式", "Change note mode", "ノートモードを変更"),
        "调整音符演绎" | "Adjust note expression" | "ノートの表現を調整" => lang.text("调整音符演绎", "Adjust note expression", "ノートの表現を調整"),
        "调整颤音调制" | "Adjust vibrato" | "ビブラートを調整" => lang.text("调整颤音调制", "Adjust vibrato", "ビブラートを調整"),
        "调整说唱音调" | "Adjust rap tone" | "ラップの声調を調整" => lang.text("调整说唱音调", "Adjust rap tone", "ラップの声調を調整"),
        "调整说唱语调" | "Adjust rap intonation" | "ラップのイントネーションを調整" => lang.text("调整说唱语调", "Adjust rap intonation", "ラップのイントネーションを調整"),
        "切换说唱声调" | "Change rap tone" | "ラップの声調を変更" => lang.text("切换说唱声调", "Change rap tone", "ラップの声調を変更"),
        "调整音素时值" | "Adjust phoneme timing" | "音素の長さを調整" => lang.text("调整音素时值", "Adjust phoneme timing", "音素の長さを調整"),
        "设置音符组声库" | "Set group voice" | "グループの歌声を設定" => lang.text("设置音符组声库", "Set group voice", "グループの歌声を設定"),
        "批量切换音轨歌手" | "Change voices for selected tracks" | "選択トラックの歌声を変更" => lang.text("批量切换音轨歌手", "Change voices for selected tracks", "選択トラックの歌声を変更"),
        "切换音轨歌手" | "Change track voice" | "トラックの歌声を変更" => lang.text("切换音轨歌手", "Change track voice", "トラックの歌声を変更"),
        "切换歌声语言" | "Change voice language" | "歌声の言語を変更" => lang.text("切换歌声语言", "Change voice language", "歌声の言語を変更"),
        "调整调音参数" | "Adjust tuning" | "チューニングを調整" => lang.text("调整调音参数", "Adjust tuning", "チューニングを調整"),
        "调整歌声参数" | "Adjust voice parameters" | "歌声パラメーターを調整" => lang.text("调整歌声参数", "Adjust voice parameters", "歌声パラメーターを調整"),
        "调整唱法" | "Adjust vocal modes" | "歌唱スタイルを調整" => lang.text("调整唱法", "Adjust vocal modes", "歌唱スタイルを調整"),
        "重置不支持的唱法参数" | "Reset unsupported vocal modes" | "未対応の歌唱スタイル設定をリセット" => lang.text("重置不支持的唱法参数", "Reset unsupported vocal modes", "未対応の歌唱スタイル設定をリセット"),
        "删除曲速" | "Delete tempo" | "テンポを削除" => lang.text("删除曲速", "Delete tempo", "テンポを削除"),
        "删除拍号" | "Delete time signature" | "拍子を削除" => lang.text("删除拍号", "Delete time signature", "拍子を削除"),
        "编辑曲速" | "Edit tempo" | "テンポを編集" => lang.text("编辑曲速", "Edit tempo", "テンポを編集"),
        "插入曲速" | "Insert tempo" | "テンポを挿入" => lang.text("插入曲速", "Insert tempo", "テンポを挿入"),
        "编辑拍号" | "Edit time signature" | "拍子を編集" => lang.text("编辑拍号", "Edit time signature", "拍子を編集"),
        "插入拍号" | "Insert time signature" | "拍子を挿入" => lang.text("插入拍号", "Insert time signature", "拍子を挿入"),
        "创建伴奏轨" | "Create instrumental track" | "伴奏トラックを作成" => lang.text("创建伴奏轨", "Create instrumental track", "伴奏トラックを作成"),
        "导入伴奏" | "Import instrumental audio" | "伴奏をインポート" => lang.text("导入伴奏", "Import instrumental audio", "伴奏をインポート"),
        "移除伴奏" | "Remove instrumental audio" | "伴奏を削除" => lang.text("移除伴奏", "Remove instrumental audio", "伴奏を削除"),
        "移动伴奏" | "Move instrumental" | "伴奏を移動" => lang.text("移动伴奏", "Move instrumental", "伴奏を移動"),
        "修改调式音阶" | "Change musical scale" | "スケールを変更" => lang.text("修改调式音阶", "Change musical scale", "スケールを変更"),
        "创建音符" | "Create note" | "ノートを作成" => lang.text("创建音符", "Create note", "ノートを作成"),
        "拖动音符" | "Move notes" | "ノートを移動" => lang.text("拖动音符", "Move notes", "ノートを移動"),
        "剪切音高控制点" | "Cut pitch anchors" | "ピッチ制御点を切り取る" => lang.text("剪切音高控制点", "Cut pitch anchors", "ピッチ制御点を切り取る"),
        "粘贴音高控制点" | "Paste pitch anchors" | "ピッチ制御点を貼り付ける" => lang.text("粘贴音高控制点", "Paste pitch anchors", "ピッチ制御点を貼り付ける"),
        "复制音高控制点" | "Duplicate pitch anchors" | "ピッチ制御点を複製" => lang.text("复制音高控制点", "Duplicate pitch anchors", "ピッチ制御点を複製"),
        "删除音高控制点" | "Delete pitch anchors" | "ピッチ制御点を削除" => lang.text("删除音高控制点", "Delete pitch anchors", "ピッチ制御点を削除"),
        "创建音高控制点" | "Create pitch anchor" | "ピッチ制御点を作成" => lang.text("创建音高控制点", "Create pitch anchor", "ピッチ制御点を作成"),
        "缩放音高控制点" | "Scale pitch anchors" | "ピッチ制御点を拡大縮小" => lang.text("缩放音高控制点", "Scale pitch anchors", "ピッチ制御点を拡大縮小"),
        "移动音高控制点" | "Move pitch anchors" | "ピッチ制御点を移動" => lang.text("移动音高控制点", "Move pitch anchors", "ピッチ制御点を移動"),
        "音高控制点升八度" | "Pitch anchors up one octave" | "ピッチ制御点を1オクターブ上げる" => lang.text("音高控制点升八度", "Pitch anchors up one octave", "ピッチ制御点を1オクターブ上げる"),
        "音高控制点降八度" | "Pitch anchors down one octave" | "ピッチ制御点を1オクターブ下げる" => lang.text("音高控制点降八度", "Pitch anchors down one octave", "ピッチ制御点を1オクターブ下げる"),
        "手绘音高" | "Draw pitch" | "ピッチを描く" => lang.text("手绘音高", "Draw pitch", "ピッチを描く"),
        "擦除手绘音高" | "Erase pitch" | "ピッチを消去" => lang.text("擦除手绘音高", "Erase pitch", "ピッチを消去"),
        "添加参数控制点" | "Add parameter point" | "パラメーター制御点を追加" => lang.text("添加参数控制点", "Add parameter point", "パラメーター制御点を追加"),
        "清空参数控制点" | "Clear parameter points" | "パラメーター制御点をすべて消去" => lang.text("清空参数控制点", "Clear parameter points", "パラメーター制御点をすべて消去"),
        "移动参数控制点" | "Move parameter points" | "パラメーター制御点を移動" => lang.text("移动参数控制点", "Move parameter points", "パラメーター制御点を移動"),
        "擦除参数控制点" | "Erase parameter points" | "パラメーター制御点を消去" => lang.text("擦除参数控制点", "Erase parameter points", "パラメーター制御点を消去"),
        "绘制参数曲线" | "Draw parameter curve" | "パラメーター曲線を描く" => lang.text("绘制参数曲线", "Draw parameter curve", "パラメーター曲線を描く"),
        "剪切参数控制点" | "Cut parameter points" | "パラメーター制御点を切り取る" => lang.text("剪切参数控制点", "Cut parameter points", "パラメーター制御点を切り取る"),
        "删除参数控制点" | "Delete parameter points" | "パラメーター制御点を削除" => lang.text("删除参数控制点", "Delete parameter points", "パラメーター制御点を削除"),
        "粘贴参数控制点" | "Paste parameter points" | "パラメーター制御点を貼り付ける" => lang.text("粘贴参数控制点", "Paste parameter points", "パラメーター制御点を貼り付ける"),
        "重复参数控制点" | "Duplicate parameter points" | "パラメーター制御点を複製" => lang.text("重复参数控制点", "Duplicate parameter points", "パラメーター制御点を複製"),
        "参数插值模式" | "Parameter interpolation mode" | "パラメーター補間モード" => lang.text("参数插值模式", "Parameter interpolation mode", "パラメーター補間モード"),
        "音轨混音" | "Track mixing" | "トラックミキシング" => lang.text("音轨混音", "Track mixing", "トラックミキシング"),
        "调整混音" | "Adjust mixer" | "ミキサー調整" => lang.text("调整混音", "Adjust mixer", "ミキサー調整"),
        "选择发音词典" | "Select pronunciation dictionary" | "発音辞書を選択" => lang.text("选择发音词典", "Select pronunciation dictionary", "発音辞書を選択"),
        "新建发音词典" | "Create pronunciation dictionary" | "発音辞書を新規作成" => lang.text("新建发音词典", "Create pronunciation dictionary", "発音辞書を新規作成"),
        "移除发音词典" | "Remove pronunciation dictionary" | "発音辞書を削除" => lang.text("移除发音词典", "Remove pronunciation dictionary", "発音辞書を削除"),
        "编辑发音词典" | "Edit pronunciation dictionary" | "発音辞書を編集" => lang.text("编辑发音词典", "Edit pronunciation dictionary", "発音辞書を編集"),
        "导入 MIDI 音轨" | "Import MIDI tracks" | "MIDI トラックをインポート" => lang.text("导入 MIDI 音轨", "Import MIDI tracks", "MIDI トラックをインポート"),
        "导入 MIDI" | "Import MIDI" | "MIDI をインポート" => lang.text("导入 MIDI", "Import MIDI", "MIDI をインポート"),
        "调整音节分配" | "Adjust syllable assignment" | "音節の割り当てを調整" => lang.text("调整音节分配", "Adjust syllable assignment", "音節の割り当てを調整"),
        "重录新的采样" | "Generate new retake" | "新しいリテイクを生成" => lang.text("重录新的采样", "Generate new retake", "新しいリテイクを生成"),
        "重录音高和音色采样" | "Retake pitch and timbre" | "ピッチと音色をリテイク" => lang.text("重录音高和音色采样", "Retake pitch and timbre", "ピッチと音色をリテイク"),
        "重录时值采样" | "Retake duration" | "長さをリテイク" => lang.text("重录时值采样", "Retake duration", "長さをリテイク"),
        "重录音高采样" | "Retake pitch" | "ピッチをリテイク" => lang.text("重录音高采样", "Retake pitch", "ピッチをリテイク"),
        "重录音色采样" | "Retake timbre" | "音色をリテイク" => lang.text("重录音色采样", "Retake timbre", "音色をリテイク"),
        "切换重录采样" | "Switch retake" | "リテイクを切り替え" => lang.text("切换重录采样", "Switch retake", "リテイクを切り替え"),
        "裁剪保留使用采样" | "Prune unused retakes" | "未使用リテイクを整理" => lang.text("裁剪保留使用采样", "Prune unused retakes", "未使用リテイクを整理"),
        "粘贴音符" | "Paste notes" | "ノートを貼り付ける" => lang.text("粘贴音符", "Paste notes", "ノートを貼り付ける"),
        "重复音符" | "Duplicate notes" | "ノートを複製" => lang.text("重复音符", "Duplicate notes", "ノートを複製"),
        _ => label,
    }
}

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
                &format!("{} {}", lang.text("撤销", "Undo", "元に戻す"), localized_undo_label(lang, app.editor.undo_label())),
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
                .checkbox(&mut app.view.arrangement_collapsed, lang.text("折叠编排", "Collapse arrangement", "アレンジを折りたたむ"))
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
    if command(Key::F, Modifiers::SHIFT) {
        super::piano::fit_song(app, ctx);
    }
    if command(Key::A, shift) {
        app.editor.select_all_notes();
    } else if command(Key::A, ctrl) {
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
