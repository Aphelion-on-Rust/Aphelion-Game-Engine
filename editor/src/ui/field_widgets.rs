//! The Eclipse Inspector: draws the fields of a component from their descriptions.
//!
//! It knows nothing about Transform or scripts. For every field it looks at the kind
//! (bool, float, list ...) and the attributes ([Range], [Header], ShowIf ...) and draws the
//! right widget. It only reads values and pushes commands, it never changes anything.

use super::commands::UiCommand;
use super::field::{NumberTarget, Part};
use super::number_drag;
use crate::number_field::NumberFieldConfig;
use eframe::egui;
use flux_engine::identity::{AssetUuid, ComponentId};
use flux_engine::inspect::defaults::default_value;
use flux_engine::inspect::query::{label, layout, tooltip, LayoutItem};
use flux_engine::inspect::rules::{is_read_only, range_of};
use flux_engine::inspect::{FieldAttribute, FieldDescriptor, FieldKind, FieldPath, FieldValue};
use flux_engine::math::Rgba;
use flux_engine::scene::GameObjectData;

/// What every widget of one component needs to know.
pub struct FieldContext<'a> {
    pub object: AssetUuid,
    pub component: ComponentId,
    /// All objects of the scene, for the object picker.
    pub objects: &'a [GameObjectData],
}

/// Gives the current value of a field at the level being drawn (for ShowIf and for the values).
type Lookup<'a> = &'a dyn Fn(&str) -> Option<FieldValue>;

/// Everything needed to draw one value.
struct Spec<'a> {
    label: String,
    tooltip: Option<String>,
    kind: &'a FieldKind,
    attributes: &'a [FieldAttribute],
    /// What "Reset" goes back to. None for list elements.
    default: Option<FieldValue>,
    /// For a list element: the list and the element's index, to show a remove button.
    on_remove: Option<(FieldPath, usize)>,
}

fn unique_id(context: &FieldContext, path: &FieldPath) -> String {
    format!("{}{:?}", context.component.value(), path.segments())
}

/// Draws a list of field descriptions: headers, spaces, foldout groups and the fields.
/// `parent` is the path of the struct the fields belong to (empty at the top).
pub fn draw_fields(
    ui: &mut egui::Ui,
    context: &FieldContext,
    descriptors: &[FieldDescriptor],
    parent: &FieldPath,
    lookup: Lookup,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    let items = layout(descriptors, lookup);

    draw_items(ui, context, &items, parent, lookup, read_only, commands);
}

fn draw_items(
    ui: &mut egui::Ui,
    context: &FieldContext,
    items: &[LayoutItem],
    parent: &FieldPath,
    lookup: Lookup,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    for item in items {
        match item {
            LayoutItem::Header(text) => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(*text).strong());
            }
            LayoutItem::Space(height) => ui.add_space(*height),
            LayoutItem::Field(descriptor) => draw_field(ui, context, descriptor, parent, lookup, read_only, commands),
            LayoutItem::Group { name, items } => {
                ui.push_id(format!("{}group{name}", unique_id(context, parent)), |ui| {
                    egui::CollapsingHeader::new(*name).default_open(true).show(ui, |ui| {
                        draw_items(ui, context, items, parent, lookup, read_only, commands);
                    });
                });
            }
        }
    }
}

fn draw_field(
    ui: &mut egui::Ui,
    context: &FieldContext,
    descriptor: &FieldDescriptor,
    parent: &FieldPath,
    lookup: Lookup,
    parent_read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    let path = parent.child_field(&descriptor.name);
    let value = lookup(&descriptor.name).unwrap_or_else(|| default_value(&descriptor.kind));
    let spec = Spec {
        label: label(descriptor),
        tooltip: tooltip(descriptor).map(str::to_string),
        kind: &descriptor.kind,
        attributes: &descriptor.attributes,
        default: Some(descriptor.default.clone().unwrap_or_else(|| default_value(&descriptor.kind))),
        on_remove: None,
    };
    let read_only = parent_read_only || is_read_only(&descriptor.attributes);

    ui.push_id(unique_id(context, &path), |ui| {
        draw_value(ui, context, &spec, &path, &value, read_only, commands);
    });
}

fn draw_value(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    path: &FieldPath,
    value: &FieldValue,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    match (spec.kind, value) {
        (FieldKind::List { element }, FieldValue::List(items)) => {
            draw_list(ui, context, spec, element, path, items, read_only, commands);
        }
        (FieldKind::Struct { fields }, FieldValue::Struct(entries)) => {
            egui::CollapsingHeader::new(spec.label.as_str()).default_open(true).show(ui, |ui| {
                remove_button(ui, context, spec, read_only, commands);

                let lookup = |name: &str| entries.iter().find(|entry| entry.name == name).map(|entry| entry.value.clone());

                draw_fields(ui, context, fields, path, &lookup, read_only, commands);
            });
        }
        _ => draw_row(ui, context, spec, path, value, read_only, commands),
    }
}

/// The small "Remove" button of a list element.
fn remove_button(ui: &mut egui::Ui, context: &FieldContext, spec: &Spec, read_only: bool, commands: &mut Vec<UiCommand>) {
    if let Some((list, index)) = &spec.on_remove {
        if !read_only && ui.small_button("Remove element").clicked() {
            commands.push(UiCommand::ListRemove {
                object: context.object,
                component: context.component,
                path: list.clone(),
                index: *index,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_list(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    element: &FieldKind,
    path: &FieldPath,
    items: &[FieldValue],
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    egui::CollapsingHeader::new(spec.label.as_str()).default_open(true).show(ui, |ui| {
        remove_button(ui, context, spec, read_only, commands);
        ui.weak(format!("{} elements", items.len()));

        for (index, item) in items.iter().enumerate() {
            let element_path = path.child_index(index);
            let element_spec = Spec {
                label: format!("Element {index}"),
                tooltip: None,
                kind: element,
                attributes: spec.attributes,
                default: None,
                on_remove: Some((path.clone(), index)),
            };

            ui.push_id(unique_id(context, &element_path), |ui| {
                draw_value(ui, context, &element_spec, &element_path, item, read_only, commands);
            });
        }

        if !read_only && ui.button("+ Add").clicked() {
            commands.push(UiCommand::ListPush { object: context.object, component: context.component, path: path.clone() });
        }
    });
}

/// One line: the name on the left (right-click for "Reset to default", hover for the tooltip),
/// the widget on the right.
fn draw_row(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    path: &FieldPath,
    value: &FieldValue,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    ui.horizontal(|ui| {
        let mut name = ui.add_sized([120.0, 20.0], egui::Label::new(spec.label.as_str()).sense(egui::Sense::click()));

        if let Some(text) = &spec.tooltip {
            name = name.on_hover_text(text.as_str());
        }

        let can_reset = !read_only && spec.default.as_ref().map_or(false, |default| default != value);

        name.context_menu(|ui| {
            if can_reset && ui.button("Reset to default").clicked() {
                commands.push(UiCommand::ResetField {
                    object: context.object,
                    component: context.component,
                    path: path.clone(),
                });
                ui.close_menu();
            } else if !can_reset {
                ui.weak("Nothing to reset");
            }
        });

        edit_value(ui, context, spec, path, value, read_only, commands);

        if let Some((list, index)) = &spec.on_remove {
            if !read_only && ui.small_button("x").clicked() {
                commands.push(UiCommand::ListRemove {
                    object: context.object,
                    component: context.component,
                    path: list.clone(),
                    index: *index,
                });
            }
        }
    });
}

fn set(commands: &mut Vec<UiCommand>, context: &FieldContext, path: &FieldPath, value: FieldValue) {
    commands.push(UiCommand::SetField {
        object: context.object,
        component: context.component,
        path: path.clone(),
        value,
    });
}

fn multiline_rows(attributes: &[FieldAttribute]) -> Option<usize> {
    attributes.iter().find_map(|attribute| match attribute {
        FieldAttribute::Multiline { lines } => Some((*lines).max(1) as usize),
        _ => None,
    })
}

/// The widget for the value of one field.
fn edit_value(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    path: &FieldPath,
    value: &FieldValue,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    match (spec.kind, value) {
        (FieldKind::Bool, FieldValue::Bool(flag)) => {
            let mut checked = *flag;

            if ui.add_enabled(!read_only, egui::Checkbox::without_text(&mut checked)).changed() {
                set(commands, context, path, FieldValue::Bool(checked));
            }
        }
        (FieldKind::Float, FieldValue::Float(number)) => {
            number_or_slider(ui, context, spec, path, *number, false, read_only, commands);
        }
        (FieldKind::Int, FieldValue::Int(number)) => {
            number_or_slider(ui, context, spec, path, *number as f64, true, read_only, commands);
        }
        (FieldKind::Text, FieldValue::Text(text)) => {
            let mut edited = text.clone();
            let editor = match multiline_rows(spec.attributes) {
                Some(rows) => egui::TextEdit::multiline(&mut edited).desired_rows(rows).desired_width(220.0),
                None => egui::TextEdit::singleline(&mut edited).desired_width(220.0),
            };

            if ui.add_enabled(!read_only, editor).changed() {
                set(commands, context, path, FieldValue::Text(edited));
            }
        }
        (FieldKind::Vec2, FieldValue::Vec2(vector)) => {
            for (part, number) in [(Part::X, vector.x), (Part::Y, vector.y)] {
                number_box(ui, context, spec, path, part, number as f64, read_only, commands);
            }
        }
        (FieldKind::Vec3, FieldValue::Vec3(vector)) => {
            for (part, number) in [(Part::X, vector.x), (Part::Y, vector.y), (Part::Z, vector.z)] {
                number_box(ui, context, spec, path, part, number as f64, read_only, commands);
            }
        }
        (FieldKind::Color, FieldValue::Color(color)) => {
            let mut rgba = [color.r, color.g, color.b, color.a];
            let changed = ui
                .add_enabled_ui(!read_only, |ui| egui::color_picker::color_edit_button_rgba_unmultiplied(ui, &mut rgba).changed())
                .inner;

            if changed {
                set(commands, context, path, FieldValue::Color(Rgba::new(rgba[0], rgba[1], rgba[2], rgba[3])));
            }
        }
        (FieldKind::Enum { options }, FieldValue::Enum(current)) => {
            let mut picked: Option<String> = None;

            ui.add_enabled_ui(!read_only, |ui| {
                egui::ComboBox::from_id_source((context.component.value(), format!("{:?}", path.segments())))
                    .selected_text(current.as_str())
                    .show_ui(ui, |ui| {
                        for option in options {
                            if ui.selectable_label(option == current, option.as_str()).clicked() {
                                picked = Some(option.clone());
                            }
                        }
                    });
            });

            if let Some(name) = picked {
                set(commands, context, path, FieldValue::Enum(name));
            }
        }
        (FieldKind::ObjectRef, FieldValue::ObjectRef(target)) => {
            let shown = match target {
                None => "None".to_string(),
                Some(uuid) => context
                    .objects
                    .iter()
                    .find(|object| object.uuid == *uuid)
                    .map_or_else(|| "(missing object)".to_string(), |object| object.name.clone()),
            };
            let mut picked: Option<Option<AssetUuid>> = None;

            ui.add_enabled_ui(!read_only, |ui| {
                egui::ComboBox::from_id_source((context.component.value(), format!("{:?}", path.segments())))
                    .selected_text(shown)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(target.is_none(), "None").clicked() {
                            picked = Some(None);
                        }

                        for object in context.objects {
                            if ui.selectable_label(*target == Some(object.uuid), object.name.as_str()).clicked() {
                                picked = Some(Some(object.uuid));
                            }
                        }
                    });
            });

            if let Some(choice) = picked {
                set(commands, context, path, FieldValue::ObjectRef(choice));
            }
        }
        _ => {
            ui.weak("(the value does not match the kind of this field)");
        }
    }
}

/// A slider if the field has a [Range], otherwise the slide-to-change number box.
#[allow(clippy::too_many_arguments)]
fn number_or_slider(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    path: &FieldPath,
    number: f64,
    is_int: bool,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    let Some((a, b)) = range_of(spec.attributes) else {
        number_box(ui, context, spec, path, Part::Whole, number, read_only, commands);
        return;
    };
    let (low, high) = (a.min(b), a.max(b));

    if is_int {
        let mut whole = number.round() as i64;
        let slider = egui::Slider::new(&mut whole, (low.round() as i64)..=(high.round() as i64));

        if ui.add_enabled(!read_only, slider).changed() {
            set(commands, context, path, FieldValue::Int(whole));
        }
    } else {
        let mut value = number;

        if ui.add_enabled(!read_only, egui::Slider::new(&mut value, low..=high)).changed() {
            set(commands, context, path, FieldValue::Float(value));
        }
    }
}

/// One slide-to-change number (hover: <-> cursor, drag: the value slides, the pointer hides).
#[allow(clippy::too_many_arguments)]
fn number_box(
    ui: &mut egui::Ui,
    context: &FieldContext,
    spec: &Spec,
    path: &FieldPath,
    part: Part,
    number: f64,
    read_only: bool,
    commands: &mut Vec<UiCommand>,
) {
    let target = NumberTarget { object: context.object, component: context.component, path: path.clone(), part };
    let config = NumberFieldConfig::for_field(spec.kind, spec.attributes);

    ui.add_enabled_ui(!read_only, |ui| {
        number_drag::show(ui, target, number, &config, commands);
    });
}
