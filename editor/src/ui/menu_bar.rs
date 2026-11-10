use super::commands::UiCommand;
use super::models::EditorModels;
use super::panels::{panel_title, ALL_PANELS};
use crate::keybinds::{actions, query::chord_for};
use crate::theme::query::active_theme;
use eframe::egui;
use flux_engine::mivc::Model;

/// A menu item with its shortcut written on the right.
fn item(ui: &mut egui::Ui, models: &EditorModels, label: &str, action: &str) -> bool {
    let mut button = egui::Button::new(label);

    if let Some(chord) = chord_for(models.keybinds.data(), action) {
        button = button.shortcut_text(chord.describe());
    }

    ui.add(button).clicked()
}

pub fn show(ctx: &egui::Context, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if item(ui, models, "Save Scene", actions::SAVE) {
                    commands.push(UiCommand::SaveScene);
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui| {
                let parent = models.hierarchy_ui.data().selected.first().copied();

                if item(ui, models, "New Object", actions::NEW_OBJECT) {
                    commands.push(UiCommand::CreateObject { parent });
                    ui.close_menu();
                }

                if item(ui, models, "Delete", actions::DELETE) {
                    commands.push(UiCommand::DeleteSelected);
                    ui.close_menu();
                }
            });

            ui.menu_button("Window", |ui| {
                for panel in ALL_PANELS {
                    if ui.button(panel_title(panel)).clicked() {
                        commands.push(UiCommand::OpenPanel(panel));
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("Reset Layout").clicked() {
                    commands.push(UiCommand::ResetLayout);
                    ui.close_menu();
                }
            });

            ui.menu_button("Theme", |ui| {
                let active = active_theme(models.theme.data()).name.clone();

                for theme in &models.theme.data().themes {
                    if ui.selectable_label(theme.name == active, &theme.name).clicked() {
                        commands.push(UiCommand::SetTheme(theme.name.clone()));
                        ui.close_menu();
                    }
                }
            });
        });
    });
}
