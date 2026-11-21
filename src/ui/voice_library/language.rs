use super::{error_label, widgets};
use crate::{
    app::App,
    model::{Language, database_language},
    ui::theme::*,
};
use eframe::egui::*;

pub(super) fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.preferences.language;
    let Some(config) = app
        .voice_inventory()
        .and_then(|voice| voice.configuration.as_ref())
    else {
        widgets::field(ui, lang.text("未加载歌声", "No voice loaded", "歌声が読み込まれていません"), true, lang.text("加载声库后可选择其支持的语言", "Load a voice to select its supported languages", "歌声を読み込むと対応言語を選べます"));
        return;
    };
    let Some(native) = Language::parse(&config.default_language) else {
        error_label(
            ui,
            format!("{}: {}", lang.text("无法识别声库默认语言", "Unrecognized default voice language", "歌声の既定言語を認識できません"), config.default_language),
        );
        return;
    };
    let languages: Vec<_> = config
        .supported_languages
        .iter()
        .filter_map(|id| Language::parse(id))
        .collect();
    let track = &app.editor.current_track().main_ref;
    let target = app.editor.active_ref().unwrap_or(track);
    let original = target
        .database
        .get("languageOverride")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let mut selected = original.to_owned();
    let inherited = if target.id != track.id {
        database_language(&track.database, native)
    } else {
        Ok(native)
    };
    let mut default_database = target.database.clone();
    if let Some(database) = default_database.as_object_mut() {
        database.insert("languageOverride".into(), "".into());
    }
    let default = inherited.and_then(|inherited| database_language(&default_database, inherited));
    let default_label = match &default {
        Ok(language) => format!("{} {}", lang.language_label(*language), lang.text("（默认）", "(default)", "（既定）")),
        Err(_) => lang.text("默认语言不可用", "Default language unavailable", "既定の言語は利用できません").into(),
    };
    let selected_label = if selected.is_empty() {
        default_label.clone()
    } else {
        Language::parse(&selected)
            .map_or_else(|| selected.clone(), |language| lang.language_label(language).to_owned())
    };
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 28.0;
        ui.spacing_mut().button_padding = vec2(8.0, 5.0);
        ui.visuals_mut().widgets.inactive.bg_fill = BORDER;
        ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::same(4);
        ComboBox::from_id_salt("voice-language")
            .width(ui.available_width())
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                ui.spacing_mut().interact_size.y = 27.0;
                let default_supported = default
                    .as_ref()
                    .is_ok_and(|language| languages.contains(language));
                ui.add_enabled_ui(default_supported, |ui| {
                    ui.selectable_value(&mut selected, String::new(), &default_label);
                });
                for language in &languages {
                    ui.selectable_value(&mut selected, language.id().to_owned(), lang.language_label(*language))
                        .on_hover_text(input_hint(*language, lang));
                }
            })
            .response
            .on_hover_text(if app.editor.active_ref().is_some() {
                lang.text("应用于当前音符组；默认项使用其继承的歌声语言", "Applies to the selected group; default inherits its voice language", "選択したグループに適用。既定では歌声の言語を継承します")
            } else {
                lang.text("应用于当前音轨，新建音符组继承此设置", "Applies to the current track; new groups inherit it", "現在のトラックに適用。新しいグループはこの設定を継承します")
            });
    });
    if selected != original {
        app.commit_lyric();
        if let Err(error) = app.editor.set_voice_language(Language::parse(&selected)) {
            app.error = Some(error);
        }
    }
}

fn input_hint(language: Language, lang: crate::ui::settings::UiLanguage) -> &'static str {
    match language {
        Language::Mandarin => lang.text("可输入汉字、拼音或显式音素；多音字可用拼音指定读音", "Enter Chinese characters, pinyin, or explicit phonemes; pinyin disambiguates polyphonic characters", "漢字・ピンイン・明示的な音素を入力できます。多音字にはピンインを指定してください"),
        Language::English => lang.text("支持发音词典覆盖的单词；生词或多读音单词需填写显式音素", "Words covered by the pronunciation dictionary are supported; specify explicit phonemes for unknown or ambiguous words", "発音辞書にある単語に対応します。未知語や複数の読みは音素を指定してください"),
        Language::Japanese => lang.text("可输入假名、罗马音或词典覆盖的汉字；多读音词可用假名指定", "Enter kana, romaji, or dictionary-covered kanji; specify kana for ambiguous readings", "仮名・ローマ字・辞書にある漢字を入力できます。複数の読みには仮名を指定してください"),
        Language::Spanish => lang.text("支持词典覆盖的音节与字母组合；未覆盖的读音需填写显式音素", "Dictionary-covered syllables and letter combinations are supported; enter phonemes for others", "辞書に登録された音節・文字列に対応します。それ以外は音素を指定してください"),
        Language::Cantonese => lang.text("可输入粤拼或词典覆盖的汉字；多读音字可用粤拼指定", "Enter Jyutping or dictionary-covered Chinese characters; Jyutping selects ambiguous readings", "粤拼または辞書にある漢字を入力できます。複数の読みは粤拼で指定してください"),
        Language::Korean => lang.text("当前支持按音节分隔的罗马音或显式音素；韩文整句自动注音尚未接入", "Use syllable-separated romanization or explicit phonemes; full Hangul text conversion is unavailable", "音節で区切ったローマ字または音素を使用してください。ハングル全文の自動変換は未対応です"),
    }
}
