use crate::error::{EngineError, EngineResult};
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Every YAML file starts with its format name and version, then the data:
///
/// ```yaml
/// format: id_registry
/// version: 1
/// data:
///   ...
/// ```
#[derive(Serialize)]
struct EnvelopeOut<'a, T> {
    format: &'a str,
    version: u32,
    data: &'a T,
}

#[derive(Deserialize)]
struct EnvelopeIn<T> {
    format: String,
    version: u32,
    data: T,
}

pub fn to_yaml<T: Interface>(data: &T) -> EngineResult<String> {
    let envelope = EnvelopeOut { format: T::FORMAT, version: T::VERSION, data };

    Ok(serde_yaml::to_string(&envelope)?)
}

pub fn from_yaml<T: Interface>(text: &str) -> EngineResult<T> {
    let envelope: EnvelopeIn<T> = serde_yaml::from_str(text)?;

    if envelope.format != T::FORMAT {
        return Err(EngineError::WrongFormat { expected: T::FORMAT, found: envelope.format });
    }

    if envelope.version > T::VERSION {
        return Err(EngineError::NewerVersion {
            format: T::FORMAT,
            file: envelope.version,
            supported: T::VERSION,
        });
    }

    Ok(envelope.data)
}

pub fn save_yaml<T: Interface>(path: &Path, data: &T) -> EngineResult<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }

    // Write next to the file first, then rename over it. If the program stops in the
    // middle of writing, the old file is still there and whole.
    let temp = path.with_extension("tmp");

    fs::write(&temp, to_yaml(data)?)?;
    fs::rename(&temp, path)?;
    Ok(())
}

pub fn load_yaml<T: Interface>(path: &Path) -> EngineResult<T> {
    from_yaml(&fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        name: String,
        count: u32,
    }

    impl Interface for Sample {
        const FORMAT: &'static str = "sample";
        const VERSION: u32 = 2;
    }

    #[test]
    fn data_survives_a_round_trip() {
        let sample = Sample { name: "box".to_string(), count: 3 };
        let loaded: Sample = from_yaml(&to_yaml(&sample).unwrap()).unwrap();

        assert_eq!(loaded, sample);
    }

    #[test]
    fn wrong_format_is_rejected() {
        let text = "format: something_else\nversion: 1\ndata: {name: a, count: 1}\n";

        assert!(from_yaml::<Sample>(text).is_err());
    }

    #[test]
    fn a_file_from_a_newer_version_is_rejected() {
        let text = "format: sample\nversion: 3\ndata: {name: a, count: 1}\n";

        assert!(matches!(from_yaml::<Sample>(text), Err(EngineError::NewerVersion { .. })));
    }

    #[test]
    fn saving_leaves_no_temp_file_and_loads_back() {
        let folder = std::env::temp_dir().join(format!("flux_core_yaml_{}", std::process::id()));
        let path = folder.join("sample.yaml");
        let sample = Sample { name: "disk".to_string(), count: 1 };

        save_yaml(&path, &sample).unwrap();
        save_yaml(&path, &sample).unwrap(); // saving over an existing file works too

        assert!(!folder.join("sample.tmp").exists());
        assert_eq!(load_yaml::<Sample>(&path).unwrap(), sample);
    }
}
