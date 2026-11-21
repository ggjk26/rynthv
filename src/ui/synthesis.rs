use crate::app::App;
use eframe::egui::*;

pub fn render_status(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    ui.label(RichText::new(lang.text("歌声合成", "Vocal synthesis", "歌声合成")).strong());
    ui.separator();
    ui.label(lang.runtime_status(app.synthesis.status()));
    if let Some(progress) = &app.synthesis.progress {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(progress);
        });
    }
    let stats = app.synthesis.stats;
    ui.label(format!(
        "{} {} · {} {} · {} {}",
        lang.text("已完成", "Ready", "完了"), stats.ready,
        lang.text("等待", "Queued", "待機"), stats.queued,
        lang.text("失败", "Failed", "失敗"), stats.failed
    ));
    if let Some(error) = &app.synthesis.error {
        ui.colored_label(Color32::from_rgb(218, 142, 126), error);
    }
    for (selection, name) in app.synthesis.context_import_targets() {
        if ui.button(format!("{} {name}…", lang.text("导入模型上下文：", "Import model context for", "モデルコンテキストをインポート："))).clicked() {
            app.synthesis.open_context_import(selection);
        }
    }
    if !app.synthesis.failures.is_empty() {
        ui.collapsing(lang.text("失败详情", "Failure details", "失敗の詳細"), |ui| {
            for (owner, error) in &app.synthesis.failures {
                ui.label(format!("{} {}: {error}", lang.text("音符组", "Note group", "ノートグループ"), owner.0));
            }
        });
    }
    if (stats.failed > 0 || app.synthesis.error.is_some()) && ui.button(lang.text("重试合成", "Retry synthesis", "合成を再試行")).clicked() {
        app.synthesis.retry();
    }
    if let Some(error) = app.synthesis.playback_error() {
        ui.colored_label(
            Color32::from_rgb(218, 142, 126),
            format!("{}: {error}", lang.text("播放设备", "Playback device", "再生デバイス")),
        );
    }
    ui.add_space(10.0);
    if let Some(duration) = app.synthesis.audio_duration() {
        ui.label(format!("{} {:.2} {} · 48 kHz · {}", lang.text("当前混音", "Current mix", "現在のミックス"), duration, lang.text("秒", "s", "秒"), lang.text("立体声", "stereo", "ステレオ")));
    }
    ui.collapsing(lang.text("合成计划", "Synthesis plan", "合成プラン"), |ui| {
        let summary = app.synthesis.planner.summary();
        ui.label(format!("{} {}", summary.pending_owners, lang.text("个音符组纳入当前计划", "note groups in the current plan", "ノートグループが現在のプランに含まれます")));
        ui.label(format!(
            "{} {} · {} {}",
            lang.text("本次变化", "Changed", "変更"), summary.changed_owners,
            lang.text("保留请求", "Retained", "維持"), summary.unchanged_owners
        ));
        ui.weak(lang.text("优先准备播放位置的音频，其余部分在后台继续合成；无改动区段复用已有结果。", "Audio near the playhead is prioritized. Unchanged segments reuse previous results.", "再生位置付近の音声を優先して合成します。変更のない部分は既存の結果を再利用します。"));
    });
}
