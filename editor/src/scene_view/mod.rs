//! The camera of the Scene view. You orbit it around a point with the right mouse button, pan with the
//! middle button, zoom with the wheel and focus on the selected object with F.
//! The maths is `flux_grp::frame::OrbitCamera`. Here it is a model, so the camera has one owner.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::SceneCameraController;
pub use interface::SceneCameraData;
pub use model::SceneCameraModel;
