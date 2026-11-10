use super::format_size;
use crate::installs::{InstallsView, RowState};
use crate::jobs::{query as jobs_query, JobPhase};
use crate::releases::query as releases_query;
use crate::ui::commands::LauncherCommand;
use crate::ui::models::LauncherModels;
use eframe::egui;
use flux_core::mivc::{Model, View};

pub fn show(
    ui: &mut egui::Ui,
    view: &mut InstallsView,
    models: &LauncherModels,
    refreshing: bool,
    confirm_uninstall: &mut Option<String>,
    commands: &mut Vec<LauncherCommand>,
) {
    view.render((&models.releases, &models.installs, &models.jobs, &models.settings));

    ui.horizontal(|ui| {
        ui.heading("Engine versions");

        let label = if refreshing { "Refreshing..." } else { "Refresh" };

        if ui.add_enabled(!refreshing, egui::Button::new(label)).clicked() {
            commands.push(LauncherCommand::Refresh);
        }
    });

    let fetched = models.releases.data().fetched_on.as_str();

    if fetched.is_empty() {
        ui.weak("The version list has not been loaded yet. Press Refresh.");
    } else {
        ui.weak(format!("Last refreshed {fetched}"));
    }

    let include = models.settings.data().include_prereleases;

    if let Some(update) = releases_query::update_available(models.releases.data(), models.installs.data(), include) {
        ui.horizontal(|ui| {
            ui.label(format!("Version {} is available.", update.version));

            if !jobs_query::is_active(models.jobs.data(), &update.version) && ui.button("Install").clicked() {
                commands.push(LauncherCommand::Install(update.version.clone()));
            }
        });
    }

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in view.rows() {
            ui.horizontal(|ui| {
                ui.add_sized([90.0, 20.0], egui::Label::new(egui::RichText::new(row.version.as_str()).strong()));

                if row.prerelease {
                    ui.weak("pre-release");
                }

                if !row.published.is_empty() {
                    ui.weak(row.published.as_str());
                }

                if row.size_bytes > 0 {
                    ui.weak(format_size(row.size_bytes));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match &row.state {
                    RowState::Installed { is_default } => {
                        if confirm_uninstall.as_deref() == Some(row.version.as_str()) {
                            if ui.button("No").clicked() {
                                *confirm_uninstall = None;
                            }

                            if ui.button("Yes, delete").clicked() {
                                commands.push(LauncherCommand::Uninstall(row.version.clone()));
                                *confirm_uninstall = None;
                            }

                            ui.label("Delete this version?");
                        } else {
                            if ui.button("Uninstall").clicked() {
                                *confirm_uninstall = Some(row.version.clone());
                            }

                            if *is_default {
                                ui.label("Default");
                            } else if ui.button("Set default").clicked() {
                                commands.push(LauncherCommand::SetDefault(row.version.clone()));
                            }

                            if ui.button("Launch").clicked() {
                                commands.push(LauncherCommand::Launch { version: row.version.clone(), project: None });
                            }
                        }
                    }
                    RowState::Installing { phase, fraction } => {
                        let text = match phase {
                            JobPhase::Downloading => format!("Downloading {:.0}%", fraction * 100.0),
                            JobPhase::Verifying => "Verifying".to_string(),
                            _ => "Unpacking".to_string(),
                        };

                        ui.add(egui::ProgressBar::new(*fraction).desired_width(180.0).text(text));
                    }
                    RowState::Failed(message) => {
                        if ui.button("Retry").clicked() {
                            commands.push(LauncherCommand::Install(row.version.clone()));
                        }

                        ui.colored_label(ui.visuals().error_fg_color, message.as_str());
                    }
                    RowState::Available => {
                        if ui.button("Install").clicked() {
                            commands.push(LauncherCommand::Install(row.version.clone()));
                        }
                    }
                    RowState::NoBuild => {
                        ui.weak("no build for this platform");
                    }
                });
            });

            ui.separator();
        }
    });
}
