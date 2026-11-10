//! Flux Aphelion shared base. Used by the engine, the editor and the launcher.
//!
//! - mivc     the MIVC building blocks (Interface, Model, View, Controller)
//! - error    the error type
//! - storage  YAML files and the settings folder
//! - project  the project file and folder layout (the launcher creates projects, the editor opens them)
//!
//! It has no engine or UI code, so small apps (like the launcher) can use it
//! without pulling in the whole engine.

pub mod error;
pub mod mivc;
pub mod project;
pub mod storage;
