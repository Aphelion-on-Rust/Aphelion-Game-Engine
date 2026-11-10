use flux_engine::input::KeyCode;
use flux_engine::mivc::Interface;
use serde::{Deserialize, Serialize};

/// A key plus the modifiers that must be held with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chord {
    pub key: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Chord {
    pub const fn key(key: KeyCode) -> Self {
        Self { key, ctrl: false, shift: false, alt: false }
    }

    pub const fn ctrl(key: KeyCode) -> Self {
        Self { key, ctrl: true, shift: false, alt: false }
    }

    pub const fn ctrl_shift(key: KeyCode) -> Self {
        Self { key, ctrl: true, shift: true, alt: false }
    }

    /// For menus: "Ctrl+Shift+N".
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl".to_string());
        }

        if self.shift {
            parts.push("Shift".to_string());
        }

        if self.alt {
            parts.push("Alt".to_string());
        }

        parts.push(format!("{:?}", self.key));
        parts.join("+")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub action: String,
    pub chord: Chord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindData {
    pub bindings: Vec<Binding>,
}

impl Default for KeybindData {
    fn default() -> Self {
        Self { bindings: default_bindings() }
    }
}

impl Interface for KeybindData {
    const FORMAT: &'static str = "editor_keybinds";
    const VERSION: u32 = 1;
}

/// Ids of the built-in actions.
pub mod actions {
    pub const UNDO: &str = "editor.undo";
    pub const REDO: &str = "editor.redo";
    pub const SAVE: &str = "editor.save";
    pub const DUPLICATE: &str = "editor.duplicate";
    pub const DELETE: &str = "editor.delete";
    pub const RENAME: &str = "editor.rename";
    pub const PLAY: &str = "scene.play";
    pub const FOCUS_SELECTED: &str = "scene.focus_selected";
    pub const NEW_OBJECT: &str = "scene.new_object";
}

pub fn default_bindings() -> Vec<Binding> {
    let bind = |action: &str, chord: Chord| Binding { action: action.to_string(), chord };

    vec![
        bind(actions::UNDO, Chord::ctrl(KeyCode::Z)),
        bind(actions::REDO, Chord::ctrl(KeyCode::Y)),
        bind(actions::SAVE, Chord::ctrl(KeyCode::S)),
        bind(actions::DUPLICATE, Chord::ctrl(KeyCode::D)),
        bind(actions::DELETE, Chord::key(KeyCode::Delete)),
        bind(actions::RENAME, Chord::key(KeyCode::F2)),
        bind(actions::PLAY, Chord::ctrl(KeyCode::P)),
        bind(actions::FOCUS_SELECTED, Chord::key(KeyCode::F)),
        bind(actions::NEW_OBJECT, Chord::ctrl_shift(KeyCode::N)),
    ]
}
