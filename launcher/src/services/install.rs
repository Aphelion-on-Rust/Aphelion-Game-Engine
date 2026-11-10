use super::{checksum, download, extract, paths, BackgroundEvent};
use crate::error::{LauncherError, LauncherResult};
use crate::jobs::{InstallEvent, JobPhase};
use crate::releases::AssetInfo;
use crate::version::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// Everything the install thread needs to know.
#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub version: String,
    pub asset: AssetInfo,
    pub checksum_url: Option<String>,
    pub install_root: PathBuf,
    pub installed_on: String,
}

/// Installs a version on a background thread so the window stays responsive.
/// Progress comes back as `BackgroundEvent::Install`. `wake` asks the window to redraw.
pub fn spawn_install(request: InstallRequest, events: Sender<BackgroundEvent>, wake: Box<dyn Fn() + Send>) {
    std::thread::spawn(move || {
        let send = |event: InstallEvent| {
            let _ = events.send(BackgroundEvent::Install(event));
            wake();
        };

        match run_install(&request, &send) {
            Ok(folder) => send(InstallEvent::Finished { version: request.version.clone(), folder }),
            Err(error) => send(InstallEvent::Failed { version: request.version.clone(), message: error.to_string() }),
        }
    });
}

/// download -> verify -> unpack -> move into place. Returns the install folder.
fn run_install(request: &InstallRequest, send: &dyn Fn(InstallEvent)) -> LauncherResult<String> {
    let version = request.version.as_str();

    // The version becomes a folder name, so it must really be a version.
    if Version::parse(version).is_none() {
        return Err(LauncherError::Invalid(format!("\"{version}\" is not a valid version")));
    }

    let set_phase = |phase: JobPhase| send(InstallEvent::Phase { version: version.to_string(), phase });
    let file_name = Path::new(&request.asset.name)
        .file_name()
        .ok_or_else(|| LauncherError::Invalid("the release file has no name".to_string()))?;
    let downloads = paths::downloads_folder(&request.install_root);
    let zip_path = downloads.join(file_name);

    fs::create_dir_all(&downloads)?;

    // 1. Download.
    set_phase(JobPhase::Downloading);

    let mut last_report: Option<Instant> = None;

    download::download_file(&request.asset.download_url, &zip_path, &mut |done, total| {
        let finished = total > 0 && done == total;
        let due = last_report.map_or(true, |time| time.elapsed() >= Duration::from_millis(100));

        if due || finished {
            last_report = Some(Instant::now());
            send(InstallEvent::Progress { version: version.to_string(), downloaded: done, total });
        }
    })?;

    // 2. Verify, if the release has a .sha256 file.
    set_phase(JobPhase::Verifying);

    if let Some(url) = &request.checksum_url {
        let text = download::download_text(url)?;
        let expected = checksum::parse_checksum_file(&text)
            .ok_or_else(|| LauncherError::Invalid("the checksum file could not be read".to_string()))?;

        checksum::verify_file(&zip_path, &expected)?;
    }

    // 3. Unpack into a staging folder first.
    set_phase(JobPhase::Extracting);

    let staging = paths::staging_folder(&request.install_root, version);
    let final_folder = paths::version_folder(&request.install_root, version);

    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }

    extract::extract_zip(&zip_path, &staging, &mut |_, _| {})?;

    if !paths::editor_exe(&staging).exists() {
        let _ = fs::remove_dir_all(&staging);
        return Err(LauncherError::Invalid(
            "the download does not contain the editor (flux-editor). Is it the right file?".to_string(),
        ));
    }

    // 4. Move into place and clean up.
    if final_folder.exists() {
        fs::remove_dir_all(&final_folder)?;
    }

    fs::rename(&staging, &final_folder)?;
    let _ = fs::remove_file(&zip_path);

    Ok(final_folder.to_string_lossy().to_string())
}
