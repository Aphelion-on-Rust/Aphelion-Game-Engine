use flux_core::mivc::Interface;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LauncherSettingsData {
    /// GitHub account that publishes the releases.
    pub github_owner: String,
    /// GitHub repository that publishes the releases.
    pub github_repo: String,
    /// Also list pre-releases (beta versions).
    pub include_prereleases: bool,
    /// Engine versions are installed in <install_root>/versions/<version>.
    pub install_root: String,
    pub check_updates_on_start: bool,
}

/// Windows: %LOCALAPPDATA%\FluxAphelion. Elsewhere: $HOME/.local/share/FluxAphelion.
pub fn default_install_root() -> String {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("FluxAphelion").to_string_lossy().to_string()
}

impl Default for LauncherSettingsData {
    fn default() -> Self {
        Self {
            // Change these in the launcher settings if the repository has another name.
            github_owner: "atropinex".to_string(),
            github_repo: "flux-aphelion".to_string(),
            include_prereleases: false,
            install_root: default_install_root(),
            check_updates_on_start: true,
        }
    }
}

impl Interface for LauncherSettingsData {
    const FORMAT: &'static str = "launcher_settings";
    const VERSION: u32 = 1;
}
