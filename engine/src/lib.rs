//! Flux Aphelion engine.
//!
//! Everything here follows MIVC:
//! - Interface: the shape of the data (what gets saved to YAML)
//! - Model:     stores the data
//! - View:      reads and shows the data, can never change it
//! - Controller: all the logic, the only one allowed to change a model

// The MIVC base, errors and YAML storage live in flux-core. They are re-exported here so
// `flux_engine::mivc::...` and `crate::mivc::...` keep working.
pub use flux_core::{error, mivc, storage};

pub mod components;
pub mod control;
pub mod game_loop;
pub mod gc;
pub mod identity;
pub mod input;
pub mod inspect;
pub mod lifecycle;
pub mod math;
pub mod prefab;
pub mod scene;
pub mod time;
