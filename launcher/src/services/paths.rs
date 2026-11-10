use std::path::{Component, Path, PathBuf};

/// <install_root>/versions
pub fn versions_folder(install_root: &Path) -> PathBuf {
    install_root.join("versions")
}

/// <install_root>/versions/<version>
pub fn version_folder(install_root: &Path, version: &str) -> PathBuf {
    versions_folder(install_root).join(version)
}

/// Where a version is unpacked first. It is renamed to the real folder when complete,
/// so a half-finished install is never mistaken for a working one.
pub fn staging_folder(install_root: &Path, version: &str) -> PathBuf {
    versions_folder(install_root).join(format!("{version}.installing"))
}

/// <install_root>/downloads
pub fn downloads_folder(install_root: &Path) -> PathBuf {
    install_root.join("downloads")
}

/// The editor program inside an installed version.
pub fn editor_exe(version_folder: &Path) -> PathBuf {
    version_folder.join(if cfg!(windows) { "flux-editor.exe" } else { "flux-editor" })
}

/// True if `child` is somewhere under `parent` (not `parent` itself) and has no `..` in it.
/// Used before deleting a folder, so the launcher can never delete outside its versions folder.
pub fn is_inside(parent: &Path, child: &Path) -> bool {
    child != parent
        && child.starts_with(parent)
        && !child.components().any(|part| matches!(part, Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_is_inside_its_parent() {
        let parent = Path::new("/flux/versions");

        assert!(is_inside(parent, Path::new("/flux/versions/0.7.0")));
        assert!(!is_inside(parent, Path::new("/flux/versions")));
        assert!(!is_inside(parent, Path::new("/flux/other/0.7.0")));
        assert!(!is_inside(parent, Path::new("/flux/versions/../secret")));
    }

    #[test]
    fn version_folders_live_under_versions() {
        let root = Path::new("/flux");

        assert_eq!(version_folder(root, "0.7.0"), Path::new("/flux/versions/0.7.0"));
        assert_eq!(staging_folder(root, "0.7.0"), Path::new("/flux/versions/0.7.0.installing"));
    }
}
