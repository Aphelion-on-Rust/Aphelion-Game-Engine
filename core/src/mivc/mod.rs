//! The four MIVC building blocks. Read these files in this order:
//! interface -> model -> controller -> view.

pub mod controller;
pub mod interface;
pub mod model;
pub mod view;

pub use controller::{Controller, WriteToken};
pub use interface::Interface;
pub use model::{Model, Store};
pub use view::{ReadOnly, View};
