//! The world matrices live in the engine (`flux_engine::scene::transform`) because physics needs them too.
//! They are re-exported here so the renderer keeps using the same names.

pub use flux_engine::scene::transform::{local_matrix, world_matrix};
