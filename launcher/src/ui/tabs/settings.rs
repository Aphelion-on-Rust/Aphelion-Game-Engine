use crate::ui::commands::LauncherCommand;
use crate::ui::models::LauncherModels;
use eframe::egui;
use flux_core::mivc::Model;

/// What is typed in the settings form. Screen state, not data.
#[derive(Default)]
pub struct SettingsDraft {
    pub owner: String,
    pub repo: String,
    pub install_root: String,
}

impl SettingsDraft {
    pub fn from_models(models: &LauncherModels) -> Self {
        let settings = models.settings.data();

        Self {
            owner: settings.github_owner.clone(),
            repo: settings.github_repo.clone(),
            install_root: settings.install_root.clone(),
        }
    }
}

pub fn show(
    ui: &mut egui::Ui,
    models: &LauncherModels,
    draft: &mut SettingsDraft,
    commands: &mut Vec<LauncherCommand>,
) {
    ui.heading("Settings");
    ui.separator();

    ui.label("Where the engine releases are published on GitHub");
    ui.horizontal(|ui| {
        ui.label("github.com /");
        ui.add(egui::TextEdit::singleline(&mut draft.owner).desired_width(140.0));
        ui.label("/");
        ui.add(egui::TextEdit::singleline(&mut draft.repo).desired_width(180.0));
    });

    ui.add_space(8.0);
    ui.label("Install folder (engine versions go in its \"versions\" folder)");
    ui.add(egui::TextEdit::singleline(&mut draft.install_root).desired_width(420.0));

    ui.add_space(8.0);

    if ui.button("Save").clicked() {
        commands.push(LauncherCommand::SetRepository { owner: draft.owner.clone(), repo: draft.repo.clone() });
        commands.push(LauncherCommand::SetInstallRoot(draft.install_root.clone()));
    }

    ui.separator();

    let settings = models.settings.data();
    let mut include = settings.include_prereleases;
    let mut check = settings.check_updates_on_start;

    if ui.checkbox(&mut include, "Show pre-release (beta) versions").changed() {
        commands.push(LauncherCommand::SetIncludePrereleases(include));
    }

    if ui.checkbox(&mut check, "Look for new versions when the launcher starts").changed() {
        commands.push(LauncherCommand::SetCheckUpdates(check));
    }
}
