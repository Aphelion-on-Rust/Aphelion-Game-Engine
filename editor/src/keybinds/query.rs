use super::interface::{Chord, KeybindData};
use flux_engine::input::{InputData, KeyCode};

pub fn chord_for<'a>(data: &'a KeybindData, action: &str) -> Option<&'a Chord> {
    data.bindings.iter().find(|binding| binding.action == action).map(|binding| &binding.chord)
}

pub fn action_for<'a>(data: &'a KeybindData, chord: &Chord) -> Option<&'a str> {
    data.bindings.iter().find(|binding| binding.chord == *chord).map(|binding| binding.action.as_str())
}

/// The actions whose shortcut was just pressed this frame.
/// The key must be pressed this frame, and the modifiers must match exactly
/// (Ctrl+S does not trigger a plain S shortcut, and S does not trigger Ctrl+S).
pub fn triggered_actions<'a>(data: &'a KeybindData, input: &InputData) -> Vec<&'a str> {
    let ctrl = input.keys_down.contains(&KeyCode::Ctrl);
    let shift = input.keys_down.contains(&KeyCode::Shift);
    let alt = input.keys_down.contains(&KeyCode::Alt);

    data.bindings
        .iter()
        .filter(|binding| {
            let chord = &binding.chord;

            input.keys_pressed.contains(&chord.key)
                && chord.ctrl == ctrl
                && chord.shift == shift
                && chord.alt == alt
        })
        .map(|binding| binding.action.as_str())
        .collect()
}
