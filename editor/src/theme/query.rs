use super::interface::{ThemeData, ThemeSettingsData};

pub fn find_theme<'a>(data: &'a ThemeSettingsData, name: &str) -> Option<&'a ThemeData> {
    data.themes.iter().find(|theme| theme.name == name)
}

/// The theme to draw with. Falls back to the first theme if the saved name is gone.
pub fn active_theme(data: &ThemeSettingsData) -> &ThemeData {
    find_theme(data, &data.active).unwrap_or(&data.themes[0])
}
