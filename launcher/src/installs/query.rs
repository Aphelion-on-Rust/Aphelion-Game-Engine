use super::interface::{InstalledVersion, InstallsData};
use crate::version::Version;

pub fn find<'a>(data: &'a InstallsData, version: &str) -> Option<&'a InstalledVersion> {
    data.versions.iter().find(|installed| installed.version == version)
}

pub fn is_installed(data: &InstallsData, version: &str) -> bool {
    find(data, version).is_some()
}

/// The highest installed version.
pub fn newest(data: &InstallsData) -> Option<&InstalledVersion> {
    data.versions.iter().max_by_key(|installed| Version::parse_or_zero(&installed.version))
}
