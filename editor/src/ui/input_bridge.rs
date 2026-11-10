//! Copies keyboard events from egui into the engine's input state,
//! so editor shortcuts use the same KeyCode and InputData as the game.

use eframe::egui;
use flux_engine::input::{InputController, InputModel, KeyCode};

fn map_key(key: egui::Key) -> Option<KeyCode> {
    use egui::Key as K;

    Some(match key {
        K::A => KeyCode::A, K::B => KeyCode::B, K::C => KeyCode::C, K::D => KeyCode::D,
        K::E => KeyCode::E, K::F => KeyCode::F, K::G => KeyCode::G, K::H => KeyCode::H,
        K::I => KeyCode::I, K::J => KeyCode::J, K::K => KeyCode::K, K::L => KeyCode::L,
        K::M => KeyCode::M, K::N => KeyCode::N, K::O => KeyCode::O, K::P => KeyCode::P,
        K::Q => KeyCode::Q, K::R => KeyCode::R, K::S => KeyCode::S, K::T => KeyCode::T,
        K::U => KeyCode::U, K::V => KeyCode::V, K::W => KeyCode::W, K::X => KeyCode::X,
        K::Y => KeyCode::Y, K::Z => KeyCode::Z,
        K::Num0 => KeyCode::Num0, K::Num1 => KeyCode::Num1, K::Num2 => KeyCode::Num2,
        K::Num3 => KeyCode::Num3, K::Num4 => KeyCode::Num4, K::Num5 => KeyCode::Num5,
        K::Num6 => KeyCode::Num6, K::Num7 => KeyCode::Num7, K::Num8 => KeyCode::Num8,
        K::Num9 => KeyCode::Num9,
        K::F1 => KeyCode::F1, K::F2 => KeyCode::F2, K::F3 => KeyCode::F3, K::F4 => KeyCode::F4,
        K::F5 => KeyCode::F5, K::F6 => KeyCode::F6, K::F7 => KeyCode::F7, K::F8 => KeyCode::F8,
        K::F9 => KeyCode::F9, K::F10 => KeyCode::F10, K::F11 => KeyCode::F11, K::F12 => KeyCode::F12,
        K::ArrowUp => KeyCode::Up,
        K::ArrowDown => KeyCode::Down,
        K::ArrowLeft => KeyCode::Left,
        K::ArrowRight => KeyCode::Right,
        K::Space => KeyCode::Space,
        K::Enter => KeyCode::Enter,
        K::Escape => KeyCode::Escape,
        K::Tab => KeyCode::Tab,
        K::Backspace => KeyCode::Backspace,
        K::Delete => KeyCode::Delete,
        _ => return None,
    })
}

/// Call once at the start of every frame.
pub fn feed(ctx: &egui::Context, input: &mut InputModel) {
    let (events, modifiers) = ctx.input(|state| (state.events.clone(), state.modifiers));

    // Modifiers first, so Ctrl+S in one frame counts as Ctrl held when S is pressed.
    InputController.key_event(input, KeyCode::Ctrl, modifiers.ctrl || modifiers.command);
    InputController.key_event(input, KeyCode::Shift, modifiers.shift);
    InputController.key_event(input, KeyCode::Alt, modifiers.alt);

    for event in events {
        if let egui::Event::Key { key, pressed, .. } = event {
            if let Some(code) = map_key(key) {
                InputController.key_event(input, code, pressed);
            }
        }
    }
}
