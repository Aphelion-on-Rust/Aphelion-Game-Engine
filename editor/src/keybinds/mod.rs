//! Keyboard shortcuts. An action is a text id like "editor.save".
//! Using text ids lets extensions add their own actions later.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::KeybindController;
pub use interface::{actions, default_bindings, Binding, Chord, KeybindData};
pub use model::KeybindModel;
