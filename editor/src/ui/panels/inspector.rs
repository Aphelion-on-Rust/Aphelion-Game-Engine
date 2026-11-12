use crate::ui::commands::UiCommand;
use crate::ui::field_widgets::{draw_fields, FieldContext};
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::identity::AssetUuid;
use flux_engine::inspect::query::{read_field, schema_for};
use flux_engine::inspect::FieldPath;
use flux_engine::mivc::Model;
use flux_engine::scene::{query, ComponentData, ComponentEntry, GameObjectData};

/// The Eclipse Inspector panel: the selected object, and each of its components drawn from the
/// description of its fields (see `field_widgets`).
pub fn show(ui: &mut egui::Ui, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    let Some(selected) = models.hierarchy_ui.data().selected.first().copied() else {
        ui.weak("Nothing selected");
        return;
    };

    let objects = &models.scene.data().objects;
    let Some(object) = query::find_object(objects, &selected) else {
        ui.weak("Nothing selected");
        return;
    };

    // Name and active switch. We edit local copies and send a command when they change.
    ui.horizontal(|ui| {
        let mut active = object.active;

        if ui.checkbox(&mut active, "").changed() {
            commands.push(UiCommand::SetObjectActive { object: selected, active });
        }

        let mut name = object.name.clone();

        if ui.text_edit_singleline(&mut name).changed() {
            commands.push(UiCommand::RenameObject { object: selected, name });
        }
    });
    ui.weak(format!("uuid {}", object.uuid));

    if object.prefab_source.is_some() {
        ui.weak("Instance of a prefab (right-click it in the Hierarchy to apply changes)");
    }

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in &object.components {
            ui.push_id(entry.id.value(), |ui| component_section(ui, models, objects, object, entry, commands));
        }
    });
}

fn component_section(
    ui: &mut egui::Ui,
    models: &EditorModels,
    objects: &[GameObjectData],
    object: &GameObjectData,
    entry: &ComponentEntry,
    commands: &mut Vec<UiCommand>,
) {
    let kind = match &entry.data {
        ComponentData::Transform(_) => "Transform",
        ComponentData::RectTransform(_) => "RectTransform",
        ComponentData::Script(_) => "Script",
    };
    let uuid: AssetUuid = object.uuid;

    egui::CollapsingHeader::new(format!("{kind}  (id {})", entry.id.value()))
        .default_open(true)
        .show(ui, |ui| {
            if let ComponentData::Script(script) = &entry.data {
                let mut enabled = script.enabled;

                if ui.checkbox(&mut enabled, "Enabled").changed() {
                    commands.push(UiCommand::SetScriptEnabled { object: uuid, component: entry.id, enabled });
                }

                ui.weak(format!("script {}", script.script));
            }

            // The fields come from the component's description (built in, or the script's schema).
            match schema_for(&entry.data, models.schemas.data()) {
                Some(schema) => {
                    let context = FieldContext { object: uuid, component: entry.id, objects };
                    let lookup = |name: &str| read_field(&entry.data, &FieldPath::field(name));

                    draw_fields(ui, &context, &schema, &FieldPath::default(), &lookup, false, commands);
                }
                None => {
                    ui.weak("No field information for this script yet. The scripting host writes it.");
                }
            }

            // A Transform / RectTransform can be reset but not removed. Anything else can be removed.
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if entry.data.is_placement() {
                    if ui.small_button("Reset").clicked() {
                        commands.push(UiCommand::ResetComponent { object: uuid, component: entry.id });
                    }
                } else if ui.small_button("Remove").clicked() {
                    commands.push(UiCommand::RemoveComponent { object: uuid, component: entry.id });
                }
            });
        });
}
