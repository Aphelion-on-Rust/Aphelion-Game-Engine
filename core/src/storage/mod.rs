//! Saving and loading. Everything is YAML, no JSON.
//! (The launcher reads GitHub's release list, which is JSON only because GitHub sends it that way.)

pub mod store;
pub mod yaml;

pub use store::SettingsStore;
