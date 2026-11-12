//! Flux Aphelion editor: logic (step 4a) and the window + drawing (step 4b, folder `ui`).
//!
//! - layout       dockable panels: split, drag to dock, tabs, resize
//! - theme        colors, built-in Dark and Light, custom themes
//! - keybinds     actions and shortcuts, rebinding with conflict checks
//! - number_field slide-to-change numbers (<-> cursor, hidden pointer while dragging)
//! - hierarchy    the object tree: rows, selection, drag to reparent
//! - history      undo / redo of scene edits
//! - project_io   opening a project folder, loading and saving its scene and id registry
//!
//! Every folder uses the same MIVC files: interface, model, controller, (view, query).

pub mod hierarchy;
pub mod history;
pub mod keybinds;
pub mod layout;
pub mod number_field;
pub mod prefab_library;
pub mod project_io;
pub mod theme;
pub mod ui;
