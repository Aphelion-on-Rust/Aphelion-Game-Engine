//! Flux Aphelion launcher: a separate app from the editor.
//! It finds engine releases on GitHub, downloads and installs them, and starts the editor
//! of the version you pick.
//!
//! MIVC folders (pure logic, no network or disk access, fully testable):
//!   settings, releases, installs, projects, jobs
//! Side effects (network, zip, disk, starting programs) live in `services` only.
//! `ui` draws the window and glues the two together.

pub mod error;
pub mod installs;
pub mod jobs;
pub mod projects;
pub mod releases;
pub mod services;
pub mod settings;
pub mod ui;
pub mod version;
