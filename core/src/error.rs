use std::fmt;

/// Every error the engine can return.
#[derive(Debug)]
pub enum EngineError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    /// The file is a different kind of data than we asked for.
    WrongFormat { expected: &'static str, found: String },
    /// The file was saved by a newer engine than this one.
    NewerVersion { format: &'static str, file: u32, supported: u32 },
    DuplicateUuid(String),
    DuplicateComponentId(u32),
    ObjectNotFound(String),
    ComponentNotFound(u32),
    /// The request makes no sense (parenting an object into its own child, two transforms, ...).
    InvalidOperation(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Io(error) => write!(f, "file error: {error}"),
            EngineError::Yaml(error) => write!(f, "yaml error: {error}"),
            EngineError::WrongFormat { expected, found } => {
                write!(f, "expected a \"{expected}\" file but found \"{found}\"")
            }
            EngineError::NewerVersion { format, file, supported } => write!(
                f,
                "\"{format}\" file is version {file}, this engine only supports up to {supported}"
            ),
            EngineError::DuplicateUuid(uuid) => write!(f, "uuid {uuid} is already used"),
            EngineError::DuplicateComponentId(id) => write!(f, "component id {id} is already used"),
            EngineError::ObjectNotFound(uuid) => write!(f, "no object with uuid {uuid}"),
            EngineError::ComponentNotFound(id) => write!(f, "no component with id {id}"),
            EngineError::InvalidOperation(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<std::io::Error> for EngineError {
    fn from(error: std::io::Error) -> Self {
        EngineError::Io(error)
    }
}

impl From<serde_yaml::Error> for EngineError {
    fn from(error: serde_yaml::Error) -> Self {
        EngineError::Yaml(error)
    }
}
