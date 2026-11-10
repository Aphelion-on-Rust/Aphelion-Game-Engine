//! Slide-to-change number fields (any int or float in the inspector).
//!
//! Hover      -> the cursor becomes a <-> arrow.
//! Mouse down + move -> the value changes, the pointer is hidden while dragging.
//! Shift = fine (x0.1), Ctrl = coarse (x10). Escape cancels. Min/max (like [Range]) are respected.
//!
//! This is only the logic. The window layer (step 4b) shows the cursor icon and
//! keeps the real pointer from running off the screen while it is hidden.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::NumberFieldController;
pub use interface::{CursorIcon, DragPhase, DragSpeed, NumberFieldConfig, NumberFieldData};
pub use model::NumberFieldModel;
