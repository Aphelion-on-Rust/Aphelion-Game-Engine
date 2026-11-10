use super::interface::{Color, ThemeColorSlot};
use super::model::ThemeModel;
use super::query::find_theme;
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::mivc::{Controller, Model};

pub struct ThemeController;

impl Controller for ThemeController {}

fn invalid(message: String) -> EngineError {
    EngineError::InvalidOperation(message)
}

impl ThemeController {
    pub fn set_active(&self, model: &mut ThemeModel, name: &str) -> EngineResult<()> {
        if find_theme(model.data(), name).is_none() {
            return Err(invalid(format!("no theme called \"{name}\"")));
        }

        self.write(model).active = name.to_string();
        Ok(())
    }

    /// Copies a theme under a new name. The copy can be edited.
    pub fn create_custom(&self, model: &mut ThemeModel, base: &str, new_name: &str) -> EngineResult<()> {
        let mut copy = find_theme(model.data(), base)
            .ok_or_else(|| invalid(format!("no theme called \"{base}\"")))?
            .clone();

        if find_theme(model.data(), new_name).is_some() {
            return Err(invalid(format!("a theme called \"{new_name}\" already exists")));
        }

        copy.name = new_name.to_string();
        copy.built_in = false;
        self.write(model).themes.push(copy);
        Ok(())
    }

    pub fn set_color(
        &self,
        model: &mut ThemeModel,
        name: &str,
        slot: ThemeColorSlot,
        color: Color,
    ) -> EngineResult<()> {
        let theme = self
            .write(model)
            .themes
            .iter_mut()
            .find(|theme| theme.name == name)
            .ok_or_else(|| invalid(format!("no theme called \"{name}\"")))?;

        if theme.built_in {
            return Err(invalid("built-in themes can't be edited, make a copy first".to_string()));
        }

        theme.colors.set(slot, color);
        Ok(())
    }

    /// Deletes a custom theme. If it was the active one, the editor goes back to Dark.
    pub fn delete_custom(&self, model: &mut ThemeModel, name: &str) -> EngineResult<()> {
        let theme = find_theme(model.data(), name).ok_or_else(|| invalid(format!("no theme called \"{name}\"")))?;

        if theme.built_in {
            return Err(invalid("built-in themes can't be deleted".to_string()));
        }

        let data = self.write(model);

        data.themes.retain(|theme| theme.name != name);

        if data.active == name {
            data.active = "Dark".to_string();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::interface::ThemeSettingsData;
    use crate::theme::query::active_theme;
    use flux_engine::storage::yaml::{from_yaml, to_yaml};

    #[test]
    fn hex_colors_round_trip() {
        let color = Color::from_hex("#3D8BFF").unwrap();

        assert_eq!(color, Color::rgb(0x3D, 0x8B, 0xFF));
        assert_eq!(color.to_hex(), "#3D8BFF");
        assert_eq!(Color::from_hex("#3D8BFF80").unwrap().a, 0x80);
        assert!(Color::from_hex("3D8BFF").is_none());
        assert!(Color::from_hex("#12345").is_none());
        assert!(Color::from_hex("#GGGGGG").is_none());
    }

    #[test]
    fn built_in_themes_are_read_only() {
        let mut model = ThemeModel::default();

        assert!(ThemeController.set_color(&mut model, "Dark", ThemeColorSlot::Accent, Color::rgb(1, 2, 3)).is_err());
        assert!(ThemeController.delete_custom(&mut model, "Dark").is_err());
    }

    #[test]
    fn a_custom_theme_can_be_made_edited_and_deleted() {
        let mut model = ThemeModel::default();
        let red = Color::rgb(255, 0, 0);

        ThemeController.create_custom(&mut model, "Dark", "Mine").unwrap();
        ThemeController.set_color(&mut model, "Mine", ThemeColorSlot::Accent, red).unwrap();
        ThemeController.set_active(&mut model, "Mine").unwrap();
        assert_eq!(active_theme(model.data()).colors.accent, red);

        ThemeController.delete_custom(&mut model, "Mine").unwrap();
        assert_eq!(active_theme(model.data()).name, "Dark");
    }

    #[test]
    fn duplicate_theme_names_are_refused() {
        let mut model = ThemeModel::default();

        assert!(ThemeController.create_custom(&mut model, "Dark", "Light").is_err());
    }

    #[test]
    fn themes_survive_a_yaml_round_trip() {
        let model = ThemeModel::default();
        let loaded: ThemeSettingsData = from_yaml(&to_yaml(model.data()).unwrap()).unwrap();

        assert_eq!(&loaded, model.data());
    }
}
