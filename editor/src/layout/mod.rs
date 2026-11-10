//! Dockable panel layout.
//!
//! The layout is a tree. A `Tabs` node is a group of panels shown as tabs.
//! A `Split` node puts two nodes side by side (or one above the other).
//! Dragging a panel onto another group's edge makes a new split, dropping it
//! in the middle adds it as a tab.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;
mod tree;

pub use controller::LayoutController;
pub use interface::{Axis, DockNode, DropZone, LayoutData, PanelKind};
pub use model::LayoutModel;
