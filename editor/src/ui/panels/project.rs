use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::mivc::Model;

/// Shows the open project. Read-only: the pipeline and Aphelion's Control are fixed at creation.
pub fn show(ui: &mut egui::Ui, models: &EditorModels) {
    let Some(project) = &models.project else {
        ui.label(egui::RichText::new("No project open").strong());
        ui.weak("This is the demo scene. Start the editor from the launcher to open a project.");
        return;
    };

    let data = project.data.data();

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
}
