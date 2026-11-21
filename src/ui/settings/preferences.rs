use eframe::egui::Context;
use crate::model::Language;
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
    pub fn language_label(self, language: Language) -> &'static str {
        match language {
            Language::English => self.text("英语", "English", "英語"),
            Language::Mandarin => self.text("汉语普通话", "Mandarin Chinese", "中国語（普通話）"),
            Language::Japanese => self.text("日语", "Japanese", "日本語"),
            Language::Spanish => self.text("西班牙语", "Spanish", "スペイン語"),
            Language::Cantonese => self.text("汉语广东话", "Cantonese Chinese", "広東語"),
            Language::Korean => self.text("韩语", "Korean", "韓国語"),
        }
    }

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
            "请先选择音符组" => self.text("请先选择音符组", "Select a note group first", "先にノートグループを選択してください"),
            "唱法参数名称不能为空" => self.text("唱法参数名称不能为空", "Vocal mode parameter name cannot be empty", "歌唱スタイルのパラメーター名を空にできません"),
            "音符剪贴板的起始时间无效。" => self.text("音符剪贴板的起始时间无效。", "Invalid note clipboard start time.", "ノートのクリップボード開始時刻が無効です。"),
            "音符剪贴板包含无效的起始时间或时长。" => self.text("音符剪贴板包含无效的起始时间或时长。", "Note clipboard contains invalid start times or durations.", "クリップボードのノートの開始時刻または長さが無効です。"),
            "音符剪贴板的音高超出 MIDI 范围。" => self.text("音符剪贴板的音高超出 MIDI 范围。", "Note pitch in clipboard exceeds the MIDI range.", "クリップボードのノート音高が MIDI 範囲外です。"),
            "音符剪贴板的时间超出可用范围。" => self.text("音符剪贴板的时间超出可用范围。", "Note clipboard time is out of range.", "クリップボードのノート時刻が範囲外です。"),
            "所选音符数据过大，无法复制。" => self.text("所选音符数据过大，无法复制。", "Selected note data is too large to copy.", "選択したノートのデータが大きすぎてコピーできません。"),
            "剪贴板数据过大，无法粘贴音符。" => self.text("剪贴板数据过大，无法粘贴音符。", "Clipboard data is too large to paste notes.", "クリップボードのデータが大きすぎてノートを貼り付けられません。"),
            "剪贴板中没有 RynthV 音符，请先复制音符。" => self.text("剪贴板中没有 RynthV 音符，请先复制音符。", "Clipboard has no RynthV notes. Copy notes first.", "クリップボードに RynthV ノートがありません。先にコピーしてください。"),
            "请先选择音符组。" => self.text("请先选择音符组。", "Select a note group first.", "先にノートグループを選択してください。"),
            "请先选择音符。" => self.text("请先选择音符。", "Select notes first.", "先にノートを選択してください。"),
            "所选音符的结束时间超出可用范围。" => self.text("所选音符的结束时间超出可用范围。", "Selected notes end outside the supported range.", "選択したノートの終了時刻が範囲外です。"),
            "所选音符的时间跨度超出可用范围。" => self.text("所选音符的时间跨度超出可用范围。", "Selected notes span an unsupported time range.", "選択したノートの時間範囲が対応範囲外です。"),
            "所选音符的音高超出可用范围。" => self.text("所选音符的音高超出可用范围。", "Selected notes exceed the supported pitch range.", "選択したノートの音高が対応範囲外です。"),
            "粘贴位置不能早于项目起点。" => self.text("粘贴位置不能早于项目起点。", "Cannot paste before the project start.", "プロジェクトの開始前には貼り付けられません。"),
            "请先选择歌声音轨再粘贴音符。" => self.text("请先选择歌声音轨再粘贴音符。", "Select a vocal track before pasting notes.", "ノートを貼り付ける前に歌声トラックを選択してください。"),
            "粘贴后的音符时间超出可用范围。" => self.text("粘贴后的音符时间超出可用范围。", "Pasted note times are out of range.", "貼り付け後のノート時刻が範囲外です。"),
            "粘贴后的音符结束时间超出可用范围。" => self.text("粘贴后的音符结束时间超出可用范围。", "Pasted note ends are out of range.", "貼り付け後のノート終了時刻が範囲外です。"),
            "粘贴后的组内时间超出可用范围。" => self.text("粘贴后的组内时间超出可用范围。", "Pasted group-local times are out of range.", "貼り付け後のグループ内時刻が範囲外です。"),
            "粘贴后的组内结束时间超出可用范围。" => self.text("粘贴后的组内结束时间超出可用范围。", "Pasted group-local end times are out of range.", "貼り付け後のグループ内終了時刻が範囲外です。"),
            "目标音符组的移调使粘贴音高超出 MIDI 范围。" => self.text("目标音符组的移调使粘贴音高超出 MIDI 范围。", "The destination group's transposition puts pasted notes outside MIDI pitch range.", "貼り付け先のグループの移調によってノート音高が MIDI 範囲外になります。"),
            "目标音符组不存在，无法粘贴。" => self.text("目标音符组不存在，无法粘贴。", "Destination note group not found; cannot paste.", "貼り付け先のノートグループが見つかりません。"),
            "请输入词典名称" => self.text("请输入词典名称", "Enter a dictionary name", "辞書名を入力してください"),
            "已有同名词典" => self.text("已有同名词典", "A dictionary with this name already exists", "同じ名前の辞書が既にあります"),
            "请输入字词" => self.text("请输入字词", "Enter a word", "単語を入力してください"),
            "请输入音素，以空格分隔" => self.text("请输入音素，以空格分隔", "Enter phonemes separated by spaces", "音素を空白区切りで入力してください"),
            "请先新建或选择词典" => self.text("请先新建或选择词典", "Create or select a dictionary first", "先に辞書を作成するか選択してください"),
            "请选择至少一条包含音符的 MIDI 音轨" => self.text("请选择至少一条包含音符的 MIDI 音轨", "Select at least one MIDI track containing notes", "ノートのある MIDI トラックを1つ以上選択してください"),
            "所选 MIDI 音轨不存在，请重新打开导入窗口" => self.text("所选 MIDI 音轨不存在，请重新打开导入窗口", "Selected MIDI track no longer exists; reopen the import window", "選択した MIDI トラックが存在しません。インポート画面を開き直してください"),
            "所选 MIDI 音轨没有可导入的音符" => self.text("所选 MIDI 音轨没有可导入的音符", "Selected MIDI track has no importable notes", "選択した MIDI トラックにインポート可能なノートがありません"),
            "MIDI 音符的时间或音高超出支持范围" => self.text("MIDI 音符的时间或音高超出支持范围", "MIDI note time or pitch is out of supported range", "MIDI ノートの時刻または音高が対応範囲外です"),
            "当前音符组中没有此音素的源音符" => self.text("当前音符组中没有此音素的源音符", "Source note for this phoneme is not in the current group", "現在のグループにこの音素の元ノートがありません"),
            "请先完成当前编辑" => self.text("请先完成当前编辑", "Finish the current edit first", "現在の編集を先に完了してください"),
            "无法重置当前音素" => self.text("无法重置当前音素", "Cannot reset the current phoneme", "現在の音素をリセットできません"),
            "重录采样编号已用尽" => self.text("重录采样编号已用尽", "Retake IDs are exhausted", "リテイク ID を使い切りました"),
            "曲速位置必须非负，BPM 必须是大于 0 的有限数值" => self.text("曲速位置必须非负，BPM 必须是大于 0 的有限数值", "Tempo position must be nonnegative and BPM must be a finite positive value", "テンポ位置は0以上、BPM は正の有限値である必要があります"),
            "拍号分子须为 1–64，分母须为 1、2、4、8、16、32 或 64" => self.text("拍号分子须为 1–64，分母须为 1、2、4、8、16、32 或 64", "Time signature numerator must be 1–64; denominator must be 1, 2, 4, 8, 16, 32 or 64", "拍子の分子は1〜64、分母は1・2・4・8・16・32・64である必要があります"),
            "目标音轨不存在" => self.text("目标音轨不存在", "Target track does not exist", "対象トラックが存在しません"),
            "伴奏轨不能设置歌手" => self.text("伴奏轨不能设置歌手", "Cannot assign a singer to an instrumental track", "伴奏トラックに歌手は設定できません"),
            "音符组不存在或 database 字段无效" => self.text("音符组不存在或 database 字段无效", "Note group is missing or its database field is invalid", "ノートグループが存在しないか database フィールドが無効です"),
            "所选轨道包含无效或伴奏轨道" => self.text("所选轨道包含无效或伴奏轨道", "Selection contains invalid or instrumental tracks", "選択に無効なトラックまたは伴奏トラックが含まれます"),
            "所选音轨的歌声 database 字段必须是对象" => self.text("所选音轨的歌声 database 字段必须是对象", "Selected track voice database field must be an object", "選択したトラックの歌声 database フィールドはオブジェクトである必要があります"),
            "音轨的歌声 database 字段必须是对象" => self.text("音轨的歌声 database 字段必须是对象", "Track voice database field must be an object", "トラックの歌声 database フィールドはオブジェクトである必要があります"),
            "歌声的 database 字段必须是对象" => self.text("歌声的 database 字段必须是对象", "Voice database field must be an object", "歌声の database フィールドはオブジェクトである必要があります"),
            "伴奏轨不能设置唱法" => self.text("伴奏轨不能设置唱法", "Cannot set vocal styles on an instrumental track", "伴奏トラックに歌唱スタイルは設定できません"),
            "保存的工作区尺寸或结构无效，已重置布局" => self.text("保存的工作区尺寸或结构无效，已重置布局", "Saved workspace layout is invalid and has been reset", "保存されたワークスペース配置が無効なためリセットしました"),
            "界面缩放必须为 75% 至 200%" => self.text("界面缩放必须为 75% 至 200%", "Interface scale must be between 75% and 200%", "画面拡大率は75%〜200%である必要があります"),
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
