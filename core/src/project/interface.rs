use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

/// The graphics pipeline of a project. Chosen when the project is created, never changed after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsPipeline {
    /// SGP: Simple Graphics Pipeline.
    #[default]
    #[serde(rename = "sgp")]
    Simple,
    /// HGP: Heavy Graphics Pipeline.
    #[serde(rename = "hgp")]
    Heavy,
}

impl GraphicsPipeline {
    pub fn short_name(self) -> &'static str {
        match self {
            GraphicsPipeline::Simple => "SGP",
            GraphicsPipeline::Heavy => "HGP",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            GraphicsPipeline::Simple => "Simple Graphics Pipeline",
            GraphicsPipeline::Heavy => "Heavy Graphics Pipeline",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            GraphicsPipeline::Simple => {
                "Fast and light. Good for 2D and simple 3D on any computer. No ray tracing, DLSS or frame boost."
            }
            GraphicsPipeline::Heavy => {
                "Shader graph, baked lighting, probes, ray tracing, DLSS and frame boost. For high-end 3D."
            }
        }
    }
}

/// The contents of `project.yaml`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
    /// The engine version this project was last opened with.
    pub engine_version: String,
    /// Fixed at creation.
    pub pipeline: GraphicsPipeline,
    /// Aphelion's Control (version history for prefab saves). Fixed at creation.
    pub aphelion_control: bool,
    pub created_on: String,
}

impl Interface for ProjectData {
    const FORMAT: &'static str = "project";
    const VERSION: u32 = 1;
}
