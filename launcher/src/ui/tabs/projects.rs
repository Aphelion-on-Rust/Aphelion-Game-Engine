use crate::projects::ProjectsView;
use crate::releases::query as releases_query;
use crate::ui::commands::LauncherCommand;
use crate::ui::models::LauncherModels;
use crate::version::Version;
use eframe::egui;
use flux_core::mivc::{Model, View};
use flux_core::project::GraphicsPipeline;

/// What is typed in the two forms at the bottom. Screen state, not data.
#[derive(Default)]
pub struct ProjectDraft {
    pub name: String,
    pub parent_folder: String,
    pub version: String,
    pub pipeline: GraphicsPipeline,
    pub aphelion_control: bool,
    pub existing_path: String,
}

/// Every version the user could pick: installed ones and ones listed on GitHub. Newest first.
fn known_versions(models: &LauncherModels) -> Vec<String> {
    let include = models.settings.data().include_prereleases;
    let mut versions: Vec<String> = releases_query::visible(models.releases.data(), include)
        .iter()
        .map(|release| release.version.clone())
        .collect();

    for installed in &models.installs.data().versions {
        if !versions.contains(&installed.version) {
            versions.push(installed.version.clone());
        }
    }

    versions.sort_by(|a, b| Version::parse_or_zero(b).cmp(&Version::parse_or_zero(a)));
    versions
}

fn version_picker(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    selected: &str,
    versions: &[String],
) -> Option<String> {
    let mut picked = None;
    let text = if selected.is_empty() { "default".to_string() } else { selected.to_string() };

    egui::ComboBox::from_id_source(id).selected_text(text).show_ui(ui, |ui| {
        for version in versions {
            if ui.selectable_label(version == selected, version.as_str()).clicked() {
                picked = Some(version.clone());
            }
        }
    });

    picked
}

fn pipeline_choice(ui: &mut egui::Ui, draft: &mut ProjectDraft, pipeline: GraphicsPipeline) {
    let title = format!("{} - {}", pipeline.short_name(), pipeline.full_name());

    ui.radio_value(&mut draft.pipeline, pipeline, title);
    ui.indent(pipeline.short_name(), |ui| {
        ui.weak(pipeline.description());
    });
}

fn create_form(ui: &mut egui::Ui, draft: &mut ProjectDraft, versions: &[String], commands: &mut Vec<LauncherCommand>) {
    ui.label(egui::RichText::new("Create a new project").strong());

    ui.horizontal(|ui| {
        ui.label("Name");
        ui.add(egui::TextEdit::singleline(&mut draft.name).desired_width(220.0));
    });

    ui.horizontal(|ui| {
        ui.label("Create in folder");
        ui.add(egui::TextEdit::singleline(&mut draft.parent_folder).hint_text("D:\\Games").desired_width(320.0));
    });

    ui.horizontal(|ui| {
        ui.label("Engine version");

        if let Some(version) = version_picker(ui, "new_project_version", &draft.version, versions) {
            draft.version = version;
        }
    });

    ui.add_space(4.0);
    ui.label("Graphics pipeline (chosen now, can't be changed later)");
    pipeline_choice(ui, draft, GraphicsPipeline::Simple);
    pipeline_choice(ui, draft, GraphicsPipeline::Heavy);

    ui.add_space(4.0);
    ui.checkbox(&mut draft.aphelion_control, "Aphelion's Control: version history for prefabs (chosen now, can't be changed later)");

    ui.add_space(6.0);

    if ui.button("Create project").clicked() {
        commands.push(LauncherCommand::CreateProject {
            name: draft.name.clone(),
            parent_folder: draft.parent_folder.clone(),
            version: draft.version.clone(),
            pipeline: draft.pipeline,
            aphelion_control: draft.aphelion_control,
        });
        draft.name.clear();
    }
}

fn existing_form(ui: &mut egui::Ui, draft: &mut ProjectDraft, commands: &mut Vec<LauncherCommand>) {
    ui.label(egui::RichText::new("Add an existing project").strong());

    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut draft.existing_path).hint_text("Project folder").desired_width(380.0));

        if ui.button("Add").clicked() {
            commands.push(LauncherCommand::ImportProject(draft.existing_path.clone()));
            draft.existing_path.clear();
        }
    });
}

pub fn show(
    ui: &mut egui::Ui,
    view: &mut ProjectsView,
    models: &LauncherModels,
    draft: &mut ProjectDraft,
    commands: &mut Vec<LauncherCommand>,
) {
    view.render((&models.projects, &models.installs));

    let versions = known_versions(models);

    ui.heading("Projects");
    ui.weak("Each project opens with the engine version you choose. A missing version is installed first.");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if view.rows().is_empty() {
            ui.weak("No projects yet. Create one or add an existing one below.");
        }

        for row in view.rows() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(row.name.as_str()).strong());
                        ui.weak(row.path.as_str());

                        if !row.last_opened.is_empty() {
                            ui.weak(format!("opened {}", row.last_opened));
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Remove").clicked() {
                            commands.push(LauncherCommand::RemoveProject(row.path.clone()));
                        }

                        let open = if row.engine_installed { "Open" } else { "Install & Open" };

                        if ui.button(open).clicked() {
                            commands.push(LauncherCommand::Launch {
                                version: row.engine_version.clone(),
                                project: Some(row.path.clone()),
                            });
                        }

                        if let Some(version) = version_picker(ui, ("project_version", &row.path), &row.engine_version, &versions) {
                            commands.push(LauncherCommand::SetProjectVersion { path: row.path.clone(), version });
                        }
                    });
                });
            });
        }

        ui.add_space(12.0);
        ui.separator();
        create_form(ui, draft, &versions, commands);

        ui.add_space(12.0);
        ui.separator();
        existing_form(ui, draft, commands);
    });
}
