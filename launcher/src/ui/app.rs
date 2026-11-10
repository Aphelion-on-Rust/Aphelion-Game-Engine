use super::commands::{self, ControllerState, Effect, LauncherCommand};
use super::models::LauncherModels;
use super::tabs::installs as installs_tab;
use super::tabs::projects::{self as projects_tab, ProjectDraft};
use super::tabs::settings::{self as settings_tab, SettingsDraft};
use crate::installs::InstallsView;
use crate::jobs::query as jobs_query;
use crate::projects::{ProjectsController, ProjectsView};
use crate::services::{clock, github, install, launch, BackgroundEvent};
use eframe::egui;
use flux_core::mivc::{Model, Store};
use flux_core::project::disk;
use flux_core::storage::SettingsStore;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Versions,
    Projects,
    Settings,
}

/// The launcher window. One frame:
/// background events -> draw (collect commands) -> apply commands -> run effects -> save.
pub struct LauncherApp {
    models: LauncherModels,
    state: ControllerState,
    store: SettingsStore,
    events_tx: Sender<BackgroundEvent>,
    events_rx: Receiver<BackgroundEvent>,
    // Screen state (not data):
    tab: Tab,
    installs_view: InstallsView,
    projects_view: ProjectsView,
    confirm_uninstall: Option<String>,
    settings_draft: SettingsDraft,
    project_draft: ProjectDraft,
    started: bool,
}

impl LauncherApp {
    pub fn new() -> Self {
        let store = SettingsStore::for_app("launcher");
        let mut models = LauncherModels::default();

        models.settings = Store::new(store.load_or_default("settings.yaml"));
        models.releases = Store::new(store.load_or_default("releases.yaml"));
        models.installs = Store::new(store.load_or_default("installs.yaml"));
        models.projects = Store::new(store.load_or_default("projects.yaml"));

        let (events_tx, events_rx) = channel();
        let settings_draft = SettingsDraft::from_models(&models);

        Self {
            models,
            state: ControllerState::default(),
            store,
            events_tx,
            events_rx,
            tab: Tab::Versions,
            installs_view: InstallsView::default(),
            projects_view: ProjectsView::default(),
            confirm_uninstall: None,
            settings_draft,
            project_draft: ProjectDraft::default(),
            started: false,
        }
    }

    fn save(&mut self) {
        let results = [
            self.store.save("settings.yaml", self.models.settings.data()),
            self.store.save("releases.yaml", self.models.releases.data()),
            self.store.save("installs.yaml", self.models.installs.data()),
            self.store.save("projects.yaml", self.models.projects.data()),
        ];

        if let Some(Err(error)) = results.into_iter().find(|result| result.is_err()) {
            self.state.status = format!("could not save: {error}");
        }

        self.state.dirty = false;
    }

    /// Does the outside-world part of a command: network, disk, starting programs.
    fn run_effect(&mut self, effect: Effect, ctx: &egui::Context) {
        // Background threads call this to make the window redraw.
        let wake = {
            let ctx = ctx.clone();

            move || ctx.request_repaint()
        };

        match effect {
            Effect::FetchReleases { owner, repo } => {
                let sender = self.events_tx.clone();

                std::thread::spawn(move || {
                    let result = github::fetch_releases_json(&owner, &repo).map_err(|error| error.to_string());

                    let _ = sender.send(BackgroundEvent::ReleasesFetched(result));
                    wake();
                });
            }
            Effect::Install(request) => install::spawn_install(request, self.events_tx.clone(), Box::new(wake)),
            Effect::DeleteFolder(folder) => {
                self.state.status = match std::fs::remove_dir_all(&folder) {
                    Ok(()) => format!("Removed {}", folder.display()),
                    Err(error) => format!("could not remove {}: {error}", folder.display()),
                };
            }
            Effect::CreateProjectFolder { root, data } => {
                self.state.status = match disk::create_project(&root, &data) {
                    Ok(()) => format!("Created project {}", data.name),
                    Err(error) => {
                        // It could not be created, so take it out of the list again.
                        let _ = ProjectsController.remove(&mut self.models.projects, &root.to_string_lossy());
                        format!("could not create the project: {error}")
                    }
                };
            }
            Effect::ReadProject(folder) => match disk::load_project(&folder) {
                Ok(data) => {
                    let add = LauncherCommand::AddProject {
                        name: data.name,
                        path: folder.to_string_lossy().to_string(),
                        version: data.engine_version,
                    };

                    let _ = commands::apply_all(vec![add], &mut self.models, &mut self.state, &clock::today());
                }
                Err(error) => self.state.status = format!("not a Flux Aphelion project: {error}"),
            },
            Effect::SaveProjectVersion { path, version } => {
                if let Err(error) = disk::set_engine_version(&path, &version) {
                    self.state.status = format!("the project file was not updated: {error}");
                }
            }
            Effect::LaunchEditor { folder, project } => {
                self.state.status = match launch::launch_editor(&folder, project.as_deref()) {
                    Ok(()) => "Editor started".to_string(),
                    Err(error) => error.to_string(),
                };
            }
        }
    }
}

impl Default for LauncherApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let today = clock::today();
        let mut requests: Vec<LauncherCommand> = Vec::new();
        let mut effects: Vec<Effect> = Vec::new();

        if !self.started {
            self.started = true;

            if self.models.settings.data().check_updates_on_start {
                requests.push(LauncherCommand::Refresh);
            }
        }

        // 1. What the background threads reported.
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                BackgroundEvent::ReleasesFetched(Ok(json)) => {
                    commands::apply_releases_json(&json, &mut self.models, &mut self.state, &today);
                }
                BackgroundEvent::ReleasesFetched(Err(message)) => commands::apply_fetch_error(&message, &mut self.state),
                BackgroundEvent::Install(event) => {
                    effects.extend(commands::apply_install_event(&event, &mut self.models, &mut self.state, &today));
                }
            }
        }

        // 2. Draw. The views only read the models and push commands.
        egui::SidePanel::left("sidebar").exact_width(170.0).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Flux Aphelion");
            ui.weak("Launcher");
            ui.separator();
            ui.selectable_value(&mut self.tab, Tab::Versions, "Versions");
            ui.selectable_value(&mut self.tab, Tab::Projects, "Projects");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.weak(self.state.status.as_str());
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Versions => installs_tab::show(
                ui,
                &mut self.installs_view,
                &self.models,
                self.state.refreshing,
                &mut self.confirm_uninstall,
                &mut requests,
            ),
            Tab::Projects => projects_tab::show(ui, &mut self.projects_view, &self.models, &mut self.project_draft, &mut requests),
            Tab::Settings => settings_tab::show(ui, &self.models, &mut self.settings_draft, &mut requests),
        });

        // 3. Apply the commands through the controllers.
        effects.extend(commands::apply_all(requests, &mut self.models, &mut self.state, &today));

        // 4. Carry out the effects.
        for effect in effects {
            self.run_effect(effect, ctx);
        }

        // 5. Save, and keep redrawing while something is running.
        if self.state.dirty {
            self.save();
        }

        if self.state.refreshing || jobs_query::any_active(self.models.jobs.data()) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
