use super::paths;
use crate::error::{LauncherError, LauncherResult};
use std::path::Path;
use std::process::Command;

/// Starts the editor of an installed version. A project is passed as `--project <folder>`.
pub fn launch_editor(version_folder: &Path, project: Option<&Path>) -> LauncherResult<()> {
    let exe = paths::editor_exe(version_folder);

    if !exe.exists() {
        return Err(LauncherError::NotFound(format!("the editor is missing: {}", exe.display())));
    }

    let mut command = Command::new(&exe);

    command.current_dir(version_folder);

    if let Some(project) = project {
        command.arg("--project").arg(project);
    }

    command.spawn()?;
    Ok(())
}
