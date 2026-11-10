use super::interface::{default_bindings, Binding, Chord};
use super::model::KeybindModel;
use super::query;
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::mivc::{Controller, Model};

pub struct KeybindController;

impl Controller for KeybindController {}

impl KeybindController {
    /// Gives an action a new shortcut. Refused if another action already uses it.
    pub fn rebind(&self, model: &mut KeybindModel, action: &str, chord: Chord) -> EngineResult<()> {
        if let Some(other) = query::action_for(model.data(), &chord) {
            if other != action {
                return Err(EngineError::InvalidOperation(format!(
                    "{} is already used by {other}",
                    chord.describe()
                )));
            }
        }

        let data = self.write(model);

        match data.bindings.iter_mut().find(|binding| binding.action == action) {
            Some(binding) => binding.chord = chord,
            None => data.bindings.push(Binding { action: action.to_string(), chord }),
        }

        Ok(())
    }

    /// Removes the shortcut of an action. The action itself still works from menus.
    pub fn unbind(&self, model: &mut KeybindModel, action: &str) {
        self.write(model).bindings.retain(|binding| binding.action != action);
    }

    /// Puts one action back to its default shortcut.
    /// Fails if the action has no default, or the default is taken by another action now.
    pub fn reset_action(&self, model: &mut KeybindModel, action: &str) -> EngineResult<()> {
        let default = default_bindings()
            .into_iter()
            .find(|binding| binding.action == action)
            .ok_or_else(|| EngineError::InvalidOperation(format!("\"{action}\" has no default shortcut")))?;

        self.rebind(model, action, default.chord)
    }

    pub fn reset_all(&self, model: &mut KeybindModel) {
        self.write(model).bindings = default_bindings();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keybinds::interface::{actions, KeybindData};
    use crate::keybinds::query::{chord_for, triggered_actions};
    use flux_engine::input::{InputController, InputModel, KeyCode};
    use flux_engine::storage::yaml::{from_yaml, to_yaml};

    #[test]
    fn rebinding_to_a_used_shortcut_is_refused() {
        let mut model = KeybindModel::default();

        let result = KeybindController.rebind(&mut model, actions::PLAY, Chord::ctrl(KeyCode::S));

        assert!(result.is_err());
        assert_eq!(chord_for(model.data(), actions::PLAY), Some(&Chord::ctrl(KeyCode::P)));
    }

    #[test]
    fn rebinding_to_a_free_shortcut_works() {
        let mut model = KeybindModel::default();

        KeybindController.rebind(&mut model, actions::PLAY, Chord::key(KeyCode::F5)).unwrap();

        assert_eq!(chord_for(model.data(), actions::PLAY), Some(&Chord::key(KeyCode::F5)));
    }

    #[test]
    fn reset_brings_the_default_back() {
        let mut model = KeybindModel::default();

        KeybindController.rebind(&mut model, actions::PLAY, Chord::key(KeyCode::F5)).unwrap();
        KeybindController.reset_action(&mut model, actions::PLAY).unwrap();

        assert_eq!(model.data(), &KeybindData::default());
        assert!(KeybindController.reset_action(&mut model, "extension.thing").is_err());
    }

    #[test]
    fn modifiers_must_match_exactly() {
        let model = KeybindModel::default();
        let mut input = InputModel::default();

        // Plain S: no shortcut for it.
        InputController.key_event(&mut input, KeyCode::S, true);
        assert!(triggered_actions(model.data(), input.data()).is_empty());
        InputController.end_frame(&mut input);
        InputController.key_event(&mut input, KeyCode::S, false);
        InputController.end_frame(&mut input);

        // Ctrl held, then S pressed: save.
        InputController.key_event(&mut input, KeyCode::Ctrl, true);
        InputController.end_frame(&mut input);
        InputController.key_event(&mut input, KeyCode::S, true);

        assert_eq!(triggered_actions(model.data(), input.data()), vec![actions::SAVE]);
    }

    #[test]
    fn keybinds_survive_a_yaml_round_trip() {
        let model = KeybindModel::default();
        let loaded: KeybindData = from_yaml(&to_yaml(model.data()).unwrap()).unwrap();

        assert_eq!(&loaded, model.data());
    }
}
