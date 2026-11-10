//! The hierarchy panel: the object tree.
//!
//! - HierarchyUiData (model): which objects are expanded and selected
//! - HierarchyView: reads the scene + that state and builds the rows to draw (read-only)
//! - HierarchyController: select, expand, create, delete, drag to reparent

pub mod controller;
pub mod interface;
pub mod model;
pub mod view;

pub use controller::HierarchyController;
pub use interface::{HierarchyRow, HierarchyUiData};
pub use model::HierarchyUiModel;
pub use view::HierarchyView;
