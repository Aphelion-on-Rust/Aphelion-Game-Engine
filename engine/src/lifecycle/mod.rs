//! Script lifecycle: Awake, Start, Update, FixedUpdate, OnIK, OnMouseDown, ...
//!
//! - interface.rs        the list of events and the runtime state
//! - model.rs            stores that state
//! - controller.rs       decides which script gets which event, and when
//! - mouse_controller.rs turns hover + mouse input into OnMouse... events
//! - runner.rs           plugs it into the game loop and calls the script host
//!
//! The controllers only DECIDE (they return a list of `Dispatch`).
//! The ScriptHost (the C# host, step 9) is what actually calls the script method.
//! See docs/LIFECYCLE.md for when every event fires.

pub mod controller;
pub mod interface;
pub mod model;
pub mod mouse_controller;
pub mod runner;

pub use controller::LifecycleController;
pub use interface::{
    Dispatch, InstanceState, LifecycleData, LifecycleEvent, MouseTrackerData, PendingEvent,
};
pub use model::{LifecycleModel, MouseTrackerModel};
pub use mouse_controller::MouseEventController;
pub use runner::{ScriptHost, ScriptRunner};
