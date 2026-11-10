use crate::installs::InstallsModel;
use crate::jobs::JobsModel;
use crate::projects::ProjectsModel;
use crate::releases::ReleasesModel;
use crate::settings::SettingsModel;

/// Every model the launcher uses. Views get this as `&LauncherModels`.
#[derive(Default)]
pub struct LauncherModels {
    pub settings: SettingsModel,
    pub releases: ReleasesModel,
    pub installs: InstallsModel,
    pub projects: ProjectsModel,
    pub jobs: JobsModel,
}
