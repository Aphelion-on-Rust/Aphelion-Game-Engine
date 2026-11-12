//! Aphelion's Control: version history for prefab saves.
//!
//! Every time a prefab is saved, the difference to the previous save is stored as a revision.
//! Reverting a revision undoes ONLY what that revision changed. Edits made in later revisions
//! stay, and if a later edit touched the same thing, that part is skipped and reported.
//!
//! - interface.rs  the data (revisions, changes) and its YAML shape
//! - diff.rs       finding the changes between two prefab states, and undoing them
//! - controller.rs recording a save, reverting a revision
//! - query.rs      lookups, summaries, ids used in the history
//! - view.rs       the rows of the history list
//!
//! Disk access (one YAML file per prefab in `.aphelion/prefabs`) is done by the editor.

pub mod controller;
pub mod diff;
pub mod interface;
pub mod model;
pub mod query;
pub mod view;

pub use controller::{ControlController, SaveInfo};
pub use diff::RevertReport;
pub use interface::{ControlData, PrefabChange, PrefabHistoryData, Revision};
pub use model::ControlModel;
pub use view::{ControlView, RevisionRow};
