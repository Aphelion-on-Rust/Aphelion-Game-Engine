//! Undo / redo for scene edits.
//!
//! It keeps snapshots of the scene: before every edit the scene is copied, and undo puts that copy
//! back. Simple and always correct. (A scene with tens of thousands of objects would need a
//! lighter way later, the rest of the editor would not have to change.)
//!
//! - one edit = one step. A whole number drag is ONE step (begin_group / commit_group).
//! - typing a name is ONE step (steps with the same merge key are merged).
//! - a new edit after an undo throws the redo steps away.
//! - the history lives only while the editor is open (it is not saved).

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;
pub mod view;

pub use controller::HistoryController;
pub use interface::{HistoryData, HistoryEntry, OpenGroup};
pub use model::HistoryModel;
pub use view::{HistoryRow, HistoryRowKind, HistoryView};
