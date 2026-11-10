use crate::error::{LauncherError, LauncherResult};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub(super) const USER_AGENT: &str = "FluxAphelion-Launcher";

pub(super) fn network_error(error: ureq::Error) -> LauncherError {
    match error {
        ureq::Error::Status(code, _) => LauncherError::Network(format!("the server answered with status {code}")),
        other => LauncherError::Network(other.to_string()),
    }
}

fn request(url: &str) -> LauncherResult<ureq::Response> {
    ureq::get(url).set("User-Agent", USER_AGENT).call().map_err(network_error)
}

/// Downloads a small text file (like a .sha256 file).
pub fn download_text(url: &str) -> LauncherResult<String> {
    Ok(request(url)?.into_string()?)
}

/// Downloads a file. It is written to `<name>.part` first and renamed when complete,
/// so an interrupted download never looks like a finished one.
/// `on_progress(downloaded, total)` is called for every chunk (total is 0 if unknown).
pub fn download_file(
    url: &str,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64, u64),
) -> LauncherResult<()> {
    let response = request(url)?;
    let total = response.header("Content-Length").and_then(|text| text.parse::<u64>().ok()).unwrap_or(0);
    let mut reader = response.into_reader();

    if let Some(folder) = destination.parent() {
        fs::create_dir_all(folder)?;
    }

    let partial = destination.with_extension("part");
    let mut file = File::create(&partial)?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut downloaded = 0u64;

    loop {
        let read = reader.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        file.write_all(&buffer[..read])?;
        downloaded += read as u64;
        on_progress(downloaded, total);
    }

    file.flush()?;
    drop(file);
    fs::rename(&partial, destination)?;
    Ok(())
}
