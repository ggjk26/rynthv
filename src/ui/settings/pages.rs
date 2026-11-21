use super::widgets::*;
use crate::{app::App, ui::state::Column};
use eframe::egui::*;

pub(super) fn general(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    row(ui, lang.text("界面语言", "Interface language", "表示言語"), |ui| {
        ComboBox::from_id_salt("interface-language")
            .width(ui.available_width())
            .selected_text(app.preferences.language.name())
            .show_ui(ui, |ui| {
                for candidate in super::UiLanguage::ALL {
                    ui.selectable_value(&mut app.preferences.language, candidate, candidate.name());
                }
            });
    });
    row(ui, lang.text("界面缩放", "Interface scale", "画面の拡大率"), |ui| {
        let id = ui.make_persistent_id("zoom-draft");
        let mut percent = ui
            .ctx()
            .data_mut(|data| data.get_temp::<f32>(id))
            .unwrap_or(app.preferences.zoom * 100.0);
        ui.label(format!("{percent:.0}%"));
        ui.spacing_mut().slider_width = ui.available_width().max(40.0);
        let response = ui.add(
            Slider::new(&mut percent, 75.0..=200.0)
                .step_by(5.0)
                .show_value(false),
        );
        if response.drag_stopped() || response.changed() && !response.dragged() {
            app.preferences.zoom = percent / 100.0;
        }
        if response.dragged() {
            ui.ctx().data_mut(|data| data.insert_temp(id, percent));
        } else {
            ui.ctx().data_mut(|data| data.remove::<f32>(id));
        }
    });
    fixed(
        ui,
        lang.text("图形渲染", "Graphics backend", "描画バックエンド"),
        "Direct3D 12",
        lang.text("当前使用 GPU 渲染，暂不支持切换渲染后端", "The graphics backend cannot currently be changed", "現在、描画バックエンドは変更できません"),
    );
    section(ui, lang.text("帮助和指导", "Help & guidance", "ヘルプとガイド"));
    toggle(ui, lang.text("显示状态栏", "Show status bar", "ステータスバーを表示"), &mut app.preferences.show_status_bar);
    toggle(ui, lang.text("显示控件操作提示", "Show control hints", "操作ヒントを表示"), &mut app.preferences.control_hints);
    section(ui, lang.text("工作区", "Workspace", "ワークスペース"));
    row(ui, lang.text("工具列宽度", "Tool panel width", "ツールパネルの幅"), |ui| {
        ui.spacing_mut().slider_width = (ui.available_width() - 55.0).max(40.0);
        ui.add(Slider::new(&mut app.view.panel_width, 140.0..=420.0).suffix(" px"));
    });
    row(ui, lang.text("布局", "Layout", "レイアウト"), |ui| {
        if ui.button(lang.text("重置工作区布局", "Reset workspace layout", "レイアウトをリセット")).clicked() {
            app.view = Default::default();
        }
    });
}

pub(super) fn audio(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    section(ui, lang.text("音频输出", "Audio output", "オーディオ出力"));
    fixed(
        ui,
        lang.text("输出设备", "Output device", "出力デバイス"),
        lang.text("系统默认输出设备", "System default device", "システム既定のデバイス"),
        lang.text("当前跟随系统默认设备，设备切换尚未接入", "Uses the system default device; switching devices is not supported yet", "現在はシステム既定のデバイスを使用します。切り替えは未対応です"),
    );
    fixed(
        ui,
        lang.text("播放格式", "Playback format", "再生形式"),
        lang.text("设备默认格式", "Device default", "デバイス既定"),
        lang.text("预览音频自动适配输出设备的采样率和声道", "Preview audio adapts to the device sample rate and channels", "プレビュー音声はデバイスのサンプルレートとチャンネル数に合わせます"),
    );
    if let Some(error) = app.synthesis.playback_error() {
        ui.colored_label(Color32::from_rgb(218, 142, 126), error);
    }
    section(ui, lang.text("MIDI 输入", "MIDI input", "MIDI 入力"));
    fixed(ui, lang.text("输入设备", "Input device", "入力デバイス"), lang.text("未启用", "Disabled", "無効"), lang.text("MIDI 输入尚未接入", "MIDI input is not supported yet", "MIDI 入力は未対応です"));
}

pub(super) fn editing(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    section(ui, lang.text("钢琴卷帘", "Piano roll", "ピアノロール"));
    row(ui, lang.text("半音行高", "Semitone row height", "半音の行の高さ"), |ui| {
        ui.spacing_mut().slider_width = (ui.available_width() - 55.0).max(40.0);
        ui.add(Slider::new(&mut app.view.row_height, 12.0..=36.0).suffix(" px"));
    });
    toggle(ui, lang.text("音符对齐网格", "Snap notes to grid", "ノートをグリッドにスナップ"), &mut app.view.snap);
    row(ui, lang.text("网格精度", "Grid division", "グリッド分解能"), |ui| {
        ComboBox::from_id_salt("settings-grid")
            .width(ui.available_width())
            .selected_text(format!("{} {}", app.view.grid, lang.text("分音符", "th note", "分音符")))
            .show_ui(ui, |ui| {
                for value in [4, 8, 16, 32, 64, 128] {
                    ui.selectable_value(&mut app.view.grid, value, format!("{value} {}", lang.text("分音符", "th note", "分音符")));
                }
            });
    });
    toggle(ui, lang.text("三连音网格", "Triplet grid", "3連符グリッド"), &mut app.view.triplet);
    section(ui, lang.text("显示", "Display", "表示"));
    toggle(ui, lang.text("显示音高线", "Show pitch curve", "ピッチ曲線を表示"), &mut app.show_pitch);
    toggle(ui, lang.text("播放时按音高中位数纵向居中", "Center vertically on median note pitch during playback", "再生中、音高の中央値に縦方向を合わせる"), &mut app.view.auto_pitch_center);
    toggle(ui, lang.text("显示波形", "Show waveform", "波形を表示"), &mut app.show_waveform);
    toggle(ui, lang.text("编排位于上方", "Arrangement above piano roll", "アレンジを上に配置"), &mut app.view.arrangement_first);
    toggle(ui, lang.text("显示曲速和拍号轨", "Show tempo and time signature tracks", "テンポ・拍子トラックを表示"), &mut app.show_time_tracks);
    ui.add_space(10.0);
    ui.weak(lang.text("拖动标题可调整工具列顺序，拖动分隔线可调整区域尺寸。", "Drag headers to reorder tools and dividers to resize panels.", "見出しをドラッグしてツールを並べ替え、仕切りをドラッグしてサイズを変更します。"));
}

pub(super) fn recording(ui: &mut Ui, lang: super::UiLanguage) {
    fixed(ui, lang.text("录音输入设备", "Recording input device", "録音入力デバイス"), lang.text("未启用", "Disabled", "無効"), lang.text("录音功能尚未接入", "Recording is not supported yet", "録音は未対応です"));
    fixed(ui, lang.text("录音声道", "Recording channels", "録音チャンネル"), lang.text("未启用", "Disabled", "無効"), lang.text("录音功能尚未接入", "Recording is not supported yet", "録音は未対応です"));
    section(ui, lang.text("录制", "Recording", "録音"));
    ui.weak(lang.text("录制功能尚未接入，相关设置暂不可用。", "Recording is not available yet; these settings are disabled.", "録音機能は未対応のため、設定は利用できません。"));
}

pub(super) fn rendering(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    crate::ui::voice_library::settings(app, ui);
    row(ui, lang.text("歌声数据库", "Voice database", "歌声データベース"), |ui| {
        if ui.button(lang.text("管理歌声…", "Manage voices…", "歌声を管理…")).clicked() {
            app.settings = false;
            app.voice_manager = true;
        }
    });
    section(ui, lang.text("音频导出", "Audio export", "音声書き出し"));
    row(ui, lang.text("导出设置", "Export settings", "書き出し設定"), |ui| {
        if ui.button(lang.text("打开渲染面板", "Open render panel", "レンダリングパネルを開く")).clicked() {
            if !app.view.columns.contains(&Column::Render) {
                app.view.toggle(Column::Render);
            }
            app.settings = false;
        }
    });
    ui.weak(lang.text("目标文件夹、音轨、声道、位深度和采样率在渲染面板中设置。", "Set destination, tracks, channels, bit depth and sample rate in the render panel.", "保存先、トラック、チャンネル、ビット深度、サンプルレートはレンダリングパネルで設定します。"));
}
