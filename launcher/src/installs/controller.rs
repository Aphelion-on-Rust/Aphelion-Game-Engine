use super::interface::InstalledVersion;
use super::model::InstallsModel;
use super::query;
use crate::error::{LauncherError, LauncherResult};
use flux_core::mivc::{Controller, Model};

pub struct InstallsController;

impl Controller for InstallsController {}

impl InstallsController {
    /// Records an installed version. The first one installed becomes the default.
    pub fn register(&self, model: &mut InstallsModel, entry: InstalledVersion) {
        let version = entry.version.clone();
        let data = self.write(model);

        match data.versions.iter_mut().find(|installed| installed.version == version) {
            Some(existing) => *existing = entry,
            None => data.versions.push(entry),
        }

        if data.default_version.is_none() {
            data.default_version = Some(version);
        }
    }

    /// Forgets a version and returns it (so the caller knows which folder to delete).
    /// If it was the default, the newest remaining version becomes the default.
    pub fn remove(&self, model: &mut InstallsModel, version: &str) -> LauncherResult<InstalledVersion> {
        if !query::is_installed(model.data(), version) {
            return Err(LauncherError::NotFound(format!("version {version} is not installed")));
        }

        let data = self.write(model);
        let position = data.versions.iter().position(|installed| installed.version == version);
        let removed = data.versions.remove(position.unwrap_or(0));

        if data.default_version.as_deref() == Some(version) {
            data.default_version = query::newest(data).map(|installed| installed.version.clone());
        }

        Ok(removed)
    }

    pub fn set_default(&self, model: &mut InstallsModel, version: &str) -> LauncherResult<()> {
        if !query::is_installed(model.data(), version) {
            return Err(LauncherError::NotFound(format!("version {version} is not installed")));
        }

        self.write(model).default_version = Some(version.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(version: &str) -> InstalledVersion {
        InstalledVersion {
            version: version.to_string(),
            folder: format!("C:/Flux/versions/{version}"),
            installed_on: "2026-11-10".to_string(),
        }
    }

    #[test]
    fn the_first_install_becomes_the_default() {
        let mut model = InstallsModel::default();

        InstallsController.register(&mut model, entry("0.7.0"));
        InstallsController.register(&mut model, entry("0.8.0"));

        assert_eq!(model.data().default_version.as_deref(), Some("0.7.0"));
        assert_eq!(model.data().versions.len(), 2);
    }

    #[test]
    fn installing_the_same_version_again_replaces_it() {
        let mut model = InstallsModel::default();

        InstallsController.register(&mut model, entry("0.7.0"));
        InstallsController.register(&mut model, entry("0.7.0"));

        assert_eq!(model.data().versions.len(), 1);
    }

    #[test]
    fn removing_the_default_picks_the_newest_remaining() {
        let mut model = InstallsModel::default();

        for version in ["0.7.0", "0.9.0", "0.8.0"] {
            InstallsController.register(&mut model, entry(version));
        }

        InstallsController.remove(&mut model, "0.7.0").unwrap();

        assert_eq!(model.data().default_version.as_deref(), Some("0.9.0"));
        assert!(InstallsController.remove(&mut model, "0.7.0").is_err());
    }

    #[test]
    fn only_installed_versions_can_be_the_default() {
        let mut model = InstallsModel::default();

        InstallsController.register(&mut model, entry("0.7.0"));

        assert!(InstallsController.set_default(&mut model, "0.8.0").is_err());
        assert!(InstallsController.set_default(&mut model, "0.7.0").is_ok());
    }
}
