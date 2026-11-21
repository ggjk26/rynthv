use crate::{
    app::App,
    ui::{
        icons::{Icon, IconButton},
        state::DragPayload,
        theme::*,
    },
};
use eframe::egui::*;

pub fn toolbar(app: &mut App, ui: &mut Ui) {
    ui.style_mut().always_scroll_the_only_direction = true;
    ScrollArea::horizontal()
        .id_salt("piano-toolbar-scroll")
        .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(ui, |ui| contents(app, ui));
}

fn contents(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        title(app, ui);
        editing_tools(app, ui);
        ui.add_space(15.0);
        for (icon, label, active) in [
            (Icon::Pitch, lang.text("显示音高曲线", "Show pitch curve", "ピッチカーブを表示"), &mut app.show_pitch),
            (Icon::Waveform, lang.text("显示渲染波形", "Show rendered waveform", "レンダリング波形を表示"), &mut app.show_waveform),
            (Icon::Retake, lang.text("显示 AI 重录小部件", "Show AI retake controls", "AI リテイク操作を表示"), &mut app.show_retakes),
        ] {
            if ui
                .add(
                    IconButton::new(icon, label)
                        .framed()
                        .size(vec2(32.0, 24.0))
                        .selected(*active),
                )
                .clicked()
            {
                *active = !*active;
            }
            ui.add_space(7.0);
        }
        super::scale_guide::toggle(app, ui);
        ui.add_space(7.0);
        ui.label(RichText::new(lang.text("网格", "Grid", "グリッド")).color(MUTED));
        ui.add_space(8.0);
        ComboBox::from_id_salt("grid")
            .width(96.0)
            .selected_text(format!(
                "1/{} {}",
                app.view.grid,
                if app.view.triplet {
                    lang.text("三连音", "triplet", "3連符")
                } else {
                    lang.text("音符", "note", "音符")
                }
            ))
            .show_ui(ui, |ui| {
                for value in [4, 8, 16, 32, 64, 128] {
                    if ui
                        .selectable_label(
                            app.view.grid == value && !app.view.triplet,
                            format!("1/{value} {}", lang.text("音符", "note", "音符")),
                        )
                        .clicked()
                    {
                        app.view.grid = value;
                        app.view.triplet = false;
                    }
                }
                for value in [8, 16, 32, 64] {
                    if ui
                        .selectable_label(
                            app.view.grid == value && app.view.triplet,
                            format!("1/{value} {}", lang.text("三连音", "triplet", "3連符")),
                        )
                        .clicked()
                    {
                        app.view.grid = value;
                        app.view.triplet = true;
                    }
                }
            });
        ui.add_space(9.0);
        if ui
            .add(
                IconButton::new(Icon::Magnet, lang.text("对齐网格", "Snap to grid", "グリッドにスナップ"))
                    .size(vec2(32.0, 24.0))
                    .framed()
                    .selected(app.view.snap),
            )
            .clicked()
        {
            app.view.snap = !app.view.snap;
        }
        crate::ui::scale::controls::toolbar(app, ui);
    });
}

fn title(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let (rect, _) = ui.allocate_exact_size(vec2(110.0, 28.0), Sense::hover());
    let mut header = crate::ui::child(ui, rect, "piano-title");
    header.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(4.0);
        if ui
            .add(
                IconButton::new(
                    if app.piano_collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    },
                    lang.text("展开 / 折叠钢琴卷帘", "Expand / collapse piano roll", "ピアノロールを展開／折りたたむ"),
                )
                .size(vec2(20.0, 24.0))
                .icon_size(12.0),
            )
            .clicked()
        {
            app.commit_lyric();
            app.piano_collapsed = !app.piano_collapsed;
            ui.ctx().request_repaint();
        }
        ui.add_space(3.0);
        crate::ui::section_title(ui, lang.text("钢琴卷帘", "Piano roll", "ピアノロール"), DragPayload::Piano);
    });
}

fn editing_tools(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let (rect, _) = ui.allocate_exact_size(vec2(160.0, 24.0), Sense::hover());
    ui.painter().rect_filled(rect, 3, BUTTON);
    let mut group = crate::ui::child(ui, rect, "piano-editing-tools");
    group.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if ui
            .add(
                IconButton::new(Icon::NoteSelect, lang.text("编辑音符（Alt+1）", "Select/edit notes (Alt+1)", "ノートを選択・編集（Alt+1）"))
                    .size(vec2(40.0, 24.0))
                    .grouped()
                    .selected(app.piano_tool == super::Tool::NoteSelect),
            )
            .clicked()
        {
            app.commit_lyric();
            app.piano_tool = super::Tool::NoteSelect;
            app.status.clear();
        }
        if ui
            .add(
                IconButton::new(Icon::NoteDraw, lang.text("创建音符（Alt+2）", "Draw notes (Alt+2)", "ノートを描画（Alt+2）"))
                    .size(vec2(40.0, 24.0))
                    .grouped()
                    .selected(app.piano_tool == super::Tool::NoteDraw),
            )
            .clicked()
        {
            app.commit_lyric();
            app.piano_tool = super::Tool::NoteDraw;
            app.status.clear();
        }
        if ui
            .add(
                IconButton::new(Icon::PitchAnchor, lang.text("编辑音高控制点（Alt+3）", "Edit pitch anchors (Alt+3)", "ピッチ制御点を編集（Alt+3）"))
                    .size(vec2(40.0, 24.0))
                    .grouped()
                    .selected(app.piano_tool == super::Tool::PitchAnchor),
            )
            .clicked()
        {
            app.commit_lyric();
            app.piano_tool = super::Tool::PitchAnchor;
            app.editor.selected.clear();
            app.show_pitch = true;
            app.status.clear();
        }
        if ui
            .add(
                IconButton::new(Icon::PitchDraw, lang.text("手绘音高（Alt+4）：左键绘制，右键擦除", "Draw pitch (Alt+4): left-draw, right-erase", "ピッチを手描き（Alt+4）：左で描画、右で消去"))
                    .size(vec2(40.0, 24.0))
                    .grouped()
                    .selected(app.piano_tool == super::Tool::PitchDraw),
            )
            .clicked()
        {
            app.commit_lyric();
            app.piano_tool = super::Tool::PitchDraw;
            app.show_pitch = true;
            app.status.clear();
        }
    });
}
