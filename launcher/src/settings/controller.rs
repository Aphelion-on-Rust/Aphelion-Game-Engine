use super::model::SettingsModel;
use crate::error::{LauncherError, LauncherResult};
use flux_core::mivc::Controller;

pub struct SettingsController;

impl Controller for SettingsController {}

/// A GitHub account or repository name: one word, no slashes.
fn clean_name(text: &str, what: &str) -> LauncherResult<String> {
    let text = text.trim();

    if text.is_empty() || text.contains('/') || text.chars().any(char::is_whitespace) {
        return Err(LauncherError::Invalid(format!("the GitHub {what} must be one word without slashes")));
    }

    Ok(text.to_string())
}

impl SettingsController {
    pub fn set_repository(&self, model: &mut SettingsModel, owner: &str, repo: &str) -> LauncherResult<()> {
        let owner = clean_name(owner, "owner")?;
        let repo = clean_name(repo, "repository")?;
        let data = self.write(model);

        data.github_owner = owner;
        data.github_repo = repo;
        Ok(())
    }

    pub fn set_install_root(&self, model: &mut SettingsModel, root: &str) -> LauncherResult<()> {
        let root = root.trim();

        if root.is_empty() {
            return Err(LauncherError::Invalid("the install folder can't be empty".to_string()));
        }

        self.write(model).install_root = root.to_string();
        Ok(())
    }

    pub fn set_include_prereleases(&self, model: &mut SettingsModel, include: bool) {
        self.write(model).include_prereleases = include;
    }

    pub fn set_check_updates_on_start(&self, model: &mut SettingsModel, check: bool) {
        self.write(model).check_updates_on_start = check;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::mivc::Model;

    #[test]
    fn a_repository_must_be_two_clean_words() {
        let mut model = SettingsModel::default();

        assert!(SettingsController.set_repository(&mut model, "someone", "engine").is_ok());
        assert_eq!(model.data().github_owner, "someone");
        assert!(SettingsController.set_repository(&mut model, "a/b", "engine").is_err());
        assert!(SettingsController.set_repository(&mut model, "someone", "my engine").is_err());
        assert!(SettingsController.set_repository(&mut model, "", "engine").is_err());
        assert_eq!(model.data().github_owner, "someone");
    }

    #[test]
    fn the_install_folder_can_not_be_empty() {
        let mut model = SettingsModel::default();

        assert!(SettingsController.set_install_root(&mut model, "  ").is_err());
        assert!(SettingsController.set_install_root(&mut model, "D:/Flux").is_ok());
    }
}
