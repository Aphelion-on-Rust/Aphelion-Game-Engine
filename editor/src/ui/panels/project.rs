use crate::ui::commands::UiCommand;
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::assets::TextureFilter;
use flux_engine::mivc::Model;

/// Shows the open project and its prefabs. The pipeline and Aphelion's Control are fixed at creation.
pub fn show(ui: &mut egui::Ui, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    let Some(project) = &models.project else {
        ui.label(egui::RichText::new("No project open").strong());
        ui.weak("This is the demo scene. Start the editor from the launcher to open a project.");
        return;
    };

    let data = project.data.data();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(egui::RichText::new(data.name.as_str()).heading());
        ui.weak(project.root.display().to_string());
        ui.separator();

        egui::Grid::new("project_info").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
            ui.label("Engine version");
            ui.label(data.engine_version.as_str());
            ui.end_row();

            ui.label("Graphics pipeline");
            ui.label(format!("{} ({})", data.pipeline.short_name(), data.pipeline.full_name()));
            ui.end_row();

            ui.label("Aphelion's Control");
            ui.label(if data.aphelion_control { "On" } else { "Off" });
            ui.end_row();

            ui.label("Created");
            ui.label(data.created_on.as_str());
            ui.end_row();

            ui.label("Scene file");
            ui.label(project.scene_path.display().to_string());
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.weak(data.pipeline.description());
        ui.weak("The pipeline and Aphelion's Control were chosen when the project was created and can't be changed.");

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Prefabs").strong());

        let prefabs = &models.library.data().prefabs;

        if prefabs.is_empty() {
            ui.weak("None yet. Right-click an object in the Hierarchy and choose Create Prefab.");
        }

        for prefab in prefabs {
            ui.horizontal(|ui| {
                ui.label(prefab.name.as_str());

                if ui.button("Place in scene").clicked() {
                    commands.push(UiCommand::PlacePrefab { file: prefab.file.clone() });
                }
            });
        }

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Assets").strong());

            if ui.small_button("Refresh").clicked() {
                commands.push(UiCommand::RefreshAssets);
            }
        });

        let assets = &models.assets.data().assets;

        if assets.is_empty() {
            ui.weak("No pictures yet. Put .png or .jpg files in the project's Assets folder and press Refresh.");
        }

        for asset in assets {
            ui.horizontal(|ui| {
                ui.label(asset.path.as_str());

                let shown = if asset.filter == TextureFilter::Nearest { "Hard pixels" } else { "Smooth" };

                egui::ComboBox::from_id_source(("texture_filter", asset.uuid.to_string())).selected_text(shown).show_ui(ui, |ui| {
                    for (filter, text) in [(TextureFilter::Linear, "Smooth"), (TextureFilter::Nearest, "Hard pixels (pixel art)")] {
                        if ui.selectable_label(asset.filter == filter, text).clicked() {
                            commands.push(UiCommand::SetTextureFilter { asset: asset.uuid, filter });
                        }
                    }
                });
            });
        }
    });
}
