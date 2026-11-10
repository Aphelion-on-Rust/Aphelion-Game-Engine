//! Reads GitHub's "list releases" answer.
//!
//! This is the ONLY place that touches JSON, because GitHub sends JSON.
//! Everything the launcher saves itself is YAML.

use super::interface::{AssetInfo, ReleaseInfo};
use crate::error::{LauncherError, LauncherResult};
use crate::version::Version;
use serde::Deserialize;

/// The platform word a release file name must contain, like "windows" in
/// flux-aphelion-0.7.0-windows-x64.zip
pub const PLATFORM: &str = if cfg!(windows) {
    "windows"
} else if cfg!(target_os = "macos") {
    "macos"
} else {
    "linux"
};

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    published_at: Option<String>,
    body: Option<String>,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// The .zip for this platform.
pub fn choose_asset<'a>(assets: &'a [AssetInfo], platform: &str) -> Option<&'a AssetInfo> {
    assets.iter().find(|asset| {
        let name = asset.name.to_lowercase();

        name.ends_with(".zip") && name.contains(platform)
    })
}

/// Turns GitHub's text into our release list. Drafts and tags that are not
/// versions (like "nightly") are skipped.
pub fn parse_releases(json: &str, platform: &str) -> LauncherResult<Vec<ReleaseInfo>> {
    let raw: Vec<GithubRelease> =
        serde_json::from_str(json).map_err(|error| LauncherError::Json(error.to_string()))?;
    let mut releases = Vec::new();

    for release in raw {
        if release.draft {
            continue;
        }

        let Some(version) = Version::parse(&release.tag_name) else {
            continue;
        };

        let assets: Vec<AssetInfo> = release
            .assets
            .iter()
            .map(|asset| AssetInfo {
                name: asset.name.clone(),
                download_url: asset.browser_download_url.clone(),
                size: asset.size,
            })
            .collect();

        let asset = choose_asset(&assets, platform).cloned();
        let checksum_url = asset.as_ref().and_then(|asset| {
            let wanted = format!("{}.sha256", asset.name);

            assets.iter().find(|candidate| candidate.name == wanted).map(|found| found.download_url.clone())
        });

        releases.push(ReleaseInfo {
            tag: release.tag_name.clone(),
            version: version.to_string(),
            name: release.name.unwrap_or_else(|| release.tag_name.clone()),
            prerelease: release.prerelease || version.is_prerelease(),
            published: release.published_at.unwrap_or_default().chars().take(10).collect(),
            notes: release.body.unwrap_or_default(),
            asset,
            checksum_url,
        });
    }

    Ok(releases)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const SAMPLE: &str = r#"[
      {"tag_name":"v0.8.0","name":"Flux 0.8.0","prerelease":false,"draft":false,
       "published_at":"2026-11-20T10:00:00Z","body":"Notes",
       "assets":[
         {"name":"flux-aphelion-0.8.0-windows-x64.zip","browser_download_url":"https://example.com/win.zip","size":1000},
         {"name":"flux-aphelion-0.8.0-windows-x64.zip.sha256","browser_download_url":"https://example.com/win.zip.sha256","size":90},
         {"name":"flux-aphelion-0.8.0-linux-x64.zip","browser_download_url":"https://example.com/linux.zip","size":900}]},
      {"tag_name":"v0.9.0-beta.1","name":null,"prerelease":true,"draft":false,
       "published_at":null,"body":null,"assets":[]},
      {"tag_name":"v0.7.5","name":"draft","prerelease":false,"draft":true,
       "published_at":null,"body":null,"assets":[]},
      {"tag_name":"nightly","name":"x","prerelease":false,"draft":false,
       "published_at":null,"body":null,"assets":[]}
    ]"#;

    #[test]
    fn drafts_and_non_version_tags_are_skipped() {
        let releases = parse_releases(SAMPLE, "windows").unwrap();
        let versions: Vec<&str> = releases.iter().map(|release| release.version.as_str()).collect();

        assert_eq!(versions, vec!["0.8.0", "0.9.0-beta.1"]);
    }

    #[test]
    fn the_right_platform_file_and_checksum_are_found() {
        let releases = parse_releases(SAMPLE, "windows").unwrap();
        let first = &releases[0];

        assert_eq!(first.asset.as_ref().unwrap().download_url, "https://example.com/win.zip");
        assert_eq!(first.checksum_url.as_deref(), Some("https://example.com/win.zip.sha256"));
        assert_eq!(first.published, "2026-11-20");

        let linux = parse_releases(SAMPLE, "linux").unwrap();

        assert_eq!(linux[0].asset.as_ref().unwrap().download_url, "https://example.com/linux.zip");
        assert!(linux[0].checksum_url.is_none());
    }

    #[test]
    fn a_release_without_a_build_has_no_asset_and_pre_releases_are_flagged() {
        let releases = parse_releases(SAMPLE, "windows").unwrap();

        assert!(releases[1].asset.is_none());
        assert!(releases[1].prerelease);
        assert_eq!(releases[1].name, "v0.9.0-beta.1");
    }

    #[test]
    fn broken_json_is_an_error_not_a_crash() {
        assert!(parse_releases("not json", "windows").is_err());
    }
}
