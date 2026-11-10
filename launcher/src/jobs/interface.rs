use flux_core::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobPhase {
    Downloading,
    Verifying,
    Extracting,
    Finished,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstallJob {
    pub version: String,
    pub phase: JobPhase,
    pub downloaded_bytes: u64,
    /// 0 if the server did not say how big the file is.
    pub total_bytes: u64,
    /// The error text when the phase is Failed.
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JobsData {
    pub jobs: Vec<InstallJob>,
}

impl Interface for JobsData {
    const FORMAT: &'static str = "install_jobs";
    const VERSION: u32 = 1;
}

/// What the background install thread reports.
#[derive(Clone, Debug, PartialEq)]
pub enum InstallEvent {
    Progress { version: String, downloaded: u64, total: u64 },
    Phase { version: String, phase: JobPhase },
    Failed { version: String, message: String },
    Finished { version: String, folder: String },
}

impl InstallEvent {
    pub fn version(&self) -> &str {
        match self {
            InstallEvent::Progress { version, .. }
            | InstallEvent::Phase { version, .. }
            | InstallEvent::Failed { version, .. }
            | InstallEvent::Finished { version, .. } => version,
        }
    }
}
