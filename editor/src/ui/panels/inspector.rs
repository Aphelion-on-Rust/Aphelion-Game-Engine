use crate::number_field::NumberFieldConfig;
use crate::ui::commands::UiCommand;
use crate::ui::field::{ComponentField, NumberTarget};
use crate::ui::models::EditorModels;
use crate::ui::number_drag;
use eframe::egui;
use flux_engine::identity::{AssetUuid, ComponentId};
use flux_engine::mivc::Model;
use flux_engine::scene::{query, ComponentData, ComponentEntry};

pub fn show(ui: &mut egui::Ui, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    let Some(selected) = models.hierarchy_ui.data().selected.first().copied() else {
        ui.weak("Nothing selected");
        return;
    };

    let Some(object) = query::find_object(&models.scene.data().objects, &selected) else {
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
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in &object.components {
            ui.push_id(entry.id.value(), |ui| component_section(ui, selected, entry, commands));
        }
    });
}

fn component_section(ui: &mut egui::Ui, object: AssetUuid, entry: &ComponentEntry, commands: &mut Vec<UiCommand>) {
    let kind = match &entry.data {
        ComponentData::Transform(_) => "Transform",
        ComponentData::RectTransform(_) => "RectTransform",
        ComponentData::Script(_) => "Script",
    };

    egui::CollapsingHeader::new(format!("{kind}  (id {})", entry.id.value()))
        .default_open(true)
        .show(ui, |ui| match &entry.data {
            ComponentData::Transform(transform) => {
                let position = NumberFieldConfig { sensitivity: 0.05, ..Default::default() };
                let rotation = NumberFieldConfig { sensitivity: 0.5, ..Default::default() };
                let scale = NumberFieldConfig { sensitivity: 0.01, ..Default::default() };

                row3(ui, "Position", object, entry.id, transform.position.into(), ComponentField::Position, &position, commands);
                row3(ui, "Rotation", object, entry.id, transform.rotation.into(), ComponentField::Rotation, &rotation, commands);
                row3(ui, "Scale", object, entry.id, transform.scale.into(), ComponentField::Scale, &scale, commands);
            }
            ComponentData::RectTransform(rect) => {
                let size = NumberFieldConfig { sensitivity: 0.5, ..Default::default() };
                let unit = NumberFieldConfig { sensitivity: 0.005, min: Some(0.0), max: Some(1.0), ..Default::default() };

                row2(ui, "Position", object, entry.id, [rect.position.x, rect.position.y], ComponentField::RectPosition, &size, commands);
                row2(ui, "Size", object, entry.id, [rect.size.x, rect.size.y], ComponentField::Size, &size, commands);
                row2(ui, "Anchor Min", object, entry.id, [rect.anchor_min.x, rect.anchor_min.y], ComponentField::AnchorMin, &unit, commands);
                row2(ui, "Anchor Max", object, entry.id, [rect.anchor_max.x, rect.anchor_max.y], ComponentField::AnchorMax, &unit, commands);
                row2(ui, "Pivot", object, entry.id, [rect.pivot.x, rect.pivot.y], ComponentField::Pivot, &unit, commands);
            }
            ComponentData::Script(script) => {
                let mut enabled = script.enabled;

                if ui.checkbox(&mut enabled, "Enabled").changed() {
                    commands.push(UiCommand::SetScriptEnabled { object, component: entry.id, enabled });
                }

                ui.weak(format!("script {}", script.script));
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn row3(
    ui: &mut egui::Ui,
    label: &str,
    object: AssetUuid,
    component: ComponentId,
    values: [f32; 3],
    make_field: fn(usize) -> ComponentField,
    config: &NumberFieldConfig,
    commands: &mut Vec<UiCommand>,
) {
    ui.horizontal(|ui| {
        ui.add_sized([80.0, 20.0], egui::Label::new(label));

        for (axis, value) in values.iter().enumerate() {
            let target = NumberTarget { object, component, field: make_field(axis) };

            number_drag::show(ui, target, *value as f64, config, commands);
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn row2(
    ui: &mut egui::Ui,
    label: &str,
    object: AssetUuid,
    component: ComponentId,
    values: [f32; 2],
    make_field: fn(usize) -> ComponentField,
    config: &NumberFieldConfig,
    commands: &mut Vec<UiCommand>,
) {
    ui.horizontal(|ui| {
        ui.add_sized([80.0, 20.0], egui::Label::new(label));

        for (axis, value) in values.iter().enumerate() {
            let target = NumberTarget { object, component, field: make_field(axis) };

            number_drag::show(ui, target, *value as f64, config, commands);
        }
    });
}
