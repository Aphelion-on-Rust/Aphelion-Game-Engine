//! The launcher window (egui).
//!
//! Same rule as the editor: views read the models and push `LauncherCommand`s.
//! `commands::apply_all` runs them through the controllers and returns `Effect`s
//! (download, delete, start a program). The app carries the effects out with `services`.
//!
//! One frame: background events -> draw -> apply commands -> run effects -> save.

pub mod app;
pub mod commands;
pub mod models;
pub mod tabs;

pub use app::LauncherApp;
pub use commands::{ControllerState, Effect, LauncherCommand};
pub use models::LauncherModels;
