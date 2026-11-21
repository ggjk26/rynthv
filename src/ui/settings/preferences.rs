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
