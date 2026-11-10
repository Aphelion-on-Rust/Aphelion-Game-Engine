use flux_core::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    /// The project folder.
    pub path: String,
    /// The engine version this project opens with.
    pub engine_version: String,
    /// Date only. Empty if never opened.
    pub last_opened: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectsData {
    pub projects: Vec<ProjectEntry>,
}

impl Interface for ProjectsData {
    const FORMAT: &'static str = "launcher_projects";
    const VERSION: u32 = 1;
}
