use super::interface::{PlayData, PlayState};

/// Playing or paused: the scene is a temporary copy that Stop will replace.
pub fn in_play_mode(data: &PlayData) -> bool {
    data.state != PlayState::Stopped
}

pub fn is_running(data: &PlayData) -> bool {
    data.state == PlayState::Playing
}

/// The text for the toolbar.
pub fn label(data: &PlayData) -> String {
    match data.state {
        PlayState::Stopped => "Stopped".to_string(),
        PlayState::Playing => format!("Playing {:.1} s", data.elapsed),
        PlayState::Paused => format!("Paused at {:.1} s", data.elapsed),
    }
}
