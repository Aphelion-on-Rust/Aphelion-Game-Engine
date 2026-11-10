//! The list of projects the launcher knows, and which engine version each one uses.
//! Creating a new project (templates, SGP/HGP choice) comes with the project step.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;
pub mod view;

pub use controller::ProjectsController;
pub use interface::{ProjectEntry, ProjectsData};
pub use model::ProjectsModel;
pub use view::{ProjectRow, ProjectsView};
