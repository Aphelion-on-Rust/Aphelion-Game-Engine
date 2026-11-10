//! Game time: frame delta, total time, fixed steps for physics, time scale (slow motion).

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::TimeController;
pub use interface::TimeData;
pub use model::TimeModel;
