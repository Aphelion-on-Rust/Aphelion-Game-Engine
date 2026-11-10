use flux_core::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    /// Where it was unpacked: <install_root>/versions/<version>.
    pub folder: String,
    pub installed_on: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InstallsData {
    pub versions: Vec<InstalledVersion>,
    pub default_version: Option<String>,
}

impl Interface for InstallsData {
    const FORMAT: &'static str = "launcher_installs";
    const VERSION: u32 = 1;
}
