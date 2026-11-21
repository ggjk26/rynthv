use super::{text, weak};
use crate::{synthesis::runtime::{ContextImportDialog, ContextImportPhase, ContextImportSource}, ui::settings::UiLanguage};
use eframe::egui::*;

pub(super) fn show(ui: &mut Ui, dialog: &mut ContextImportDialog, lang: UiLanguage) -> bool {
    match dialog.source {
        ContextImportSource::SessionFile => session_file(ui, dialog, lang),
        ContextImportSource::File => file(ui, dialog, lang),
        ContextImportSource::Manual => manual(ui, dialog, lang),
        ContextImportSource::Online => online(ui, dialog, lang),
    }
}

fn session_file(ui: &mut Ui, dialog: &mut ContextImportDialog, lang: UiLanguage) -> bool {
    text(
        ui,
        lang.text("选择本机 Synthesizer V Studio 2 的 session 登录会话文件，无需启动原编辑器。", "Select a local Synthesizer V Studio 2 session file; the original editor need not be running.", "Synthesizer V Studio 2 のローカルセッションファイルを選択します。元のエディターを起動する必要はありません。"),
    );
    weak(
        ui,
        lang.text("点击“导入”后才会读取，并将此声库的上下文保存到 RV 的 context.json。", "The file is only read on Import; this voice's context is saved to RynthV context.json.", "「インポート」を押した時だけ読み取り、この歌声の情報を RynthV の context.json に保存します。"),
    );
    if let Some(path) = &dialog.session_path {
        ui.add(Label::new(path.display().to_string()).wrap());
    } else {
        weak(ui, lang.text("尚未选择会话文件", "No session file selected", "セッションファイル未選択"));
    }
    if !ui.button(lang.text("选择会话文件…", "Choose session file…", "セッションファイルを選択…")).clicked() {
        return false;
    }
    let mut picker = rfd::FileDialog::new().set_title(lang.text("选择 Synthesizer V Studio 2 的会话文件", "Select Synthesizer V Studio 2 session file", "Synthesizer V Studio 2 のセッションファイルを選択"));
    if let Some(parent) = dialog.session_path.as_ref().and_then(|path| path.parent()) {
        picker = picker.set_directory(parent);
    }
    if let Some(path) = picker.pick_file() {
        dialog.session_path = Some(path);
        return true;
    }
    false
}

fn file(ui: &mut Ui, dialog: &mut ContextImportDialog, lang: UiLanguage) -> bool {
    text(ui, lang.text("选择包含此声库及对应模型代际的上下文 JSON 文件。", "Select a context JSON file for this voice and model generation.", "この歌声とモデル世代に対応するコンテキスト JSON を選択してください。"));
    if let Some(path) = &dialog.file_path {
        ui.add(Label::new(path.display().to_string()).wrap());
    } else {
        weak(ui, lang.text("尚未选择文件", "No file selected", "ファイル未選択"));
    }
    if !ui.button(lang.text("选择上下文文件…", "Choose context file…", "コンテキストファイルを選択…")).clicked() {
        return false;
    }
    let mut picker = rfd::FileDialog::new()
        .set_title(lang.text("选择模型上下文文件", "Select model context file", "モデルコンテキストファイルを選択"))
        .add_filter(lang.text("模型上下文 JSON", "Model context JSON", "モデルコンテキスト JSON"), &["json"]);
    if let Some(parent) = dialog.file_path.as_ref().and_then(|path| path.parent()) {
        picker = picker.set_directory(parent);
    }
    if let Some(path) = picker.pick_file() {
        dialog.file_path = Some(path);
        return true;
    }
    false
}

fn manual(ui: &mut Ui, dialog: &mut ContextImportDialog, lang: UiLanguage) -> bool {
    text(ui, lang.text("填写此声库及对应模型代际的 identity 模型条件值。", "Enter the identity conditioning value for this voice and model generation.", "この歌声とモデル世代に対応する identity 条件値を入力してください。"));
    ui.add(
        TextEdit::singleline(&mut dialog.manual_identity)
            .desired_width(f32::INFINITY)
            .hint_text(lang.text("十进制整数或 0x 十六进制", "Decimal integer or 0x hexadecimal", "10進数または 0x 16進数"))
            .password(true),
    )
    .changed()
}

fn online(ui: &mut Ui, dialog: &mut ContextImportDialog, lang: UiLanguage) -> bool {
    text(
        ui,
        lang.text("打开并登录 Synthesizer V Studio 2，选择其进程后导入。完成后可关闭原编辑器。", "Open and sign in to Synthesizer V Studio 2, then select its process to import. You may close it afterward.", "Synthesizer V Studio 2 にログインし、プロセスを選択してインポートします。完了後は閉じても構いません。"),
    );
    if dialog.phase == ContextImportPhase::Discovering {
        ui.horizontal(|ui| {
            ui.spinner();
            text(ui, lang.text("正在查找已打开的编辑器…", "Searching for running editors…", "起動中のエディターを検索中…"));
        });
        return false;
    }
    if dialog.sources.is_empty() {
        weak(ui, lang.text("未找到可用的编辑器进程。打开原编辑器后点击刷新。", "No editor process found. Open the original editor and select Refresh.", "エディターのプロセスが見つかりません。元のエディターを開いて更新してください。"));
        return false;
    }
    let mut changed = false;
    ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
        for source in &dialog.sources {
            changed |= ui
                .radio_value(
                    &mut dialog.selected_process,
                    Some(source.process_id),
                    format!("{} · PID {}", source.label, source.process_id),
                )
                .changed();
        }
    });
    changed
}
