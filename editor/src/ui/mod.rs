//! The editor window, drawn with egui.
//!
//! How this follows MIVC:
//! - The models live in `EditorModels`.
//! - Everything that DRAWS (dock_view, panels, menu_bar, number_drag) is a view:
//!   it gets `&EditorModels` and can't change anything. When the user does something
//!   (clicks, drags, types) the view pushes a `UiCommand` into a list.
//! - After drawing, `commands::apply_all` hands every command to the right controller.
//!
//! One frame: read input -> draw (collect commands) -> apply commands -> set cursor -> save settings.

pub mod app;
pub mod commands;
pub mod dock_view;
pub mod field;
pub mod input_bridge;
pub mod menu_bar;
pub mod models;
pub mod number_drag;
pub mod panels;
pub mod theme_apply;

pub use app::EditorApp;
pub use commands::{ControllerState, UiCommand};
pub use models::EditorModels;
