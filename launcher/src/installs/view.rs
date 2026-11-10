use super::interface::InstallsData;
use super::model::InstallsModel;
use super::query;
use crate::jobs::{query as jobs_query, JobPhase, JobsData, JobsModel};
use crate::releases::{query as releases_query, ReleaseInfo, ReleasesModel};
use crate::settings::SettingsModel;
use crate::version::Version;
use flux_core::mivc::{Model, View};

#[derive(Clone, Debug, PartialEq)]
pub enum RowState {
    Installed { is_default: bool },
    Installing { phase: JobPhase, fraction: f32 },
    Failed(String),
    Available,
    /// The release has no file for this platform.
    NoBuild,
}

/// One line of the versions list.
#[derive(Clone, Debug, PartialEq)]
pub struct InstallRow {
    pub version: String,
    pub prerelease: bool,
    pub published: String,
    pub size_bytes: u64,
    pub state: RowState,
}

/// Builds the versions list: every release from GitHub plus anything installed that
/// is no longer listed. It only reads.
#[derive(Default)]
pub struct InstallsView {
    rows: Vec<InstallRow>,
}

impl InstallsView {
    pub fn rows(&self) -> &[InstallRow] {
        &self.rows
    }
}

fn state_for(release: &ReleaseInfo, installs: &InstallsData, jobs: &JobsData) -> RowState {
    if query::is_installed(installs, &release.version) {
        return RowState::Installed { is_default: installs.default_version.as_deref() == Some(release.version.as_str()) };
    }

    if let Some(job) = jobs_query::find(jobs, &release.version) {
        match job.phase {
            JobPhase::Failed => return RowState::Failed(job.message.clone()),
            JobPhase::Finished => {}
            phase => return RowState::Installing { phase, fraction: jobs_query::fraction(job) },
        }
    }

    if release.asset.is_none() {
        RowState::NoBuild
    } else {
        RowState::Available
    }
}

impl View for InstallsView {
    type Models<'a> = (&'a ReleasesModel, &'a InstallsModel, &'a JobsModel, &'a SettingsModel);

    fn render(&mut self, models: Self::Models<'_>) {
        let (releases, installs, jobs, settings) = models;
        let include = settings.data().include_prereleases;
        let mut listed: Vec<String> = Vec::new();

        self.rows.clear();

        for release in releases_query::visible(releases.data(), include) {
            listed.push(release.version.clone());
            self.rows.push(InstallRow {
                version: release.version.clone(),
                prerelease: release.prerelease,
                published: release.published.clone(),
                size_bytes: release.asset.as_ref().map_or(0, |asset| asset.size),
                state: state_for(release, installs.data(), jobs.data()),
            });
        }

        for installed in &installs.data().versions {
            if !listed.contains(&installed.version) {
                self.rows.push(InstallRow {
                    version: installed.version.clone(),
                    prerelease: Version::parse_or_zero(&installed.version).is_prerelease(),
                    published: String::new(),
                    size_bytes: 0,
                    state: RowState::Installed {
                        is_default: installs.data().default_version.as_deref() == Some(installed.version.as_str()),
                    },
                });
            }
        }

        self.rows.sort_by(|a, b| Version::parse_or_zero(&b.version).cmp(&Version::parse_or_zero(&a.version)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installs::{InstalledVersion, InstallsController};
    use crate::jobs::JobsController;
    use crate::releases::{AssetInfo, ReleasesController};

    fn release(version: &str, with_asset: bool) -> ReleaseInfo {
        ReleaseInfo {
            tag: format!("v{version}"),
            version: version.to_string(),
            name: version.to_string(),
            prerelease: false,
            published: "2026-11-10".to_string(),
            notes: String::new(),
            asset: with_asset.then(|| AssetInfo { name: "a.zip".to_string(), download_url: "u".to_string(), size: 5 }),
            checksum_url: None,
        }
    }

    fn installed(version: &str) -> InstalledVersion {
        InstalledVersion { version: version.to_string(), folder: String::new(), installed_on: String::new() }
    }

    #[test]
    fn rows_show_installed_installing_available_and_unlisted_versions() {
        let mut releases = ReleasesModel::default();
        let mut installs = InstallsModel::default();
        let mut jobs = JobsModel::default();
        let settings = SettingsModel::default();

        ReleasesController.set_releases(
            &mut releases,
            vec![release("0.7.0", true), release("0.8.0", true), release("0.9.0", true), release("0.10.0", false)],
            "2026-11-10",
        );
        InstallsController.register(&mut installs, installed("0.7.0"));
        InstallsController.register(&mut installs, installed("0.1.0"));
        JobsController.start(&mut jobs, "0.8.0").unwrap();

        let mut view = InstallsView::default();
        view.render((&releases, &installs, &jobs, &settings));

        let states: Vec<(&str, RowState)> = view.rows().iter().map(|row| (row.version.as_str(), row.state.clone())).collect();

        assert_eq!(states[0], ("0.10.0", RowState::NoBuild));
        assert_eq!(states[1], ("0.9.0", RowState::Available));
        assert!(matches!(states[2], ("0.8.0", RowState::Installing { .. })));
        assert_eq!(states[3], ("0.7.0", RowState::Installed { is_default: true }));
        assert_eq!(states[4], ("0.1.0", RowState::Installed { is_default: false }));
    }
}
