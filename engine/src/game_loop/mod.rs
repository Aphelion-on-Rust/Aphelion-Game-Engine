//! The frame loop that ties time, input and the GC together.
//!
//! One frame:
//!   1. time.begin_frame      2. gc.begin_frame
//!   3. fixed_update (0 or more times, for physics)
//!   4. update               5. render
//!   6. gc.collect (tell the game what was freed)
//!   7. input.end_frame
//!
//! The loop has no model of its own, the state it needs lives in TimeModel.
//! The real window comes in a later step, it will call `tick` once per frame.

pub mod callbacks;
pub mod controller;

pub use callbacks::{FrameContext, FrameReport, LoopCallbacks};
pub use controller::GameLoop;
