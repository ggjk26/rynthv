use super::{Viewport, beat_at, x_at};
use crate::{
    editor::Transport,
    model::{beats, ticks},
};
use eframe::egui::*;

pub struct RulerOptions {
    pub marker_step: Option<f64>,
    pub seek_step: Option<f64>,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
enum Part {
    Start,
    End,
    Body,
    Range,
}

#[derive(Clone, Copy)]
struct Drag {
    part: Part,
    pointer_beat: f64,
    start: i64,
    end: i64,
}

fn position(beat: f64, step: Option<f64>) -> i64 {
    ticks(step.map_or(beat, |step| (beat / step).round() * step)).max(0)
}

/// When zoomed far out, do not snap boundaries to subdivisions smaller
/// than a few screen pixels: they would be impossible to select accurately.
fn readable_step(view: &Viewport, step: Option<f64>) -> Option<f64> {
    step.map(|mut size| {
        while size * f64::from(view.beat_width) < 6.0 {
            size *= 2.0;
        }
        size
    })
}

fn marker_rect(ruler: Rect, view: &Viewport, transport: &Transport) -> Rect {
    Rect::from_min_max(
        pos2(
            x_at(ruler, view, beats(transport.loop_region.start())),
            ruler.bottom() - 32.0,
        ),
        pos2(
            x_at(ruler, view, beats(transport.loop_region.end())),
            ruler.bottom() - 16.0,
        ),
    )
}

fn hit(marker: Rect, ruler: Rect, pointer: Pos2) -> Option<Part> {
    if !ruler.contains(pointer) || !marker.contains(pointer) {
        return None;
    }
    let edge = 4.0_f32.min(marker.width() / 3.0);
    if pointer.x - marker.left() <= edge {
        Some(Part::Start)
    } else if marker.right() - pointer.x <= edge {
        Some(Part::End)
    } else {
        Some(Part::Body)
    }
}

/// Own the ruler's pointer response so loop edits cannot also seek the playhead.
/// Call before drawing ruler labels to keep the marker behind the bar numbers.
pub fn show(
    ui: &Ui,
    id: Id,
    ruler: Rect,
    view: &Viewport,
    transport: &mut Transport,
    options: RulerOptions,
    lang: crate::ui::settings::UiLanguage,
) {
    let response = ui.interact(ruler, id, Sense::click_and_drag());
    let drag_id = id.with("loop-drag");
    let menu_id = id.with("loop-menu-position");
    let (primary_pressed, primary_down, focused, pointer, alt, shift) = ui.input(|input| {
        (
            input.pointer.button_pressed(PointerButton::Primary),
            input.pointer.button_down(PointerButton::Primary),
            input.focused,
            input.pointer.interact_pos(),
            input.modifiers.alt,
            input.modifiers.shift,
        )
    });
    let mut drag = ui.ctx().data(|data| data.get_temp::<Drag>(drag_id));
    let enabled = options.enabled && focused && ui.is_enabled();
    if !enabled {
        drag = None;
    } else if primary_pressed {
        // Capture the original hit before crossing the drag threshold. Holding
        // Shift on empty ruler space creates a new loop interval, including
        // when looping was previously disabled.
        drag = pointer.filter(|_| response.contains_pointer()).and_then(|pointer| {
            let old = transport.looping.then(|| {
                hit(marker_rect(ruler, view, transport), ruler, pointer)
            }).flatten();
            let part = old.or_else(|| shift.then_some(Part::Range))?;
            Some(Drag {
                part,
                pointer_beat: beat_at(ruler, view, pointer.x),
                start: transport.loop_region.start(),
                end: transport.loop_region.end(),
            })
        });
    }

    if enabled {
        if let Some(drag) = drag {
            if (response.dragged_by(PointerButton::Primary)
                || response.drag_stopped_by(PointerButton::Primary))
                && let Some(pointer) = pointer
            {
                let delta = beat_at(ruler, view, pointer.x) - drag.pointer_beat;
                let step = if alt { None } else { readable_step(view, options.marker_step) };
                match drag.part {
                    Part::Start => transport
                        .loop_region
                        .set_start(position(beats(drag.start) + delta, step)),
                    Part::End => transport
                        .loop_region
                        .set_end(position(beats(drag.end) + delta, step)),
                    Part::Body => transport
                        .loop_region
                        .move_to(position(beats(drag.start) + delta, step)),
                    Part::Range => {
                        let anchor = position(drag.pointer_beat, step);
                        let pointer = position(beat_at(ruler, view, pointer.x), step);
                        let start = anchor.min(pointer);
                        let end = anchor.max(pointer)
                            .max(start.saturating_add(crate::model::QUARTER));
                        transport.loop_region.place_start(start);
                        transport.loop_region.set_end(end);
                        transport.looping = true;
                    }
                }
            }
        } else if (response.clicked() || response.dragged_by(PointerButton::Primary))
            && let Some(pointer) = response.interact_pointer_pos()
        {
            transport.position = position(beat_at(ruler, view, pointer.x), options.seek_step);
        }

        if response.secondary_clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let at = position(
                beat_at(ruler, view, pointer.x),
                if alt { None } else { readable_step(view, options.marker_step) },
            );
            ui.ctx().data_mut(|data| data.insert_temp(menu_id, at));
        }
        response.context_menu(|ui| {
            let Some(at) = ui.ctx().data(|data| data.get_temp::<i64>(menu_id)) else {
                return;
            };
            if ui.button(lang.text("将循环开始标记移至此处", "Set loop start here", "ループ開始をここに設定")).clicked() {
                transport.loop_region.place_start(at);
                transport.looping = true;
                ui.close();
            }
            if ui.button(lang.text("将循环结束标记移至此处", "Set loop end here", "ループ終了をここに設定")).clicked() {
                transport.loop_region.place_end(at);
                transport.looping = true;
                ui.close();
            }
        });
    }

    if transport.looping {
        let marker = marker_rect(ruler, view, transport);
        let part = enabled
            .then(|| {
                drag.map(|drag| drag.part).or_else(|| {
                    ui.input(|input| input.pointer.hover_pos())
                        .filter(|_| response.hovered())
                        .and_then(|pointer| hit(marker, ruler, pointer))
                })
            })
            .flatten();
        if let Some(part) = part {
            ui.ctx().set_cursor_icon(match part {
                Part::Start | Part::End => CursorIcon::ResizeHorizontal,
                Part::Body | Part::Range if primary_down => CursorIcon::Grabbing,
                Part::Body | Part::Range => CursorIcon::Grab,
            });
        }
        let painter = ui.painter().with_clip_rect(ruler.intersect(ui.clip_rect()));
        painter.rect_filled(marker, 0, Color32::from_rgb(67, 65, 70));
        painter.rect_stroke(
            marker,
            0,
            Stroke::new(1.0, Color32::from_rgb(85, 82, 89)),
            StrokeKind::Inside,
        );
    }
    ui.ctx().data_mut(|data| {
        if let Some(drag) = drag.filter(|_| primary_down && enabled) {
            data.insert_temp(drag_id, drag);
        } else {
            data.remove::<Drag>(drag_id);
        }
    });
}
