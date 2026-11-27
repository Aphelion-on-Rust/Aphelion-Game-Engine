use flux_engine::mivc::Interface;
use flux_engine::scene::SceneData;
use serde::{Deserialize, Serialize};

/// Physics moves forward in steps of this many seconds, whatever the speed of the screen.
pub const FIXED_STEP: f32 = 1.0 / 60.0;

/// If the screen is slow, at most this many steps are made per screen frame (the game slows down instead of freezing).
pub const MAX_STEPS_PER_FRAME: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayData {
    pub state: PlayState,
    /// The scene as it was when Play was pressed. Stop puts it back.
    pub snapshot: Option<SceneData>,
    /// Seconds played.
    pub elapsed: f32,
    /// Time that was not used yet because it is less than one step.
    pub accumulator: f32,
    /// The last thing that happened between two objects, for the toolbar.
    pub last_event: String,
}

impl Default for PlayData {
    fn default() -> Self {
        Self { state: PlayState::Stopped, snapshot: None, elapsed: 0.0, accumulator: 0.0, last_event: String::new() }
    }
}

impl Interface for PlayData {
    const FORMAT: &'static str = "play_state";
    const VERSION: u32 = 1;
}

/// What the app should do with the play mode. The command layer only asks, because starting needs the physics
/// world, which the app holds (see `ControllerState::pending_play`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayRequest {
    Start,
    Stop,
    TogglePause,
}
