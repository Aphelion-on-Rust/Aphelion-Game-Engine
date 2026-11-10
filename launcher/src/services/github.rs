use super::download::{network_error, USER_AGENT};
use crate::error::{LauncherError, LauncherResult};

/// Asks GitHub for the releases of a repository. Returns GitHub's raw answer,
/// `releases::parse` turns it into our list.
pub fn fetch_releases_json(owner: &str, repo: &str) -> LauncherResult<String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=30");

    let response = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| match error {
            ureq::Error::Status(404, _) => LauncherError::NotFound(format!(
                "no releases found for {owner}/{repo}. Is the repository public, and are the names right?"
            )),
            ureq::Error::Status(403, _) => {
                LauncherError::Network("GitHub refused the request (too many requests?). Try again later.".to_string())
            }
            other => network_error(other),
        })?;

    Ok(response.into_string()?)
}
