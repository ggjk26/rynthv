use super::theme::*;
use eframe::egui::*;

#[derive(Clone, Copy)]
pub enum Icon {
    ChevronDown,
    ChevronRight,
    Plus,
    Close,
    Play,
    Pause,
    Stop,
    Record,
    Rewind,
    Forward,
    Loop,
    Metronome,
    Return,
    Microphone,
    Notes,
    Dictionary,
    Render,
    Script,
    Cloud,
    Settings,
    NoteSelect,
    NoteDraw,
    PitchAnchor,
    PitchDraw,
    Pitch,
    Waveform,
    Retake,
    Scale,
    Magnet,
    TempoTracks,
    Pan,
    Curve,
    Swap,
    Phonemes,
    Brand,
}
impl Icon {
    pub fn image(self) -> Image<'static> {
        match self {
            Self::ChevronDown => Image::new(include_image!("../../assets/icons/chevron-down.svg")),
            Self::ChevronRight => {
                Image::new(include_image!("../../assets/icons/chevron-right.svg"))
            }
            Self::Plus => Image::new(include_image!("../../assets/icons/plus.svg")),
            Self::Close => Image::new(include_image!("../../assets/icons/close.svg")),
            Self::Play => Image::new(include_image!("../../assets/icons/play.svg")),
            Self::Pause => Image::new(include_image!("../../assets/icons/pause.svg")),
            Self::Stop => Image::new(include_image!("../../assets/icons/stop.svg")),
            Self::Record => Image::new(include_image!("../../assets/icons/record.svg")),
            Self::Rewind => Image::new(include_image!("../../assets/icons/rewind.svg")),
            Self::Forward => Image::new(include_image!("../../assets/icons/forward.svg")),
            Self::Loop => Image::new(include_image!("../../assets/icons/loop.svg")),
            Self::Metronome => Image::new(include_image!("../../assets/icons/metronome.svg")),
            Self::Return => Image::new(include_image!("../../assets/icons/return.svg")),
            Self::Microphone => Image::new(include_image!("../../assets/icons/microphone.svg")),
            Self::Notes => Image::new(include_image!("../../assets/icons/notes.svg")),
            Self::Dictionary => Image::new(include_image!("../../assets/icons/dictionary.svg")),
            Self::Render => Image::new(include_image!("../../assets/icons/render.svg")),
            Self::Script => Image::new(include_image!("../../assets/icons/script.svg")),
            Self::Cloud => Image::new(include_image!("../../assets/icons/cloud.svg")),
            Self::Settings => Image::new(include_image!("../../assets/icons/settings.svg")),
            Self::NoteSelect => Image::new(include_image!("../../assets/icons/note-select.svg")),
            Self::NoteDraw => Image::new(include_image!("../../assets/icons/note-draw.svg")),
            Self::PitchAnchor => Image::new(include_image!("../../assets/icons/pitch-anchor.svg")),
            Self::PitchDraw => Image::new(include_image!("../../assets/icons/pitch-draw.svg")),
            Self::Pitch => Image::new(include_image!("../../assets/icons/pitch.svg")),
            Self::Waveform => Image::new(include_image!("../../assets/icons/waveform.svg")),
            Self::Retake => Image::new(include_image!("../../assets/icons/retake.svg")),
            Self::Scale => Image::new(include_image!("../../assets/icons/scale.svg")),
            Self::Magnet => Image::new(include_image!("../../assets/icons/magnet.svg")),
            Self::TempoTracks => Image::new(include_image!("../../assets/icons/tempo-tracks.svg")),
            Self::Pan => Image::new(include_image!("../../assets/icons/pan.svg")),
            Self::Curve => Image::new(include_image!("../../assets/icons/curve.svg")),
            Self::Swap => Image::new(include_image!("../../assets/icons/swap.svg")),
            Self::Phonemes => Image::new(include_image!("../../assets/icons/phonemes.svg")),
            Self::Brand => Image::new(include_image!("../../assets/icons/brand.svg")),
        }
    }
    pub fn paint(self, ui: &Ui, rect: Rect, tint: Color32) {
        self.image()
            .tint(tint)
            .fit_to_exact_size(rect.size())
            .paint_at(ui, rect);
    }
}

pub struct IconButton {
    icon: Icon,
    label: &'static str,
    size: Vec2,
    icon_size: f32,
    selected: bool,
    framed: bool,
    shared_background: bool,
    background: Option<Color32>,
}
impl IconButton {
    pub fn new(icon: Icon, label: &'static str) -> Self {
        Self {
            icon,
            label,
            size: vec2(28.0, 24.0),
            icon_size: 18.0,
            selected: false,
            framed: false,
            shared_background: false,
            background: None,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn framed(mut self) -> Self {
        self.framed = true;
        self
    }
    pub fn grouped(mut self) -> Self {
        self.framed = true;
        self.shared_background = true;
        self
    }
    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }
}
impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let enabled = ui.is_enabled();
        let (rect, response) = ui.allocate_exact_size(
            self.size,
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, self.label));
        let hovered = response.hovered() && enabled;
        if hovered {
            ui.ctx().data_mut(|data| data.insert_temp(
                Id::new("toolbar-hover-status"), self.label.to_owned()
            ));
        }
        let hovering = ui.ctx().animate_bool_with_time(
            response.id.with("hover"), hovered, 0.1
        );
        let pressing = ui.ctx().animate_bool_with_time(
            response.id.with("press"), response.is_pointer_button_down_on(), 0.1
        );
        if (self.framed && !self.shared_background) || self.selected
            || hovering > 0.001 || pressing > 0.001
        {
            let base = self.background.unwrap_or(BUTTON);
            let fill = if self.selected {
                self.background.unwrap_or(BUTTON_ACTIVE)
            } else {
                base.lerp_to_gamma(BUTTON_HOVER, hovering)
                    .lerp_to_gamma(self.background.unwrap_or(BUTTON_ACTIVE), pressing)
            };
            ui.painter().rect_filled(rect, 3, fill);
        }
        let tint = if self.selected {
            TEXT
        } else if self.framed {
            ICON_TOOL
        } else {
            MUTED
        };
        let tint = if enabled {
            tint
        } else {
            tint.gamma_multiply(0.85)
        };
        self.icon.paint(
            ui,
            Rect::from_center_size(rect.center(), Vec2::splat(self.icon_size)),
            tint,
        );
        response
            .on_hover_text(self.label)
            .on_disabled_hover_text(self.label)
    }
}
