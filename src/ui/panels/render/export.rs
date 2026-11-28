use super::{error, padded};
use crate::{app::App, synthesis::runtime::ExportRequest};
use eframe::egui::*;

pub(super) fn show(app: &mut App, ui: &mut Ui, has_tracks: bool) {
    let lang = app.preferences.language;
    let exporting = app.synthesis.is_exporting();
    let editing = app.editor.is_editing();
    padded(ui, |ui| {
        let options = ExportRequest::check_options(&app.editor.project.render_config, has_tracks);
        let reason = match &options {
            _ if exporting => Some(lang.text("正在导出…", "Exporting…", "書き出し中…")),
            Err(message) => Some(lang.editor_message(message)),
            _ if editing => Some(lang.text(
                "请先结束当前编辑",
                "Finish the current edit first",
                "先に現在の編集を完了してください",
            )),
            _ if app.synthesis.export_ready(app.editor.revision) => None,
            _ => app.synthesis.recording_error.as_deref().or(Some(lang.text(
                "等待当前工程有可导出的音频",
                "Waiting for exportable audio from the current project",
                "現在のプロジェクトで書き出し可能な音声を待機中",
            ))),
        };
        let button = ui.add_enabled(
            reason.is_none(),
            Button::new(lang.text("导出为文件", "Export to files", "ファイルに書き出す"))
                .min_size(vec2(ui.available_width(), 28.0)),
        );
        if let Some(reason) = reason {
            if exporting {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.weak(reason);
                });
            } else {
                ui.weak(reason);
            }
        }
        if button.clicked() {
            start_export(app);
        }
        if let Some(result) = &app.synthesis.export_result {
            match result {
                Ok(paths) => {
                    ui.collapsing(
                        format!(
                            "{} {}",
                            paths.len(),
                            lang.text("个文件已导出", "files exported", "ファイルを書き出しました")
                        ),
                        |ui| {
                            for path in paths {
                                ui.small(path.display().to_string());
                            }
                        },
                    );
                }
                Err(message) => error(ui, lang.editor_message(message)),
            }
        }
    });
}

fn start_export(app: &mut App) {
    let lang = app.preferences.language;
    let request = match ExportRequest::from_project(&app.editor.project) {
        Ok(request) => request,
        Err(message) => {
            app.synthesis.export_result = Some(Err(message));
            return;
        }
    };
    let mut existing = Vec::new();
    for (path, _, _) in &request.outputs {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => existing.push(path.display().to_string()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            result => {
                let reason = match result {
                    Ok(_) => lang
                        .text(
                            "输出路径被非普通文件占用",
                            "The output path is occupied by a non-regular file",
                            "出力先に通常ファイル以外が存在します",
                        )
                        .into(),
                    Err(error) => error.to_string(),
                };
                app.synthesis.export_result = Some(Err(format!("{}: {reason}", path.display())));
                return;
            }
        }
    }
    if !existing.is_empty()
        && rfd::MessageDialog::new()
            .set_title(lang.text(
                "覆盖音频文件",
                "Overwrite audio files",
                "音声ファイルを上書き",
            ))
            .set_description(format!(
                "{}\n{}",
                lang.text(
                    "以下文件已存在，是否覆盖？",
                    "The following files already exist. Overwrite?",
                    "次のファイルは既に存在します。上書きしますか？"
                ),
                existing.join("\n")
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            != rfd::MessageDialogResult::Yes
    {
        return;
    }
    app.synthesis.export_project(request, app.editor.revision);
}
