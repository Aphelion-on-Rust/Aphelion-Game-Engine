//! The slide-to-change number box. Hover: <-> cursor. Press and drag: the value changes,
//! the pointer is hidden. Shift = fine, Ctrl = coarse. Escape cancels (handled in the app).
//!
//! It is a view: it draws the value it is given and pushes commands. It never changes data.

use super::commands::UiCommand;
use super::field::NumberTarget;
use crate::number_field::{DragSpeed, NumberFieldConfig};
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    target: NumberTarget,
    value: f64,
    config: &NumberFieldConfig,
    commands: &mut Vec<UiCommand>,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(64.0, 20.0), egui::Sense::click_and_drag());

    let fill = if response.hovered() || response.dragged() {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let text = if config.is_integer { format!("{value:.0}") } else { format!("{value:.3}") };

    ui.painter().rect_filled(rect, 3.0, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::monospace(12.0),
        ui.visuals().text_color(),
    );

    if response.hovered() {
        commands.push(UiCommand::NumberHover);
    }

    if response.drag_started() {
        commands.push(UiCommand::NumberDragBegin { target, value, config: config.clone() });
    }

    if response.dragged() {
        let (delta_x, modifiers) = ui.input(|state| (state.pointer.delta().x, state.modifiers));
        let speed = DragSpeed::from_keys(modifiers.shift, modifiers.ctrl || modifiers.command);

        commands.push(UiCommand::NumberDragMove { delta_x: delta_x as f64, speed });
    }

    if response.drag_stopped() {
        commands.push(UiCommand::NumberDragEnd);
    }
}
