use super::widgets::section;
use crate::ui::theme::*;
use eframe::egui::*;

pub(super) fn show(ui: &mut Ui, lang: super::UiLanguage) {
    ui.weak(lang.text("当前快捷键；自定义绑定尚未接入。", "Current shortcuts; custom key bindings are not available yet.", "現在のショートカットです。キー割り当てのカスタマイズは未対応です。"));
    for (title, entries) in [
        (
            lang.text("工程", "Project", "プロジェクト"),
            &[
                (lang.text("新建", "New", "新規"), "Ctrl+N"),
                (lang.text("打开", "Open", "開く"), "Ctrl+O"),
                (lang.text("保存", "Save", "保存"), "Ctrl+S"),
                (lang.text("另存为", "Save as", "名前を付けて保存"), "Ctrl+Shift+S"),
            ][..],
        ),
        (
            lang.text("编辑", "Edit", "編集"),
            &[
                (lang.text("撤销", "Undo", "元に戻す"), "Ctrl+Z"),
                (lang.text("重做", "Redo", "やり直し"), "Ctrl+Y / Ctrl+Shift+Z"),
                (lang.text("全选音符", "Select all notes", "すべてのノートを選択"), "Ctrl+A"),
                (lang.text("复制 / 剪切 / 粘贴", "Copy / Cut / Paste", "コピー／切り取り／貼り付け"), "Ctrl+C / X / V"),
                (lang.text("重复音符", "Duplicate notes", "ノートを複製"), "Ctrl+D"),
                (lang.text("删除音符", "Delete notes", "ノートを削除"), "Delete"),
                (lang.text("取消选择", "Deselect", "選択解除"), "Esc"),
                (lang.text("填入歌词", "Fill lyrics", "歌詞を入力"), "Ctrl+L"),
                (lang.text("静音音符", "Mute notes", "ノートをミュート"), "Ctrl+M"),
                (lang.text("重录新的采样", "Create new retake", "新しいリテイクを作成"), "Alt+T"),
                (lang.text("裁剪保留使用采样", "Prune unused retakes", "未使用のリテイクを整理"), "Shift+Alt+T"),
                (lang.text("升高 / 降低八度", "Octave up / down", "オクターブを上げる／下げる"), "Ctrl+↑ / ↓"),
                (lang.text("音符面板", "Note panel", "ノートパネル"), "Ctrl+B"),
            ][..],
        ),
        (
            lang.text("走带控制", "Transport", "再生操作"),
            &[
                (lang.text("播放 / 暂停", "Play / Pause", "再生／一時停止"), "Space"),
                (lang.text("停止", "Stop", "停止"), "Ctrl+Space"),
                (lang.text("跳到结尾", "Go to end", "末尾へ移動"), "Ctrl+End"),
                (lang.text("按网格移动走带", "Move playhead by grid", "グリッド単位で再生位置を移動"), "← / →"),
            ][..],
        ),
        (
            lang.text("工具与视野", "Tools and view", "ツールと表示"),
            &[
                (lang.text("选择 / 铅笔工具", "Select / Pencil", "選択／ペンツール"), "Alt+1 / Alt+2"),
                (lang.text("创建音符 / 编辑歌词", "Create note / Edit lyrics", "ノート作成／歌詞編集"), lang.text("双击", "Double-click", "ダブルクリック")),
                (lang.text("平移视野", "Pan view", "表示位置を移動"), lang.text("中键拖动", "Middle-drag", "中ボタンドラッグ")),
                (lang.text("水平滚动", "Horizontal scroll", "水平スクロール"), lang.text("Shift+滚轮", "Shift+Wheel", "Shift+ホイール")),
                (lang.text("缩放", "Zoom", "ズーム"), lang.text("Ctrl+滚轮", "Ctrl+Wheel", "Ctrl+ホイール")),
            ][..],
        ),
    ] {
        section(ui, title);
        for (label, key) in entries {
            ui.horizontal(|ui| {
                ui.set_min_height(28.0);
                ui.label(*label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    Frame::new()
                        .fill(BUTTON)
                        .corner_radius(3)
                        .inner_margin(Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.label(RichText::new(*key).color(TEXT));
                        });
                });
            });
        }
    }
}
