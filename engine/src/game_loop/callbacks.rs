use crate::gc::CollectedResource;
use crate::input::InputModel;
use crate::mivc::Model;
use crate::time::{query::blend_alpha, TimeModel};

/// Read-only view of the frame for game code. It can look but not change anything.
pub struct FrameContext<'a> {
    pub time: &'a TimeModel,
    pub input: &'a InputModel,
}

impl FrameContext<'_> {
    pub fn delta(&self) -> f64 {
        self.time.data().delta_seconds
    }

    pub fn fixed_delta(&self) -> f64 {
        self.time.data().fixed_delta_seconds
    }

    pub fn blend_alpha(&self) -> f64 {
        blend_alpha(self.time.data())
    }
}

/// What the game (or the editor) plugs into the loop.
pub trait LoopCallbacks {
    /// Runs at a steady rate. Physics goes here.
    fn fixed_update(&mut self, context: &FrameContext);

    /// Runs once per frame. Game logic goes here.
    fn update(&mut self, context: &FrameContext);

    /// Runs once per frame after update.
    fn render(&mut self, context: &FrameContext);

    /// Called when the GC freed resources, so the real memory can be released.
    fn on_resources_collected(&mut self, _collected: &[CollectedResource]) {}
}

/// A small summary of what happened in the frame.
#[derive(Debug, PartialEq, Eq)]
pub struct FrameReport {
    pub fixed_steps: u32,
    pub collected_resources: u32,
}
