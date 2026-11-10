//! Raw input state: which keys and mouse buttons are down, pressed or released this frame.
//! The window layer (later step) feeds events in through the InputController.
//! The custom Input System with actions and bindings is built on top of this (step 16).

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::InputController;
pub use interface::{InputData, KeyCode, MouseButton};
pub use model::InputModel;
