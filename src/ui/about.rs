//! About RynthV: build identity, live graphics diagnostics and attribution.
//! The About view uses the same locale and dark surfaces as the editor.

use crate::{
    app::App,
    ui::theme::{BACKGROUND, BORDER, CANVAS, GREEN, MUTED, PANEL, TEXT},
};
use eframe::egui::*;

fn detail(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized([148.0, 18.0], Label::new(RichText::new(label).color(MUTED)));
        ui.label(RichText::new(value).color(TEXT));
    });
}

fn section(ui: &mut Ui, title: &str, body: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(CANVAS)
        .corner_radius(8)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(14.0).strong().color(TEXT));
            ui.add_space(9.0);
            body(ui);
        });
}

pub fn show(app: &mut App, ctx: &Context) {
    let lang = app.preferences.language;
    let mut close = false;
    let width = (ctx.content_rect().width() - 44.0).clamp(320.0, 680.0);
    let height = (ctx.content_rect().height() - 140.0).clamp(220.0, 500.0);

    let build = option_env!("RYNTHV_BUILD_REV").unwrap_or("development");
    let version = env!("CARGO_PKG_VERSION");
    let system = format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH);
    let requested = app.graphics.configured.label(lang);
    let diagnostics = format!(
        "RynthV {version} ({build})\nPlatform: {system}\nInterface: {}\nGraphics preference at startup: {requested}\nActive graphics backend: {}\nGPU: {}\nDevice type: {}\nVendor:Device: {}\nDriver: {}\nVoicebanks discovered: {}\nTracks: {}\nRender status: {}",
        lang.name(),
        app.graphics.active_backend,
        app.graphics.gpu_name,
        app.graphics.gpu_type,
        app.graphics.vendor_device,
        app.graphics.driver,
        app.synthesis.library.catalog().map_or(0, |catalog| catalog.entries.len()),
        app.editor.project.tracks.len(),
        app.synthesis.status(),
    );

    let modal = Modal::new(Id::new("about"))
        .backdrop_color(Color32::from_black_alpha(165))
        .frame(
            Frame::new()
                .fill(PANEL)
                .corner_radius(12)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(16),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            // Branded title banner, deliberately independent of OS renderer.
            Frame::new()
                .fill(BACKGROUND)
                .corner_radius(10)
                .inner_margin(16)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.add(
                            Image::new(include_image!("../../assets/branding/app-icon.png"))
                                .fit_to_exact_size(vec2(62.0, 62.0)),
                        );
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("RynthV").size(26.0).strong().color(TEXT));
                            ui.label(RichText::new(format!(
                                "{} {version}  ·  {build}",
                                lang.text("版本", "Version", "バージョン")
                            )).color(MUTED));
                            ui.label(RichText::new(lang.text(
                                "开放式歌声合成工作站",
                                "Open singing synthesis workstation",
                                "オープンな歌声合成ワークステーション",
                            )).color(GREEN));
                        });
                    });
                });
            ui.add_space(12.0);

            ScrollArea::vertical()
                .id_salt("about-detail-scroll")
                .max_height(height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    section(ui, lang.text("项目简介", "About the project", "プロジェクト概要"), |ui| {
                        ui.label(lang.text(
                            "基于 Rust、egui 和 wgpu 的多音轨歌声编辑器，提供歌词与音素编辑、音高调整、声库管理、MIDI 导入和 WAV 导出。",
                            "A multitrack singing editor built with Rust, egui and wgpu, featuring lyrics/phoneme editing, pitch controls, voicebank management, MIDI import and WAV export.",
                            "Rust・egui・wgpu 製のマルチトラック歌声エディター。歌詞・音素編集、ピッチ調整、歌声ライブラリ管理、MIDI 読込、WAV 出力に対応します。",
                        ));
                        ui.add_space(5.0);
                        ui.weak(lang.text(
                            "这是独立的社区项目，并非 Dreamtonics 或其他声库开发商的官方软件。",
                            "An independent community project, not an official product of Dreamtonics or any voicebank vendor.",
                            "Dreamtonics および歌声ライブラリ各社の公式製品ではない、独立したコミュニティプロジェクトです。",
                        ));
                    });
                    ui.add_space(10.0);
                    section(ui, lang.text("运行环境", "Runtime information", "動作環境"), |ui| {
                        detail(ui, lang.text("操作系统", "Operating system", "OS"), &system);
                        detail(ui, lang.text("图形接口", "Graphics API", "描画 API"), &app.graphics.active_backend);
                        detail(ui, lang.text("图形设备", "GPU adapter", "GPU アダプター"), &app.graphics.gpu_name);
                        detail(ui, lang.text("设备类型", "Adapter type", "アダプター種別"), &app.graphics.gpu_type);
                        detail(ui, lang.text("设备 ID", "Vendor / Device", "デバイス ID"), &app.graphics.vendor_device);
                        detail(ui, lang.text("驱动", "Driver", "ドライバー"), &app.graphics.driver);
                        detail(ui, lang.text("启动时的图形选择", "Startup graphics setting", "起動時の描画設定"), requested);
                        if app.preferences.graphics_backend != app.graphics.preference_at_start {
                            ui.add_space(6.0);
                            ui.colored_label(
                                Color32::from_rgb(242, 189, 115),
                                lang.text(
                                    "图形后端设置已更改，重新启动后生效。",
                                    "Graphics preference changed; restart to apply.",
                                    "描画設定の変更は再起動後に反映されます。",
                                ),
                            );
                        }
                    });
                    ui.add_space(10.0);
                    section(ui, lang.text("项目与许可证", "Project & licenses", "プロジェクト・ライセンス"), |ui| {
                        detail(ui, lang.text("项目许可证", "Project license", "ライセンス"), "MIT License © 2026 rynthv");
                        ui.horizontal_wrapped(|ui| {
                            ui.hyperlink_to("GitHub", "https://github.com/ggjk26/rynthv");
                            ui.separator();
                            ui.hyperlink_to(
                                lang.text("反馈问题", "Report issues", "問題を報告"),
                                "https://github.com/ggjk26/rynthv/issues",
                            );
                            ui.separator();
                            ui.hyperlink_to(
                                lang.text("源代码许可证", "Source license", "ソースのライセンス"),
                                "https://github.com/ggjk26/rynthv/blob/master/LICENSE",
                            );
                        });
                        ui.add_space(5.0);
                        ui.weak(lang.text(
                            "第三方词典、依赖库与声库分别遵循各自的许可证。发行程序不包含商业声库。",
                            "Third-party dictionaries, libraries and voicebanks keep their own licenses. Commercial voicebanks are not bundled.",
                            "第三者辞書・ライブラリ・歌声データには各ライセンスが適用されます。商用歌声は同梱されません。",
                        ));
                    });
                });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(lang.text(
                    "复制诊断信息", "Copy diagnostics", "診断情報をコピー",
                )).clicked() {
                    ctx.copy_text(diagnostics.clone());
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add_sized(
                        [86.0, 29.0],
                        Button::new(lang.text("关闭", "Close", "閉じる"))
                            .fill(Color32::from_rgb(71, 97, 58)),
                    ).clicked() {
                        close = true;
                    }
                });
            });
        });
    if close || modal.should_close() {
        app.about = false;
    }
}
