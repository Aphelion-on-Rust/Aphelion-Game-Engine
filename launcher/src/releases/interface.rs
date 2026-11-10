use flux_core::mivc::Interface;
use serde::{Deserialize, Serialize};

/// The file to download for this computer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    pub download_url: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// The git tag, for example "v0.7.0".
    pub tag: String,
    /// The version without the "v", for example "0.7.0". Used as the install folder name.
    pub version: String,
    pub name: String,
    pub prerelease: bool,
    /// Date only, like 2026-11-10.
    pub published: String,
    pub notes: String,
    /// None if the release has no build for this platform.
    pub asset: Option<AssetInfo>,
    /// Link to the .sha256 file next to the download, if the release has one.
    pub checksum_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReleaseCacheData {
    /// Newest first.
    pub releases: Vec<ReleaseInfo>,
    /// Date of the last successful refresh. Empty if never.
    pub fetched_on: String,
}

impl Interface for ReleaseCacheData {
    const FORMAT: &'static str = "launcher_release_cache";
    const VERSION: u32 = 1;
}
