use super::yaml::{load_yaml, save_yaml};
use crate::error::EngineResult;
use crate::mivc::Interface;
use std::path::{Path, PathBuf};

/// A folder where an app keeps its own settings files (YAML).
/// Windows: %APPDATA%\FluxAphelion\<app>. Elsewhere: $HOME/FluxAphelion/<app>.
pub struct SettingsStore {
    folder: PathBuf,
}

impl SettingsStore {
    /// The settings folder for an app, for example `for_app("editor")` or `for_app("launcher")`.
    pub fn for_app(app: &str) -> Self {
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Self::with_folder(base.join("FluxAphelion").join(app))
    }

    pub fn with_folder(folder: PathBuf) -> Self {
        Self { folder }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Loads a settings file. A missing or broken file gives the default instead.
    pub fn load_or_default<T: Interface + Default>(&self, file: &str) -> T {
        load_yaml(&self.folder.join(file)).unwrap_or_default()
    }

    pub fn save<T: Interface>(&self, file: &str, data: &T) -> EngineResult<()> {
        save_yaml(&self.folder.join(file), data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Sample {
        name: String,
    }

    impl Interface for Sample {
        const FORMAT: &'static str = "sample";
        const VERSION: u32 = 1;
    }

    fn temp_folder(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("flux_core_{name}_{}", std::process::id()))
    }

    #[test]
    fn a_missing_file_gives_the_default() {
        let store = SettingsStore::with_folder(temp_folder("missing"));

        assert_eq!(store.load_or_default::<Sample>("sample.yaml"), Sample::default());
    }

    #[test]
    fn saved_settings_load_back() {
        let store = SettingsStore::with_folder(temp_folder("saved"));
        let sample = Sample { name: "hello".to_string() };

        store.save("sample.yaml", &sample).unwrap();

        assert_eq!(store.load_or_default::<Sample>("sample.yaml"), sample);
    }
}
