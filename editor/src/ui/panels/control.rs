use crate::ui::commands::UiCommand;
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::control::ControlView;
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::{Model, View};

/// Aphelion's Control: the saves of each prefab. "Revert" undoes only what that save changed.
pub fn show(ui: &mut egui::Ui, view: &mut ControlView, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    let enabled = models.project.as_ref().map_or(false, |project| project.data.data().aphelion_control);

    if !enabled {
        ui.label(egui::RichText::new("Aphelion's Control is off").strong());
        ui.weak("It keeps a history of every prefab save. It can only be chosen when a project is created.");
        return;
    }

    view.render(&models.control);

    let prefabs: Vec<(AssetUuid, String)> = view.prefabs().to_vec();

    if prefabs.is_empty() {
        ui.weak("No prefab has been saved yet. Right-click an object in the Hierarchy and choose Create Prefab.");
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for (uuid, name) in &prefabs {
            if ui.selectable_label(view.selected() == Some(*uuid), name.as_str()).clicked() {
                view.select(Some(*uuid));
            }
        }
    });
    ui.separator();

    let Some(prefab) = view.selected() else {
        return;
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in view.rows() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("r{}", row.number)).strong());
                ui.label(row.message.as_str());
                ui.weak(row.saved_on.as_str());

                if !row.summary.is_empty() {
                    ui.weak(row.summary.as_str());
                }

                if let Some(number) = row.reverts {
                    ui.weak(format!("(reverts r{number})"));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if row.can_revert && ui.button("Revert").clicked() {
                        commands.push(UiCommand::RevertRevision { prefab, number: row.number });
                    }
                });
            });
        }
    });
}
