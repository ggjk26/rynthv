use crate::{
    app::{App, MidiImportMode, MidiImportPhase},
    midi::MidiTrack,
    ui::{settings::UiLanguage, theme::*},
};
use eframe::egui::*;

pub fn show(app: &mut App, ctx: &Context) {
    let lang = app.preferences.language;
    let Some(mut import) = app.midi_import.take() else {
        return;
    };
    let mut cancel = false;
    let mut submit = false;
    let mut can_import = false;
    let width = (ctx.content_rect().width() - 96.0).clamp(250.0, 470.0);
    let height = (ctx.content_rect().height() - 420.0).clamp(60.0, 280.0);
    let modal = Modal::new(Id::new("midi-import"))
        .frame(
            Frame::new()
                .fill(Color32::from_rgb(55, 55, 57))
                .stroke(Stroke::new(1.0, Color32::from_rgb(73, 72, 76)))
                .corner_radius(4)
                .inner_margin(24),
        )
        .backdrop_color(Color32::from_black_alpha(30))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 12.0;
            ui.label(
                RichText::new(lang.text("导入 MIDI", "Import MIDI", "MIDI をインポート"))
                    .font(FontId::new(18.0, FontFamily::Name("SectionTitle".into()))),
            );
            ui.label(
                RichText::new(
                    import
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                )
                .size(11.0)
                .color(MUTED),
            )
            .on_hover_text(import.path.display().to_string());
            match &mut import.phase {
                MidiImportPhase::Loading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(lang.text("正在读取 MIDI 音轨…", "Reading MIDI tracks…", "MIDI トラックを読み込み中…"));
                    });
                }
                MidiImportPhase::Failed(error) => {
                    ui.label(RichText::new(error.as_str()).color(Color32::from_rgb(232, 147, 127)));
                }
                MidiImportPhase::Ready { file, selected } => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(lang.text("要导入的音轨", "Tracks to import", "インポートするトラック")).color(MUTED));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button(lang.text("取消全选", "Deselect all", "すべて選択解除")).clicked() {
                                selected.fill(false);
                                import.validation_error = None;
                            }
                            if ui.small_button(lang.text("全选", "Select all", "すべて選択")).clicked() {
                                import.validation_error = None;
                                for (selected, track) in selected.iter_mut().zip(&file.tracks) {
                                    *selected = !track.notes.is_empty();
                                }
                            }
                        });
                    });
                    ScrollArea::vertical()
                        .id_salt("midi-import-tracks")
                        .max_height(height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (track, selected) in file.tracks.iter().zip(selected.iter_mut()) {
                                if ui
                                    .add_enabled(
                                        !track.notes.is_empty(),
                                        Checkbox::new(selected, track_label(track, lang)),
                                    )
                                    .changed()
                                {
                                    import.validation_error = None;
                                }
                            }
                        });
                    can_import = file
                        .tracks
                        .iter()
                        .zip(selected.iter())
                        .any(|(track, &selected)| selected && !track.notes.is_empty());
                    ui.label(
                        RichText::new(match import.mode {
                            MidiImportMode::NewProject => lang.text("新建工程，使用 MIDI 的曲速和拍号。", "Create a new project with the MIDI tempo and time signatures.", "MIDI のテンポと拍子を使用して新規プロジェクトを作成します。"),
                            MidiImportMode::Tracks => lang.text("从工程起点追加音轨，保留当前曲速和拍号。", "Append tracks from project start; keep current tempo and time signature.", "プロジェクトの先頭にトラックを追加し、現在のテンポと拍子を維持します。"),
                        })
                        .size(11.0)
                        .color(MUTED),
                    );
                    ui.label(
                        RichText::new(lang.text("重叠音符会拆分为独立声部。", "Overlapping notes will be split into separate parts.", "重なったノートは別のパートに分割されます。"))
                            .size(11.0)
                            .color(MUTED),
                    );
                    if !file.warnings.is_empty() {
                        CollapsingHeader::new(format!("{} ({})", lang.text("导入提示", "Import warnings", "インポートの注意"), file.warnings.len())).show(
                            ui,
                            |ui| {
                                ScrollArea::vertical()
                                    .id_salt("midi-import-warnings")
                                    .max_height(100.0)
                                    .show(ui, |ui| {
                                        for warning in &file.warnings {
                                            ui.label(
                                                RichText::new(warning).size(11.0).color(MUTED),
                                            );
                                        }
                                    });
                            },
                        );
                    }
                    if file.tracks.iter().all(|track| track.notes.is_empty()) {
                        ui.label(lang.text("文件中没有可导入的音符。", "No importable notes in this file.", "インポートできるノートがありません。"));
                    }
                }
            }
            if let Some(error) = &import.validation_error {
                ui.label(RichText::new(error).color(Color32::from_rgb(232, 147, 127)));
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let size = vec2((ui.available_width() - 10.0) / 2.0, 34.0);
                cancel = ui
                    .add_sized(size, Button::new(lang.text("取消", "Cancel", "キャンセル")).fill(BUTTON_HOVER))
                    .clicked();
                ui.add_space(2.0);
                submit = ui
                    .add_enabled(
                        can_import,
                        Button::new(RichText::new(lang.text("导入", "Import", "インポート")).color(CANVAS))
                            .fill(GREEN)
                            .min_size(size),
                    )
                    .clicked();
            });
        });
    if modal.is_top_modal && !modal.any_popup_open {
        cancel |= ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        submit |= can_import && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    }
    if cancel {
        return;
    }
    if submit && can_import {
        if let Err(error) = app.apply_midi_import(&import) {
            import.validation_error = Some(error);
            app.midi_import = Some(import);
        }
    } else {
        app.midi_import = Some(import);
    }
}

fn track_label(track: &MidiTrack, lang: UiLanguage) -> String {
    let mut details = format!("{} {}", track.notes.len(), lang.text("个音符", "notes", "個のノート"));
    if let Some(low) = track.notes.iter().map(|note| note.pitch).min() {
        let high = track.notes.iter().map(|note| note.pitch).max().unwrap();
        details.push_str(&format!("，{} – {}", pitch_name(low), pitch_name(high)));
    }
    if !track.instrument.is_empty() {
        details.push_str(&format!("，{}", track.instrument));
    }
    format!("{}（{details}）", track.name)
}

fn pitch_name(pitch: i32) -> String {
    let name = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ][pitch.rem_euclid(12) as usize];
    format!("{name}{}", pitch.div_euclid(12) - 1)
}
