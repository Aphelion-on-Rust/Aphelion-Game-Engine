use flux_core::error::EngineError;
use std::fmt;

#[derive(Debug)]
pub enum LauncherError {
    Io(std::io::Error),
    Core(EngineError),
    Network(String),
    Json(String),
    Zip(String),
    Checksum { expected: String, actual: String },
    NotFound(String),
    Invalid(String),
}

pub type LauncherResult<T> = Result<T, LauncherError>;

impl fmt::Display for LauncherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LauncherError::Io(error) => write!(f, "file error: {error}"),
            LauncherError::Core(error) => write!(f, "{error}"),
            LauncherError::Network(message) => write!(f, "network error: {message}"),
            LauncherError::Json(message) => write!(f, "could not read the release list: {message}"),
            LauncherError::Zip(message) => write!(f, "zip error: {message}"),
            LauncherError::Checksum { expected, actual } => {
                write!(f, "the download is damaged (expected checksum {expected}, got {actual})")
            }
            LauncherError::NotFound(message) | LauncherError::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LauncherError {}

impl From<std::io::Error> for LauncherError {
    fn from(error: std::io::Error) -> Self {
        LauncherError::Io(error)
    }
}

impl From<EngineError> for LauncherError {
    fn from(error: EngineError) -> Self {
        LauncherError::Core(error)
    }
}
