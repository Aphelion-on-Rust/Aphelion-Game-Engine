use super::interface::{ReleaseCacheData, ReleaseInfo};
use crate::installs::{query as installs_query, InstallsData};
use crate::version::Version;

/// The releases to show. Pre-releases only if asked for.
pub fn visible(data: &ReleaseCacheData, include_prereleases: bool) -> Vec<&ReleaseInfo> {
    data.releases
        .iter()
        .filter(|release| include_prereleases || !release.prerelease)
        .collect()
}

pub fn find<'a>(data: &'a ReleaseCacheData, version: &str) -> Option<&'a ReleaseInfo> {
    data.releases.iter().find(|release| release.version == version)
}

/// The newest installable release, if it is newer than everything installed
/// (or nothing is installed yet).
pub fn update_available<'a>(
    releases: &'a ReleaseCacheData,
    installs: &InstallsData,
    include_prereleases: bool,
) -> Option<&'a ReleaseInfo> {
    let latest = visible(releases, include_prereleases).into_iter().find(|release| release.asset.is_some())?;

    match installs_query::newest(installs) {
        Some(newest) if Version::parse_or_zero(&newest.version) >= Version::parse_or_zero(&latest.version) => None,
        _ => Some(latest),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installs::InstalledVersion;
    use crate::releases::AssetInfo;

    fn release(version: &str, prerelease: bool) -> ReleaseInfo {
        ReleaseInfo {
            tag: format!("v{version}"),
            version: version.to_string(),
            name: version.to_string(),
            prerelease,
            published: String::new(),
            notes: String::new(),
            asset: Some(AssetInfo { name: "a.zip".to_string(), download_url: "u".to_string(), size: 1 }),
            checksum_url: None,
        }
    }

    fn installed(version: &str) -> InstalledVersion {
        InstalledVersion { version: version.to_string(), folder: String::new(), installed_on: String::new() }
    }

    #[test]
    fn pre_releases_are_hidden_unless_asked_for() {
        let data = ReleaseCacheData { releases: vec![release("0.9.0-beta.1", true), release("0.8.0", false)], fetched_on: String::new() };

        assert_eq!(visible(&data, false).len(), 1);
        assert_eq!(visible(&data, true).len(), 2);
    }

    #[test]
    fn an_update_is_offered_only_when_something_newer_exists() {
        let data = ReleaseCacheData { releases: vec![release("0.8.0", false)], fetched_on: String::new() };
        let mut installs = InstallsData::default();

        assert!(update_available(&data, &installs, false).is_some());

        installs.versions.push(installed("0.7.0"));
        assert!(update_available(&data, &installs, false).is_some());

        installs.versions.push(installed("0.8.0"));
        assert!(update_available(&data, &installs, false).is_none());
    }
}
