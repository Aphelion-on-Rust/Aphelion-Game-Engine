use super::interface::ReleaseInfo;
use super::model::ReleasesModel;
use crate::version::Version;
use flux_core::mivc::Controller;

pub struct ReleasesController;

impl Controller for ReleasesController {}

impl ReleasesController {
    /// Stores a fresh list, newest first.
    pub fn set_releases(&self, model: &mut ReleasesModel, mut releases: Vec<ReleaseInfo>, today: &str) {
        releases.sort_by(|a, b| Version::parse_or_zero(&b.version).cmp(&Version::parse_or_zero(&a.version)));

        let data = self.write(model);

        data.releases = releases;
        data.fetched_on = today.to_string();
    }

    /// Forgets the list, for example after the repository was changed.
    pub fn clear(&self, model: &mut ReleasesModel) {
        let data = self.write(model);

        data.releases.clear();
        data.fetched_on.clear();
    }
}
