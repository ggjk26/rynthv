mod interaction;

pub(crate) use interaction::end_frame;

use super::{error_label, selected_entry, weak, widgets};
use crate::{
    app::App,
    editor::{VoiceEditParameter, VoiceParameterState},
    model::{VocalModeValues, VoiceParameter},
    synthesis::voice::VoiceFamily,
};
use eframe::egui::*;

pub(super) fn styles(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    interaction::prepare(app, ui.ctx());
    let Some(styles) = app
        .voice_inventory()
        .and_then(|voice| voice.configuration.as_ref())
        .map(|configuration| configuration.styles.clone())
    else {
        weak(ui, lang.text("加载歌声后显示支持的唱法", "Load a voice to see available vocal styles", "歌声を読み込むと歌唱スタイルが表示されます"));
        return;
    };
    let mut current = state(app, ui);
    let family = selected_entry(app).and_then(|entry| entry.family);
    let components = match family {
        Some(VoiceFamily::Sv2) => [true; 3],
        Some(VoiceFamily::Sv1 | VoiceFamily::Sv2Compatible) => [false, true, false],
        None => [false; 3],
    };
    if family.is_some()
        && let Some(current_state) = &current
        && unsupported_styles(app, ui, current_state, &styles, components)
    {
        current = state(app, ui);
    }
    if styles.is_empty() {
        weak(ui, lang.text("此歌声没有可用的唱法", "No vocal styles available for this voice", "この歌声には使用可能な歌唱スタイルがありません"));
        return;
    }
    let enabled = current.is_some() && interaction::editable(app, ui);
    let target = target_id(app);
    ui.push_id(target, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for name in styles {
            let mut values = current
                .as_ref()
                .map_or(VocalModeValues::DEFAULT, |state| state.values.style(&name))
                .as_array();
            let id = ui.make_persistent_id(("vocal-mode", &name));
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    widgets::style(ui, &name, &mut values, components, lang)
                })
                .inner;
            interaction::apply(
                app,
                ui.ctx(),
                id,
                response,
                VoiceEditParameter::VocalMode(name.clone()),
                |editor| editor.update_vocal_mode(&name, VocalModeValues::from_array(values)),
            );
        }
    });
}

fn unsupported_styles(
    app: &mut App,
    ui: &mut Ui,
    state: &VoiceParameterState,
    styles: &[String],
    components: [bool; 3],
) -> bool {
    let lang = app.preferences.language;
    let unsupported = state.values.vocal_modes.iter().any(|(name, values)| {
        let known = styles.contains(name);
        values
            .as_array()
            .into_iter()
            .zip(components)
            .any(|(value, enabled)| value != 0.0 && (!known || !enabled))
    });
    if !unsupported {
        return false;
    }
    weak(ui, lang.text("当前设置包含此歌声不支持的唱法参数。", "Current settings contain styles unsupported by this voice.", "この歌声で使用できないスタイル設定が含まれています。"));
    let enabled = interaction::editable(app, ui) && !app.editor.is_editing();
    let clicked = ui
        .add_enabled(enabled, Button::new(lang.text("重置不支持的唱法", "Reset unsupported styles", "未対応スタイルをリセット")))
        .on_hover_text(lang.text("仅重置不支持的唱法设置，保留其余参数；可撤销。", "Reset only unsupported styles; other parameters remain. This can be undone.", "未対応のスタイルのみリセットします。他の設定は維持され、元に戻せます。"))
        .clicked();
    ui.add_space(8.0);
    if !clicked {
        return false;
    }
    app.commit_lyric();
    match app.editor.reset_unsupported_vocal_modes(styles, components) {
        Ok(()) => {
            app.status.clear();
            ui.ctx().request_repaint();
            true
        }
        Err(error) => {
            app.error = Some(error);
            false
        }
    }
}

pub(super) fn scalars(app: &mut App, ui: &mut Ui) {
    sliders(app, ui, &VoiceParameter::BASIC);
}

pub(super) fn tuning(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let summary = match app.editor.voice_parameter_state() {
        Ok(Some(state)) => {
            let key_shift = state.values.scalar(VoiceParameter::KeyShift).unwrap_or(0.0);
            let key = (60.0 + key_shift) as usize % 12;
            let names = [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
            ];
            let frequency = 440.0 * (state.values.tuning_semitones() / 12.0).exp2();
            let frequency = format!("{frequency:.2}");
            let frequency = frequency.trim_end_matches('0').trim_end_matches('.');
            format!("{}: {}, A4 = {frequency} Hz", lang.text("调性", "Key", "調性"), names[key])
        }
        _ => format!("{}: —, A4 = — Hz", lang.text("调性", "Key", "調性")),
    };
    widgets::readout(ui, &summary);
    ui.add_space(10.0);
    sliders(app, ui, &VoiceParameter::TUNING);
}

fn sliders(app: &mut App, ui: &mut Ui, parameters: &[VoiceParameter]) {
    let lang = app.preferences.language;
    interaction::prepare(app, ui.ctx());
    let state = state(app, ui);
    let enabled = state.is_some() && interaction::editable(app, ui);
    let target = target_id(app);
    ui.push_id(target, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for &parameter in parameters {
            let mut value = state
                .as_ref()
                .and_then(|state| state.values.scalar(parameter))
                .unwrap_or_else(|| parameter.default_value());
            let id = ui.make_persistent_id(parameter.key());
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    widgets::parameter_slider(ui, parameter, &mut value, lang)
                })
                .inner;
            interaction::apply(
                app,
                ui.ctx(),
                id,
                response,
                VoiceEditParameter::Scalar(parameter),
                |editor| editor.update_voice_parameter(parameter, value),
            );
        }
    });
}

fn state(app: &App, ui: &mut Ui) -> Option<VoiceParameterState> {
    let lang = app.preferences.language;
    match app.editor.voice_parameter_state() {
        Ok(Some(state)) => Some(state),
        Ok(None) => {
            weak(ui, lang.text("请选择歌声音轨以调整参数", "Select a vocal track to adjust parameters", "歌声トラックを選択してパラメーターを調整してください"));
            None
        }
        Err(error) => {
            error_label(ui, format!("{}: {error}", lang.text("无法读取当前歌声参数", "Unable to read voice parameters", "歌声パラメーターを読み込めません")));
            None
        }
    }
}

fn target_id(app: &App) -> Id {
    Id::new((
        "voice-parameters",
        app.editor.track,
        app.editor.reference,
        app.voice_selection()
            .map(|selection| (selection.database_id, selection.revision)),
    ))
}
