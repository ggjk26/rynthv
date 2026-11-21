use crate::ui::settings::UiLanguage;
use serde::{Deserialize, Serialize};

const CLIPBOARD_KIND: &str = "RynthV parameter points";
const MAX_CLIPBOARD_BYTES: usize = 4 * 1024 * 1024;
const MAX_CLIPBOARD_POINTS: usize = 100_000;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParameterClipboard {
    kind: String,
    pub(super) range: (f64, f64),
    pub(super) points: Vec<(i64, f64)>,
}

impl ParameterClipboard {
    pub(super) fn capture(
        mut points: Vec<(i64, f64)>,
        range: (f64, f64),
        lang: UiLanguage,
    ) -> Result<Option<Self>, String> {
        let Some(&(first, _)) = points.first() else {
            return Ok(None);
        };
        for point in &mut points {
            point.0 = point
                .0
                .checked_sub(first)
                .ok_or(lang.text("所选参数控制点的时间跨度超出可用范围。", "Selected parameter points span an invalid time range.", "選択したパラメーター制御点の時間範囲が無効です。"))?;
        }
        let clipboard = Self {
            kind: CLIPBOARD_KIND.into(),
            range,
            points,
        };
        clipboard.validate(lang)?;
        Ok(Some(clipboard))
    }

    fn validate(&self, lang: UiLanguage) -> Result<(), String> {
        if self.kind != CLIPBOARD_KIND {
            return Err(lang.text("剪贴板中没有 RynthV 参数控制点，请先复制参数控制点。", "Clipboard has no RynthV parameter points. Copy some points first.", "クリップボードに RynthV のパラメーター制御点がありません。先に制御点をコピーしてください。").into());
        }
        let (low, high) = self.range;
        if !low.is_finite() || !high.is_finite() || low >= high || !(high - low).is_finite() {
            return Err(lang.text("参数控制点剪贴板的取值范围无效。", "Invalid parameter clipboard value range.", "パラメーター制御点のクリップボードの値範囲が無効です。").into());
        }
        if self.points.is_empty() || self.points.len() > MAX_CLIPBOARD_POINTS {
            return Err(format!(
                "{}: 1–{MAX_CLIPBOARD_POINTS}" , lang.text("参数剪贴板须包含控制点数量", "Parameter clipboard point count must be", "パラメーター制御点の数")
            ));
        }
        if self.points[0].0 != 0
            || self.points.iter().any(|&(time, value)| {
                time < 0 || !value.is_finite() || !(low..=high).contains(&value)
            })
            || self.points.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(lang.text("参数控制点剪贴板的时间顺序或数值无效。", "Invalid parameter clipboard timestamps or values.", "パラメーター制御点の時間順または値が無効です。").into());
        }
        Ok(())
    }

    pub(super) fn to_text(&self, lang: UiLanguage) -> Result<String, String> {
        let text =
            serde_json::to_string(self).map_err(|error| format!("{}: {error}", lang.text("无法复制参数控制点", "Cannot copy parameter points", "パラメーター制御点をコピーできません")))?;
        if text.len() > MAX_CLIPBOARD_BYTES {
            return Err(lang.text("所选参数控制点数据过大，无法复制。", "Too many parameter points to copy.", "パラメーター制御点のデータが大きすぎてコピーできません。").into());
        }
        Ok(text)
    }

    pub(super) fn from_text(text: &str, lang: UiLanguage) -> Result<Self, String> {
        if text.len() > MAX_CLIPBOARD_BYTES {
            return Err(lang.text("剪贴板数据过大，无法粘贴参数控制点。", "Clipboard data is too large to paste parameter points.", "クリップボードのデータが大きすぎて制御点を貼り付けられません。").into());
        }
        let clipboard: Self = serde_json::from_str(text)
            .map_err(|error| format!("{}: {error}", lang.text("剪贴板中没有有效的 RynthV 参数控制点", "Clipboard does not contain valid RynthV parameter points", "クリップボードに有効な RynthV パラメーター制御点がありません")))?;
        clipboard.validate(lang)?;
        Ok(clipboard)
    }
}
