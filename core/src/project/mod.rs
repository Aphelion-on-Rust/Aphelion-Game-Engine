//! A project is a folder with a `project.yaml` and a fixed layout (see docs/PROJECT_FORMAT.md).
//! The launcher creates projects, the editor opens them, so this lives in core.

pub mod controller;
pub mod disk;
pub mod interface;
pub mod layout;
pub mod model;

pub use controller::ProjectController;
pub use interface::{GraphicsPipeline, ProjectData};
pub use model::ProjectModel;
