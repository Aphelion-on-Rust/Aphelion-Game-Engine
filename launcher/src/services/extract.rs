use crate::error::{LauncherError, LauncherResult};
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

fn zip_error(error: zip::result::ZipError) -> LauncherError {
    LauncherError::Zip(error.to_string())
}

/// `base` + `entry`, but only if `entry` stays inside `base`.
/// A zip entry like "../../evil.exe" returns None (the "zip slip" attack).
pub fn safe_join(base: &Path, entry: &str) -> Option<PathBuf> {
    let mut result = base.to_path_buf();

    for part in Path::new(entry).components() {
        match part {
            Component::Normal(name) => result.push(name),
            Component::CurDir => {}
            _ => return None,
        }
    }

    Some(result)
}

/// If everything in the zip sits in one top-level folder ("flux-aphelion-0.7.0/..."),
/// returns that folder's name so it can be skipped. A zip with files at the top returns None.
pub fn common_root(names: &[String]) -> Option<String> {
    let first = names.first()?;

    if !first.contains('/') {
        return None;
    }

    let root = first.split('/').next()?;

    if root.is_empty() {
        return None;
    }

    let prefix = format!("{root}/");

    names.iter().all(|name| name.starts_with(&prefix)).then(|| root.to_string())
}

/// Unpacks a zip into `destination`, dropping a single top-level folder if there is one.
/// `on_progress(done, total)` is called after every entry.
pub fn extract_zip(
    zip_path: &Path,
    destination: &Path,
    on_progress: &mut dyn FnMut(usize, usize),
) -> LauncherResult<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path)?).map_err(zip_error)?;
    let total = archive.len();
    let mut names = Vec::with_capacity(total);

    for index in 0..total {
        names.push(archive.by_index(index).map_err(zip_error)?.name().to_string());
    }

    let prefix = common_root(&names).map(|root| format!("{root}/"));

    fs::create_dir_all(destination)?;

    for index in 0..total {
        let mut entry = archive.by_index(index).map_err(zip_error)?;
        let name = entry.name().to_string();
        let relative = match &prefix {
            Some(prefix) => name.strip_prefix(prefix.as_str()).unwrap_or(&name),
            None => name.as_str(),
        };

        if !relative.is_empty() {
            let target = safe_join(destination, relative)
                .ok_or_else(|| LauncherError::Zip(format!("the zip contains an unsafe path: {name}")))?;

            if entry.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }

                std::io::copy(&mut entry, &mut File::create(&target)?)?;
            }
        }

        on_progress(index + 1, total);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn paths_that_escape_the_folder_are_refused() {
        let base = Path::new("/install/staging");

        assert_eq!(safe_join(base, "bin/editor.exe"), Some(PathBuf::from("/install/staging/bin/editor.exe")));
        assert!(safe_join(base, "../evil.exe").is_none());
        assert!(safe_join(base, "a/../../evil.exe").is_none());
        assert!(safe_join(base, "/etc/passwd").is_none());
    }

    #[test]
    fn a_single_top_folder_is_found() {
        assert_eq!(common_root(&names(&["flux/", "flux/flux-editor.exe", "flux/data/a.yaml"])), Some("flux".to_string()));
    }

    #[test]
    fn a_flat_or_mixed_zip_has_no_common_folder() {
        assert_eq!(common_root(&names(&["flux-editor.exe", "data/a.yaml"])), None);
        assert_eq!(common_root(&names(&["flux/a.exe", "other/b.exe"])), None);
        assert_eq!(common_root(&names(&["flux/a.exe", "readme.txt"])), None);
        assert_eq!(common_root(&[]), None);
    }
}
