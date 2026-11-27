//! Play mode: press Play and the scene comes alive with physics, press Stop and everything is put back.
//!
//! - interface.rs   the state (stopped, playing, paused), the saved copy of the scene, the clock
//! - model.rs       `PlayModel`
//! - query.rs       questions the views ask ("are we playing?", the text for the toolbar)
//! - controller.rs  start, pause, stop, and moving time forward
//!
//! The physics world itself (`PhysicsWorld`) is not saved data, so the app holds it while playing.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::PlayController;
pub use interface::{PlayData, PlayRequest, PlayState};
pub use model::PlayModel;
