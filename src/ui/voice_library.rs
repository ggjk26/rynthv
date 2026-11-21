mod artwork;
mod context;
mod language;
mod management;
mod panel;
mod parameters;
mod paths;
mod selector;
mod widgets;

use super::theme::*;
use crate::{
    app::App,
    synthesis::voice::{VoiceCatalogEntry, VoiceFamily},
};
use eframe::egui::*;

pub use management::dialogs;
pub use panel::show;
pub(crate) use parameters::end_frame as end_parameter_frame;
pub use paths::settings;
pub use selector::popup;

fn selected_entry(app: &App) -> Option<&VoiceCatalogEntry> {
    app.synthesis.library.entry(&app.voice_selection()?)
}

fn family_name(entry: &VoiceCatalogEntry, lang: crate::ui::settings::UiLanguage) -> &'static str {
    if entry.database_id.starts_with("svflat:") {
        // Flat JSON voices and extracted SFPK binary packages have different
        // resource layouts. The catalog keeps both metadata-only for now.
        return if matches!(&entry.availability,
            crate::synthesis::voice::VoiceAvailability::Unsupported(reason)
                if reason.starts_with("Flat SFPK")
                    || reason.starts_with("Flat config.json")
        ) {
            "SV Flat · SFPK"
        } else {
            "SV Flat · NOFS-JSON"
        };
    }
    match entry.family {
        Some(VoiceFamily::Sv1) => lang.text("SV1 原生", "Native SV1", "SV1 ネイティブ"),
        Some(VoiceFamily::Sv2Compatible) => lang.text("SV2 · 版本 1 兼容", "SV2 · version 1 compatible", "SV2 · バージョン1互換"),
        Some(VoiceFamily::Sv2) => "SV2",
        None => lang.text("未知格式", "Unknown format", "不明な形式"),
    }
}

fn text(ui: &mut Ui, value: impl Into<WidgetText>) {
    ui.add(Label::new(value).selectable(false).wrap());
}

fn weak(ui: &mut Ui, value: impl Into<String>) {
    text(ui, RichText::new(value).color(MUTED));
}

fn error_label(ui: &mut Ui, value: impl Into<String>) {
    text(
        ui,
        RichText::new(value).color(Color32::from_rgb(218, 142, 126)),
    );
}

fn language_name(language: &str, lang: crate::ui::settings::UiLanguage) -> &str {
    match language {
        "mandarin" | "mandarin-xsampa" | "zh" | "ZH" => lang.text("普通话", "Mandarin", "中国語（普通話）"),
        "english" | "english-arpabet" | "en" | "EN" => lang.text("英语", "English", "英語"),
        "japanese" | "japanese-romaji" | "ja" | "JA" => lang.text("日语", "Japanese", "日本語"),
        "spanish" | "spanish-xsampa" | "es" | "ES" => lang.text("西班牙语", "Spanish", "スペイン語"),
        "cantonese" | "cantonese-xsampa" | "yue" | "YUE" => lang.text("粤语", "Cantonese", "広東語"),
        "korean" | "korean-xsampa" | "ko" | "KO" => lang.text("韩语", "Korean", "韓国語"),
        other => other,
    }
}

/// Translate known voicebank scan and dependency messages at presentation.
/// The original technical diagnostic remains intact for unidentified formats.
pub(super) fn localized_diagnostic(
    message: &str,
    lang: crate::ui::settings::UiLanguage,
) -> String {
    if lang == crate::ui::settings::UiLanguage::ZhCn {
        return message.to_owned();
    }
    const TERMS: &[(&str, &str, &str)] = &[
        ("SV Flat 声库尚不支持合成", "SV Flat synthesis is not supported yet", "SV Flat の歌声合成は未対応です"),
        ("已找到文件", "file found", "ファイルを検出"),
        ("未找到文件", "file not found", "ファイルが見つかりません"),
        ("未定位到文件", "file not located", "ファイルが見つかりません"),
        ("未检查：未配置模型目录", "not checked: model directory is not configured", "未確認：モデルフォルダーが未設定"),
        ("模型目录未配置", "model directory not configured", "モデルフォルダーが未設定"),
        ("未配置 Flat 模型目录", "Flat model directory not configured", "Flat モデルフォルダーが未設定"),
        ("未配置模型目录", "model directory not configured", "モデルフォルダーが未設定"),
        ("模型检查", "models checked", "モデル確認"),
        ("依赖模型", "dependency models", "依存モデル"),
        ("已找到", "found", "検出"),
        ("声码器及原生 Flat 音色条件推理尚未实现，暂不能试听/导出", "Native Flat voice conditioning and vocoder inference are not implemented; playback/export is unavailable", "Flat の歌声条件付けとボコーダー推論は未実装のため、試聴／書き出しはできません"),
        ("info.nofs 二进制格式未解码；暂不能试听/导出", "Binary info.nofs is not decoded; playback/export is unavailable", "info.nofs のバイナリは未解析のため、試聴／書き出しはできません"),
        ("Flat config.json 解析失败", "Failed to parse Flat config.json", "Flat config.json の解析に失敗しました"),
        ("Flat NOFS-JSON 解析失败", "Failed to parse Flat NOFS-JSON", "Flat NOFS-JSON の解析に失敗しました"),
        ("config.json 未内嵌 Flat 模型清单", "config.json has no Flat model list", "config.json に Flat モデル一覧がありません"),
        ("包含不支持的角色字符", "contains unsupported role characters", "対応していない役割名の文字が含まれます"),
        ("不是有效的本地模型标识，已跳过文件定位", "is not a valid local model ID; lookup skipped", "ローカルモデル ID が無効なため、検索を省略しました"),
        ("数量不一致", "counts do not match", "個数が一致しません"),
        ("缺少本地", "Missing local ", "ローカルの"),
        ("模型，当前安装内容无法在本机合成", " model(s); this installation cannot synthesize", "モデルが不足しているため合成できません"),
        ("声库配置版本与目录版本不一致", "Voice configuration version differs from folder revision", "歌声設定のバージョンがフォルダーのバージョンと一致しません"),
        ("相同声库版本存在多个文件，请移除重复项后刷新", "Duplicate voice files for the same revision; remove duplicates and refresh", "同じバージョンの歌声ファイルが重複しています。重複を削除して更新してください"),
        ("内嵌 Duration 不兼容", "Embedded Duration model is incompatible", "内蔵 Duration モデルに互換性がありません"),
        ("本地编译嵌入（容器及资源标识已校验）", "embedded at local build (container and resource ID verified)", "ローカルビルドで内蔵（コンテナーとリソース ID を検証済み）"),
        ("未声明 styles 音色向量", "no styles vectors declared", "styles ベクトルが定義されていません"),
        ("尚未配置 Flat 模型目录，未进行模型文件检查", "Flat model directory has not been configured; model files were not checked", "Flat モデルフォルダーが未設定のため、ファイルは未確認です"),
        ("Flat 模型目录不存在", "Flat model directory does not exist", "Flat モデルフォルダーが存在しません"),
        ("未定位到", "Could not find ", "見つかりません："),
        ("仅检查已配置目录", "searched only configured directories", "設定済みフォルダーのみ検索"),
        ("未指定；需要根据具体 Flat 版本确认默认值", "not specified; default depends on the Flat version", "未指定：既定値は Flat のバージョンによります"),
        ("info.nofs 是未解析的二进制资源；仅解析相邻 config.json", "info.nofs is opaque binary metadata; only the neighboring config.json was parsed", "info.nofs は未解析のバイナリデータです。隣接する config.json のみ解析しました"),
        ("声库加载线程意外终止", "Voice loading worker stopped unexpectedly", "歌声読み込み処理が予期せず終了しました"),
        ("合成线程意外终止", "Synthesis worker stopped unexpectedly", "合成処理が予期せず終了しました"),
        ("无法枚举声库", "Cannot enumerate voicebank ", "歌声フォルダーを列挙できません："),
        ("无法读取声库", "Cannot read voicebank ", "歌声フォルダーを読み込めません："),
        ("无法识别声库", "Cannot identify voicebank ", "歌声を識別できません："),
    ];
    let mut localized = message.to_owned();
    for &(zh, en, ja) in TERMS {
        localized = localized.replace(
            zh,
            if lang == crate::ui::settings::UiLanguage::En { en } else { ja },
        );
    }
    localized
}
