use crate::hierarchy::HierarchyView;
use crate::ui::commands::UiCommand;
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::{Model, View};

/// The text being typed while an object is renamed. Screen state, not data.
#[derive(Default)]
pub struct RenameBuffer {
    for_object: Option<AssetUuid>,
    text: String,
}

pub fn show(
    ui: &mut egui::Ui,
    view: &mut HierarchyView,
    buffer: &mut RenameBuffer,
    models: &EditorModels,
    commands: &mut Vec<UiCommand>,
) {
    // The view reads the scene and the panel state and builds the rows. It changes nothing.
    view.render((&models.scene, &models.hierarchy_ui));

    let selected = models.hierarchy_ui.data().selected.first().copied();

    ui.horizontal(|ui| {
        if ui.button("+ Create").clicked() {
            commands.push(UiCommand::CreateObject { parent: selected });
        }

        if ui.button("Delete").clicked() {
            commands.push(UiCommand::DeleteSelected);
        }
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in view.rows() {
            let response = ui
                .horizontal(|ui| {
                    ui.add_space(row.depth as f32 * 14.0);

                    if row.has_children {
                        if ui.small_button(if row.expanded { "v" } else { ">" }).clicked() {
                            commands.push(UiCommand::ToggleExpanded(row.uuid));
                        }
                    } else {
                        ui.add_space(22.0);
                    }

                    if row.renaming {
                        // A text box instead of the name. Enter or clicking away accepts, Escape cancels.
                        let just_started = buffer.for_object != Some(row.uuid);

                        if just_started {
                            buffer.for_object = Some(row.uuid);
                            buffer.text = row.name.clone();
                        }

                        let edit = ui.add(egui::TextEdit::singleline(&mut buffer.text).desired_width(180.0));

                        if just_started {
                            edit.request_focus();
                        }

                        if edit.lost_focus() {
                            if ui.input(|state| state.key_pressed(egui::Key::Escape)) {
                                commands.push(UiCommand::CancelRename);
                            } else {
                                commands.push(UiCommand::FinishRename { object: row.uuid, name: buffer.text.clone() });
                            }
                        }

                        return None;
                    }

                    let label = if row.is_prefab_root { format!("{} (prefab)", row.name) } else { row.name.clone() };
                    let mut text = egui::RichText::new(label);

                    if !row.active_in_hierarchy {
                        text = text.weak();
                    }

                    Some(ui.selectable_label(row.selected, text).interact(egui::Sense::click_and_drag()))
                })
                .inner;

            let Some(response) = response else {
                continue;
            };

            if response.clicked() {
                let additive = ui.input(|state| state.modifiers.ctrl || state.modifiers.command);

                commands.push(UiCommand::SelectObject { object: row.uuid, additive });
            }

            if response.double_clicked() {
                commands.push(UiCommand::BeginRename(row.uuid));
            }

            // Drag a row onto another row to make it a child.
            response.dnd_set_drag_payload(row.uuid);

            if let Some(dragged) = response.dnd_release_payload::<AssetUuid>() {
                if *dragged != row.uuid {
                    commands.push(UiCommand::Reparent { object: *dragged, new_parent: Some(row.uuid) });
                }
            }

            response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    commands.push(UiCommand::BeginRename(row.uuid));
                    ui.close_menu();
                }

                if ui.button("Create Prefab").clicked() {
                    commands.push(UiCommand::SelectObject { object: row.uuid, additive: false });
                    commands.push(UiCommand::CreatePrefab);
                    ui.close_menu();
                }

                if row.is_prefab_root && ui.button("Apply Changes to Prefab").clicked() {
                    commands.push(UiCommand::SelectObject { object: row.uuid, additive: false });
                    commands.push(UiCommand::ApplyToPrefab);
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Create Child").clicked() {
                    commands.push(UiCommand::CreateObject { parent: Some(row.uuid) });
                    ui.close_menu();
                }

                let label = if row.active { "Turn Off" } else { "Turn On" };

                if ui.button(label).clicked() {
                    commands.push(UiCommand::SetObjectActive { object: row.uuid, active: !row.active });
                    ui.close_menu();
                }

                if ui.button("Delete").clicked() {
                    commands.push(UiCommand::SelectObject { object: row.uuid, additive: false });
                    commands.push(UiCommand::DeleteSelected);
                    ui.close_menu();
                }
            });
        }

        // No row is being renamed any more: forget the text, so the next rename starts fresh.
        if !view.rows().iter().any(|row| row.renaming) {
            buffer.for_object = None;
        }

        // The empty space below the rows: drop here to move an object to the top level.
        let empty = ui.allocate_response(ui.available_size().max(egui::vec2(10.0, 40.0)), egui::Sense::hover());

        if let Some(dragged) = empty.dnd_release_payload::<AssetUuid>() {
            commands.push(UiCommand::Reparent { object: *dragged, new_parent: None });
        }
    });
}
