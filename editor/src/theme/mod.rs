//! Editor themes. Dark and Light are built in and can't be edited,
//! copy one to make your own.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::ThemeController;
pub use interface::{Color, ThemeColorSlot, ThemeColors, ThemeData, ThemeSettingsData};
pub use model::ThemeModel;
