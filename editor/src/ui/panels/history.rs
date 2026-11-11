use crate::history::{query, HistoryRowKind, HistoryView};
use crate::ui::commands::UiCommand;
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::mivc::{Model, View};

/// The list of steps. Click a step to go to that state (undo or redo up to it).
pub fn show(ui: &mut egui::Ui, view: &mut HistoryView, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    view.render(&models.history);

    let history = models.history.data();

    ui.horizontal(|ui| {
        if ui.add_enabled(query::can_undo(history), egui::Button::new("Undo")).clicked() {
            commands.push(UiCommand::Undo);
        }

        if ui.add_enabled(query::can_redo(history), egui::Button::new("Redo")).clicked() {
            commands.push(UiCommand::Redo);
        }
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in view.rows() {
            let text = match row.kind {
                HistoryRowKind::Past => egui::RichText::new(row.label.as_str()),
                HistoryRowKind::Current => egui::RichText::new(format!("> {}", row.label)).strong(),
                HistoryRowKind::Future => egui::RichText::new(row.label.as_str()).weak(),
            };

            if ui.selectable_label(row.kind == HistoryRowKind::Current, text).clicked() {
                match row.kind {
                    HistoryRowKind::Past => commands.push(UiCommand::UndoSteps(row.steps)),
                    HistoryRowKind::Future => commands.push(UiCommand::RedoSteps(row.steps)),
                    HistoryRowKind::Current => {}
                }
            }
        }
    });
}
