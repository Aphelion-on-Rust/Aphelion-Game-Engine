//! Engine versions that are installed on this computer, and which one is the default.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;
pub mod view;

pub use controller::InstallsController;
pub use interface::{InstalledVersion, InstallsData};
pub use model::InstallsModel;
pub use view::{InstallRow, InstallsView, RowState};
