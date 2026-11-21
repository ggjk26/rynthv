use eframe::egui::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiLanguage {
    #[default]
    ZhCn,
    En,
    Ja,
}

impl UiLanguage {
    pub const ALL: [Self; 3] = [Self::ZhCn, Self::En, Self::Ja];

    /// Translate synthesized runtime status codes at the UI boundary without
    /// changing the language-neutral engine behavior or persisted project data.
    /// Translate UI diagnostics at presentation; preserve source errors for debugging.
    pub fn editor_message<'a>(self, message: &'a str) -> &'a str {
        match message {
            "无法编辑当前音素" => self.text("无法编辑当前音素", "Cannot edit the current phoneme", "現在の音素を編集できません"),
            "无法重置当前音素的强度和爆破设置" => self.text("无法重置当前音素的强度和爆破设置", "Cannot reset phoneme strength or burst settings", "音素の強さ／バースト設定をリセットできません"),
            "此音素没有可独立编辑的右边界" => self.text("此音素没有可独立编辑的右边界", "This phoneme has no independently editable end boundary", "この音素に独立して編集できる終端境界はありません"),
            "音素边界的顺序无效" => self.text("音素边界的顺序无效", "Phoneme boundary order is invalid", "音素境界の順序が無効です"),
            "音素源音符不存在" => self.text("音素源音符不存在", "Phoneme source note no longer exists", "音素の元ノートが存在しません"),
            "音素源音符起点超出范围" => self.text("音素源音符起点超出范围", "Phoneme source note start is out of range", "音素の元ノートの開始位置が範囲外です"),
            "当前音素没有可编辑的强度" => self.text("当前音素没有可编辑的强度", "Phoneme strength is not editable", "音素の強さは編集できません"),
            "当前音素没有可编辑的爆破位置" => self.text("当前音素没有可编辑的爆破位置", "Phoneme burst position is not editable", "音素のバースト位置は編集できません"),
            "当前音素没有可编辑的爆破强度" => self.text("当前音素没有可编辑的爆破强度", "Phoneme burst strength is not editable", "音素のバースト強度は編集できません"),
            "无法更新当前音素，请重新选择音符组" => self.text("无法更新当前音素，请重新选择音符组", "Cannot update phoneme; reselect the note group", "音素を更新できません。ノートグループを再選択してください"),
            "所选参数控制点的时间跨度超出可用范围。" => self.text("所选参数控制点的时间跨度超出可用范围。", "Selected control points exceed the available time range.", "選択した制御点が使用可能な時間範囲外です。"),
            "剪贴板中没有 RynthV 参数控制点，请先复制参数控制点。" => self.text("剪贴板中没有 RynthV 参数控制点，请先复制参数控制点。", "No RynthV control points in clipboard. Copy points first.", "クリップボードに RynthV 制御点がありません。先にコピーしてください。"),
            "参数控制点剪贴板的取值范围无效。" => self.text("参数控制点剪贴板的取值范围无效。", "Control-point clipboard range is invalid.", "制御点のクリップボード範囲が無効です。"),
            "参数控制点剪贴板的时间顺序或数值无效。" => self.text("参数控制点剪贴板的时间顺序或数值无效。", "Clipboard control-point timing or values are invalid.", "クリップボードの制御点の時刻または値が無効です。"),
            "所选参数控制点数据过大，无法复制。" => self.text("所选参数控制点数据过大，无法复制。", "Selected control-point data is too large to copy.", "制御点データが大きすぎてコピーできません。"),
            "剪贴板数据过大，无法粘贴参数控制点。" => self.text("剪贴板数据过大，无法粘贴参数控制点。", "Clipboard data is too large to paste control points.", "クリップボードのデータが大きすぎて貼り付けできません。"),
            _ => message,
        }
    }

    pub fn runtime_status<'a>(self, status: &'a str) -> &'a str {
        match status {
            "伴奏混音失败 · 查看渲染面板" => self.text("伴奏混音失败 · 查看渲染面板", "Instrumental mix failed · See render panel", "伴奏ミックスに失敗 · レンダリングパネルを確認"),
            "伴奏未就绪 · 查看伴奏轨" => self.text("伴奏未就绪 · 查看伴奏轨", "Instrumental audio not ready · See track", "伴奏の準備ができていません · トラックを確認"),
            "正在扫描声库目录" => self.text("正在扫描声库目录", "Scanning voice folders", "歌声フォルダーをスキャン中"),
            "正在加载声库" => self.text("正在加载声库", "Loading voicebank", "歌声を読み込み中"),
            "歌声需要导入模型上下文" => self.text("歌声需要导入模型上下文", "Voice requires model context import", "モデルコンテキストのインポートが必要です"),
            "正在准备播放位置的音频" => self.text("正在准备播放位置的音频", "Preparing audio at playhead", "再生位置の音声を準備中"),
            "正在合成歌声" => self.text("正在合成歌声", "Synthesizing vocals", "歌声を合成中"),
            "歌声合成失败 · 查看渲染面板" => self.text("歌声合成失败 · 查看渲染面板", "Vocal synthesis failed · See render panel", "歌声合成に失敗 · レンダリングパネルを確認"),
            "音频已就绪" => self.text("音频已就绪", "Audio ready", "音声の準備完了"),
            "等待音符" => self.text("等待音符", "Waiting for notes", "ノートを待機中"),
            "正在准备合成环境" => self.text("正在准备合成环境", "Preparing synthesis backend", "合成環境を準備中"),
            "音轨尚未指定歌声" => self.text("音轨尚未指定歌声", "No voice assigned to track", "トラックに歌声が設定されていません"),
            _ => status,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ZhCn => "简体中文",
            Self::En => "English",
            Self::Ja => "日本語",
        }
    }

    pub fn text(self, zh: &'static str, en: &'static str, ja: &'static str) -> &'static str {
        match self {
            Self::ZhCn => zh,
            Self::En => en,
            Self::Ja => ja,
        }
    }
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    #[serde(default)]
    pub language: UiLanguage,
    pub zoom: f32,
    pub show_status_bar: bool,
    pub control_hints: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: UiLanguage::default(),
            zoom: 1.0,
            show_status_bar: true,
            control_hints: false,
        }
    }
}

impl Preferences {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(format!("无法读取界面设置：{error}")),
        };
        let value: Self =
            serde_json::from_slice(&bytes).map_err(|e| format!("无法读取界面设置：{e}"))?;
        if !value.zoom.is_finite() || !(0.75..=2.0).contains(&value.zoom) {
            return Err("界面缩放必须为 75% 至 200%".into());
        }
        Ok(value)
    }

    pub fn apply(&self, ctx: &Context) {
        ctx.set_zoom_factor(self.zoom);
        ctx.style_mut_of(eframe::egui::Theme::Dark, |style| {
            style.explanation_tooltips = self.control_hints
        });
    }
}
