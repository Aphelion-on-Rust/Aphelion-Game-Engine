//! Commands are what the user asked for. `apply_all` runs them through the controllers.
//! Anything that needs the outside world comes back as an `Effect` for the app to carry out.

use super::models::LauncherModels;
use crate::error::{LauncherError, LauncherResult};
use crate::installs::{query as installs_query, InstalledVersion, InstallsController};
use crate::jobs::{InstallEvent, JobsController};
use crate::projects::{ProjectEntry, ProjectsController};
use crate::releases::{parse, query as releases_query, ReleasesController};
use crate::services::install::InstallRequest;
use crate::services::paths;
use crate::settings::SettingsController;
use flux_core::mivc::Model;
use flux_core::project::{GraphicsPipeline, ProjectController, ProjectData};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum LauncherCommand {
    Refresh,
    Install(String),
    Uninstall(String),
    SetDefault(String),
    /// Open the editor of a version, optionally with a project.
    /// If the version is not installed yet it is installed first, then the editor starts.
    Launch { version: String, project: Option<String> },
    /// Make a new project: its folder, `project.yaml` and layout. The pipeline and
    /// Aphelion's Control are chosen here and can't be changed later.
    CreateProject {
        name: String,
        parent_folder: String,
        version: String,
        pipeline: GraphicsPipeline,
        aphelion_control: bool,
    },
    /// Add a project folder that already exists (reads its `project.yaml`).
    ImportProject(String),
    AddProject { name: String, path: String, version: String },
    RemoveProject(String),
    SetProjectVersion { path: String, version: String },
    SetRepository { owner: String, repo: String },
    SetInstallRoot(String),
    SetIncludePrereleases(bool),
    SetCheckUpdates(bool),
}

/// Work for the outside world. The controllers decide, the app does it.
#[derive(Debug)]
pub enum Effect {
    FetchReleases { owner: String, repo: String },
    Install(InstallRequest),
    DeleteFolder(PathBuf),
    LaunchEditor { folder: PathBuf, project: Option<PathBuf> },
    CreateProjectFolder { root: PathBuf, data: ProjectData },
    ReadProject(PathBuf),
    /// Write the new engine version into the project's own `project.yaml`.
    SaveProjectVersion { path: PathBuf, version: String },
}

/// "Start this editor as soon as its install is done."
#[derive(Clone, Debug, PartialEq)]
pub struct PendingLaunch {
    pub version: String,
    pub project: Option<String>,
}

pub struct ControllerState {
    pub status: String,
    pub refreshing: bool,
    pub pending_launch: Option<PendingLaunch>,
    /// Something that is saved to a file changed.
    pub dirty: bool,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self { status: "Ready".to_string(), refreshing: false, pending_launch: None, dirty: false }
    }
}

/// Runs the commands in order. An error does not stop the rest, it shows in the status bar.
pub fn apply_all(
    commands: Vec<LauncherCommand>,
    models: &mut LauncherModels,
    state: &mut ControllerState,
    today: &str,
) -> Vec<Effect> {
    let mut effects = Vec::new();

    for command in commands {
        match apply(command, models, state, today) {
            Ok(mut more) => effects.append(&mut more),
            Err(error) => state.status = error.to_string(),
        }
    }

    effects
}

/// The release list arrived from GitHub.
pub fn apply_releases_json(json: &str, models: &mut LauncherModels, state: &mut ControllerState, today: &str) {
    state.refreshing = false;

    match parse::parse_releases(json, parse::PLATFORM) {
        Ok(releases) => {
            let count = releases.len();

            ReleasesController.set_releases(&mut models.releases, releases, today);
            state.dirty = true;
            state.status = format!("Found {count} versions");
        }
        Err(error) => state.status = error.to_string(),
    }
}

/// Asking GitHub failed.
pub fn apply_fetch_error(message: &str, state: &mut ControllerState) {
    state.refreshing = false;
    state.status = message.to_string();
}

/// The install thread reported something.
pub fn apply_install_event(
    event: &InstallEvent,
    models: &mut LauncherModels,
    state: &mut ControllerState,
    today: &str,
) -> Vec<Effect> {
    JobsController.apply(&mut models.jobs, event);

    match event {
        InstallEvent::Finished { version, folder } => {
            InstallsController.register(
                &mut models.installs,
                InstalledVersion {
                    version: version.clone(),
                    folder: folder.clone(),
                    installed_on: today.to_string(),
                },
            );
            state.dirty = true;
            state.status = format!("Installed {version}");

            // "Install & Open": the install was started by a launch request.
            if state.pending_launch.as_ref().map(|pending| pending.version.as_str()) == Some(version.as_str()) {
                if let Some(pending) = state.pending_launch.take() {
                    return vec![Effect::LaunchEditor {
                        folder: PathBuf::from(folder),
                        project: pending.project.map(PathBuf::from),
                    }];
                }
            }
        }
        InstallEvent::Failed { version, message } => {
            state.status = format!("Installing {version} failed: {message}");

            if state.pending_launch.as_ref().map(|pending| pending.version.as_str()) == Some(version.as_str()) {
                state.pending_launch = None;
            }
        }
        InstallEvent::Progress { .. } | InstallEvent::Phase { .. } => {}
    }

    Vec::new()
}

/// Checks that the release exists, has a build for this platform and is not installed,
/// starts a job for it, and returns the download request.
fn start_install(version: &str, models: &mut LauncherModels, today: &str) -> LauncherResult<Effect> {
    let release = releases_query::find(models.releases.data(), version).ok_or_else(|| {
        LauncherError::NotFound(format!("version {version} is not in the release list. Press Refresh first."))
    })?;
    let asset = release
        .asset
        .clone()
        .ok_or_else(|| LauncherError::Invalid(format!("version {version} has no build for this platform")))?;
    let checksum_url = release.checksum_url.clone();
    let release_version = release.version.clone();

    if installs_query::is_installed(models.installs.data(), version) {
        return Err(LauncherError::Invalid(format!("version {version} is already installed")));
    }

    JobsController.start(&mut models.jobs, version)?;

    Ok(Effect::Install(InstallRequest {
        version: release_version,
        asset,
        checksum_url,
        install_root: PathBuf::from(&models.settings.data().install_root),
        installed_on: today.to_string(),
    }))
}

fn apply(
    command: LauncherCommand,
    models: &mut LauncherModels,
    state: &mut ControllerState,
    today: &str,
) -> LauncherResult<Vec<Effect>> {
    let mut effects = Vec::new();

    match command {
        LauncherCommand::Refresh => {
            if !state.refreshing {
                state.refreshing = true;

                let settings = models.settings.data();

                effects.push(Effect::FetchReleases {
                    owner: settings.github_owner.clone(),
                    repo: settings.github_repo.clone(),
                });
            }
        }

        LauncherCommand::Install(version) => effects.push(start_install(&version, models, today)?),

        LauncherCommand::Uninstall(version) => {
            let root = PathBuf::from(&models.settings.data().install_root);
            let folder = installs_query::find(models.installs.data(), &version)
                .map(|installed| PathBuf::from(&installed.folder))
                .ok_or_else(|| LauncherError::NotFound(format!("version {version} is not installed")))?;

            // Never delete anything outside <install_root>/versions.
            if !paths::is_inside(&paths::versions_folder(&root), &folder) {
                return Err(LauncherError::Invalid(
                    "refusing to delete a folder outside the launcher's versions folder".to_string(),
                ));
            }

            InstallsController.remove(&mut models.installs, &version)?;
            state.dirty = true;
            effects.push(Effect::DeleteFolder(folder));
        }

        LauncherCommand::SetDefault(version) => {
            InstallsController.set_default(&mut models.installs, &version)?;
            state.dirty = true;
        }

        LauncherCommand::Launch { version, project } => {
            let installed_folder = installs_query::find(models.installs.data(), &version)
                .map(|installed| PathBuf::from(&installed.folder));

            match installed_folder {
                Some(folder) => {
                    if let Some(path) = &project {
                        // A project that is not in the list is fine, it just isn't remembered.
                        let _ = ProjectsController.touch(&mut models.projects, path, today);
                        state.dirty = true;
                    }

                    effects.push(Effect::LaunchEditor { folder, project: project.map(PathBuf::from) });
                }
                None => {
                    // Not installed yet: install it, and open the editor when it is done.
                    effects.push(start_install(&version, models, today)?);
                    state.pending_launch = Some(PendingLaunch { version, project });
                }
            }
        }

        LauncherCommand::AddProject { name, path, version } => {
            let engine_version = if version.is_empty() {
                models.installs.data().default_version.clone().unwrap_or_default()
            } else {
                version
            };

            ProjectsController.add(
                &mut models.projects,
                ProjectEntry { name, path, engine_version, last_opened: String::new() },
            )?;
            state.dirty = true;
        }

        LauncherCommand::RemoveProject(path) => {
            ProjectsController.remove(&mut models.projects, &path)?;
            state.dirty = true;
        }

        LauncherCommand::SetProjectVersion { path, version } => {
            ProjectsController.set_engine_version(&mut models.projects, &path, &version)?;
            state.dirty = true;
            effects.push(Effect::SaveProjectVersion { path: PathBuf::from(&path), version });
        }

        LauncherCommand::CreateProject { name, parent_folder, version, pipeline, aphelion_control } => {
            let version = if version.is_empty() {
                models
                    .installs
                    .data()
                    .default_version
                    .clone()
                    .ok_or_else(|| LauncherError::Invalid("install an engine version first (Versions tab)".to_string()))?
            } else {
                version
            };

            if parent_folder.trim().is_empty() {
                return Err(LauncherError::Invalid("choose the folder to create the project in".to_string()));
            }

            // The core controller checks the name. After this the pipeline can't change.
            let project = ProjectController.new_project(&name, &version, pipeline, aphelion_control, today)?;
            let data = project.data().clone();
            let root = PathBuf::from(parent_folder.trim()).join(&data.name);

            ProjectsController.add(
                &mut models.projects,
                ProjectEntry {
                    name: data.name.clone(),
                    path: root.to_string_lossy().to_string(),
                    engine_version: version,
                    last_opened: String::new(),
                },
            )?;
            state.dirty = true;
            effects.push(Effect::CreateProjectFolder { root, data });
        }

        LauncherCommand::ImportProject(path) => {
            if path.trim().is_empty() {
                return Err(LauncherError::Invalid("enter the project folder".to_string()));
            }

            effects.push(Effect::ReadProject(PathBuf::from(path.trim())));
        }

        LauncherCommand::SetRepository { owner, repo } => {
            SettingsController.set_repository(&mut models.settings, &owner, &repo)?;

            // The old list belongs to the old repository.
            ReleasesController.clear(&mut models.releases);
            state.dirty = true;
        }

        LauncherCommand::SetInstallRoot(root) => {
            SettingsController.set_install_root(&mut models.settings, &root)?;
            state.dirty = true;
        }

        LauncherCommand::SetIncludePrereleases(include) => {
            SettingsController.set_include_prereleases(&mut models.settings, include);
            state.dirty = true;
        }

        LauncherCommand::SetCheckUpdates(check) => {
            SettingsController.set_check_updates_on_start(&mut models.settings, check);
            state.dirty = true;
        }
    }

    Ok(effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::query as jobs_query;
    use crate::releases::{AssetInfo, ReleaseInfo};
    use crate::settings::LauncherSettingsData;
    use flux_core::mivc::Store;

    fn release(version: &str) -> ReleaseInfo {
        ReleaseInfo {
            tag: format!("v{version}"),
            version: version.to_string(),
            name: version.to_string(),
            prerelease: false,
            published: "2026-11-10".to_string(),
            notes: String::new(),
            asset: Some(AssetInfo {
                name: format!("flux-{version}-windows-x64.zip"),
                download_url: "https://example.com/flux.zip".to_string(),
                size: 10,
            }),
            checksum_url: None,
        }
    }

    fn models_with(versions: &[&str]) -> LauncherModels {
        let mut models = LauncherModels::default();

        models.settings = Store::new(LauncherSettingsData { install_root: "C:/Flux".to_string(), ..Default::default() });
        ReleasesController.set_releases(&mut models.releases, versions.iter().map(|v| release(v)).collect(), "2026-11-10");
        models
    }

    fn finished(version: &str, folder: &str) -> InstallEvent {
        InstallEvent::Finished { version: version.to_string(), folder: folder.to_string() }
    }

    #[test]
    fn installing_starts_a_job_and_asks_for_a_download() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        let effects = apply_all(vec![LauncherCommand::Install("0.7.0".to_string())], &mut models, &mut state, "2026-11-10");

        match &effects[..] {
            [Effect::Install(request)] => {
                assert_eq!(request.version, "0.7.0");
                assert_eq!(request.install_root, PathBuf::from("C:/Flux"));
            }
            other => panic!("expected one install effect, got {other:?}"),
        }

        assert!(jobs_query::is_active(models.jobs.data(), "0.7.0"));
    }

    #[test]
    fn the_same_version_is_not_downloaded_twice_at_once() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();
        let install = || vec![LauncherCommand::Install("0.7.0".to_string())];

        apply_all(install(), &mut models, &mut state, "2026-11-10");
        let second = apply_all(install(), &mut models, &mut state, "2026-11-10");

        assert!(second.is_empty());
        assert_ne!(state.status, "Ready");
    }

    #[test]
    fn an_unknown_version_shows_an_error_and_downloads_nothing() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        let effects = apply_all(vec![LauncherCommand::Install("9.9.9".to_string())], &mut models, &mut state, "2026-11-10");

        assert!(effects.is_empty());
        assert!(state.status.contains("9.9.9"));
    }

    #[test]
    fn a_finished_install_is_recorded_and_becomes_the_default() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        apply_all(vec![LauncherCommand::Install("0.7.0".to_string())], &mut models, &mut state, "2026-11-10");
        apply_install_event(&finished("0.7.0", "C:/Flux/versions/0.7.0"), &mut models, &mut state, "2026-11-10");

        assert!(installs_query::is_installed(models.installs.data(), "0.7.0"));
        assert_eq!(models.installs.data().default_version.as_deref(), Some("0.7.0"));
        assert!(state.dirty);
    }

    #[test]
    fn opening_a_project_with_a_missing_version_installs_it_then_opens_the_editor() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        let effects = apply_all(
            vec![LauncherCommand::Launch { version: "0.7.0".to_string(), project: Some("D:/game".to_string()) }],
            &mut models,
            &mut state,
            "2026-11-10",
        );

        assert!(matches!(effects[..], [Effect::Install(_)]));
        assert!(state.pending_launch.is_some());

        let after = apply_install_event(&finished("0.7.0", "C:/Flux/versions/0.7.0"), &mut models, &mut state, "2026-11-10");

        match &after[..] {
            [Effect::LaunchEditor { folder, project }] => {
                assert_eq!(folder, &PathBuf::from("C:/Flux/versions/0.7.0"));
                assert_eq!(project, &Some(PathBuf::from("D:/game")));
            }
            other => panic!("expected the editor to start, got {other:?}"),
        }

        assert!(state.pending_launch.is_none());
    }

    #[test]
    fn a_failed_install_cancels_the_pending_launch() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        apply_all(
            vec![LauncherCommand::Launch { version: "0.7.0".to_string(), project: None }],
            &mut models,
            &mut state,
            "2026-11-10",
        );
        let after = apply_install_event(
            &InstallEvent::Failed { version: "0.7.0".to_string(), message: "no network".to_string() },
            &mut models,
            &mut state,
            "2026-11-10",
        );

        assert!(after.is_empty());
        assert!(state.pending_launch.is_none());
        assert!(state.status.contains("no network"));
    }

    #[test]
    fn launching_an_installed_version_starts_the_editor_and_remembers_the_project() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        InstallsController.register(
            &mut models.installs,
            InstalledVersion { version: "0.7.0".to_string(), folder: "C:/Flux/versions/0.7.0".to_string(), installed_on: String::new() },
        );
        ProjectsController
            .add(
                &mut models.projects,
                ProjectEntry {
                    name: "Game".to_string(),
                    path: "D:/game".to_string(),
                    engine_version: "0.7.0".to_string(),
                    last_opened: String::new(),
                },
            )
            .unwrap();

        let effects = apply_all(
            vec![LauncherCommand::Launch { version: "0.7.0".to_string(), project: Some("D:/game".to_string()) }],
            &mut models,
            &mut state,
            "2026-11-10",
        );

        assert!(matches!(effects[..], [Effect::LaunchEditor { .. }]));
        assert_eq!(models.projects.data().projects[0].last_opened, "2026-11-10");
    }

    #[test]
    fn uninstalling_never_deletes_outside_the_versions_folder() {
        let mut models = models_with(&[]);
        let mut state = ControllerState::default();

        for (version, folder) in [("0.7.0", "D:/elsewhere/0.7.0"), ("0.8.0", "C:/Flux/versions/0.8.0")] {
            InstallsController.register(
                &mut models.installs,
                InstalledVersion { version: version.to_string(), folder: folder.to_string(), installed_on: String::new() },
            );
        }

        let refused = apply_all(vec![LauncherCommand::Uninstall("0.7.0".to_string())], &mut models, &mut state, "2026-11-10");

        assert!(refused.is_empty());
        assert!(installs_query::is_installed(models.installs.data(), "0.7.0"));

        let allowed = apply_all(vec![LauncherCommand::Uninstall("0.8.0".to_string())], &mut models, &mut state, "2026-11-10");

        assert!(matches!(&allowed[..], [Effect::DeleteFolder(path)] if path == &PathBuf::from("C:/Flux/versions/0.8.0")));
        assert!(!installs_query::is_installed(models.installs.data(), "0.8.0"));
    }

    #[test]
    fn the_release_list_from_github_is_stored() {
        let mut models = LauncherModels::default();
        let mut state = ControllerState { refreshing: true, ..Default::default() };

        apply_releases_json(parse::tests::SAMPLE, &mut models, &mut state, "2026-11-10");

        assert_eq!(models.releases.data().releases.len(), 2);
        assert_eq!(models.releases.data().fetched_on, "2026-11-10");
        assert!(!state.refreshing);
    }

    #[test]
    fn changing_the_repository_forgets_the_old_release_list() {
        let mut models = models_with(&["0.7.0"]);
        let mut state = ControllerState::default();

        apply_all(
            vec![LauncherCommand::SetRepository { owner: "someone".to_string(), repo: "engine".to_string() }],
            &mut models,
            &mut state,
            "2026-11-10",
        );

        assert!(models.releases.data().releases.is_empty());
        assert_eq!(models.settings.data().github_repo, "engine");
    }

    #[test]
    fn a_second_refresh_is_ignored_while_one_is_running() {
        let mut models = LauncherModels::default();
        let mut state = ControllerState::default();

        let first = apply_all(vec![LauncherCommand::Refresh], &mut models, &mut state, "2026-11-10");
        let second = apply_all(vec![LauncherCommand::Refresh], &mut models, &mut state, "2026-11-10");

        assert_eq!(first.len(), 1);
        assert!(second.is_empty());
    }

    #[test]
    fn creating_a_project_registers_it_and_asks_for_the_folder() {
        let mut models = models_with(&[]);
        let mut state = ControllerState::default();

        let effects = apply_all(
            vec![LauncherCommand::CreateProject {
                name: "Space Game".to_string(),
                parent_folder: "D:/Games".to_string(),
                version: "0.8.0".to_string(),
                pipeline: GraphicsPipeline::Heavy,
                aphelion_control: true,
            }],
            &mut models,
            &mut state,
            "2026-11-10",
        );

        match &effects[..] {
            [Effect::CreateProjectFolder { root, data }] => {
                assert_eq!(root, &PathBuf::from("D:/Games").join("Space Game"));
                assert_eq!(data.pipeline, GraphicsPipeline::Heavy);
                assert!(data.aphelion_control);
                assert_eq!(data.engine_version, "0.8.0");
            }
            other => panic!("expected a create effect, got {other:?}"),
        }

        assert_eq!(models.projects.data().projects.len(), 1);
    }

    #[test]
    fn a_bad_project_name_or_a_missing_folder_creates_nothing() {
        let mut models = models_with(&[]);
        let mut state = ControllerState::default();
        let create = |name: &str, parent: &str| LauncherCommand::CreateProject {
            name: name.to_string(),
            parent_folder: parent.to_string(),
            version: "0.8.0".to_string(),
            pipeline: GraphicsPipeline::Simple,
            aphelion_control: false,
        };

        let bad_name = apply_all(vec![create("a/b", "D:/Games")], &mut models, &mut state, "2026-11-10");
        let no_folder = apply_all(vec![create("Fine", "  ")], &mut models, &mut state, "2026-11-10");

        assert!(bad_name.is_empty() && no_folder.is_empty());
        assert!(models.projects.data().projects.is_empty());
    }

    #[test]
    fn a_project_without_a_chosen_version_uses_the_default_install() {
        let mut models = models_with(&[]);
        let mut state = ControllerState::default();
        let create = || LauncherCommand::CreateProject {
            name: "Game".to_string(),
            parent_folder: "D:/Games".to_string(),
            version: String::new(),
            pipeline: GraphicsPipeline::Simple,
            aphelion_control: false,
        };

        assert!(apply_all(vec![create()], &mut models, &mut state, "2026-11-10").is_empty());

        InstallsController.register(
            &mut models.installs,
            InstalledVersion { version: "0.8.0".to_string(), folder: String::new(), installed_on: String::new() },
        );

        assert_eq!(apply_all(vec![create()], &mut models, &mut state, "2026-11-10").len(), 1);
        assert_eq!(models.projects.data().projects[0].engine_version, "0.8.0");
    }

    #[test]
    fn changing_a_projects_version_also_updates_its_own_file() {
        let mut models = models_with(&[]);
        let mut state = ControllerState::default();

        ProjectsController
            .add(
                &mut models.projects,
                ProjectEntry {
                    name: "A".to_string(),
                    path: "D:/a".to_string(),
                    engine_version: "0.7.0".to_string(),
                    last_opened: String::new(),
                },
            )
            .unwrap();

        let effects = apply_all(
            vec![LauncherCommand::SetProjectVersion { path: "D:/a".to_string(), version: "0.8.0".to_string() }],
            &mut models,
            &mut state,
            "2026-11-10",
        );

        assert!(matches!(&effects[..], [Effect::SaveProjectVersion { version, .. }] if version == "0.8.0"));
    }

    #[test]
    fn importing_asks_for_the_project_file_to_be_read() {
        let mut models = LauncherModels::default();
        let mut state = ControllerState::default();

        let effects = apply_all(vec![LauncherCommand::ImportProject("D:/old game".to_string())], &mut models, &mut state, "2026-11-10");

        assert!(matches!(&effects[..], [Effect::ReadProject(path)] if path == &PathBuf::from("D:/old game")));
    }
}
