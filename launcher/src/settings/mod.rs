//! Launcher settings: which GitHub repository to read, where to install, pre-releases.

pub mod controller;
pub mod interface;
pub mod model;

pub use controller::SettingsController;
pub use interface::LauncherSettingsData;
pub use model::SettingsModel;
