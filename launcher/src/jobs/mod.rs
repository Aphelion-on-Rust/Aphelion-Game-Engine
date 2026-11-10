//! Progress of installs that are running in the background.
//! Nothing here is saved, it only lives while the launcher is open.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::JobsController;
pub use interface::{InstallEvent, InstallJob, JobPhase, JobsData};
pub use model::JobsModel;
