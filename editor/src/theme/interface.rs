use flux_engine::mivc::Interface;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A color. In YAML it is written as "#RRGGBB" (or "#RRGGBBAA" when not fully opaque).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_hex(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#')?;

        if !hex.is_ascii() || (hex.len() != 6 && hex.len() != 8) {
            return None;
        }

        let part = |start: usize| u8::from_str_radix(&hex[start..start + 2], 16).ok();
        let alpha = if hex.len() == 8 { part(6)? } else { 255 };

        Some(Self { r: part(0)?, g: part(2)?, b: part(4)?, a: alpha })
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;

        Color::from_hex(&text)
            .ok_or_else(|| D::Error::custom(format!("\"{text}\" is not a color like #RRGGBB")))
    }
}

/// The name of one color in a theme. Used when editing a single color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeColorSlot {
    Background,
    Panel,
    PanelHeader,
    Text,
    TextDim,
    Border,
    Accent,
    AccentHover,
    Selection,
    Error,
    Warning,
    Success,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: Color,
    pub panel: Color,
    pub panel_header: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub selection: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
}

impl ThemeColors {
    pub fn get(&self, slot: ThemeColorSlot) -> Color {
        match slot {
            ThemeColorSlot::Background => self.background,
            ThemeColorSlot::Panel => self.panel,
            ThemeColorSlot::PanelHeader => self.panel_header,
            ThemeColorSlot::Text => self.text,
            ThemeColorSlot::TextDim => self.text_dim,
            ThemeColorSlot::Border => self.border,
            ThemeColorSlot::Accent => self.accent,
            ThemeColorSlot::AccentHover => self.accent_hover,
            ThemeColorSlot::Selection => self.selection,
            ThemeColorSlot::Error => self.error,
            ThemeColorSlot::Warning => self.warning,
            ThemeColorSlot::Success => self.success,
        }
    }

    pub fn set(&mut self, slot: ThemeColorSlot, color: Color) {
        match slot {
            ThemeColorSlot::Background => self.background = color,
            ThemeColorSlot::Panel => self.panel = color,
            ThemeColorSlot::PanelHeader => self.panel_header = color,
            ThemeColorSlot::Text => self.text = color,
            ThemeColorSlot::TextDim => self.text_dim = color,
            ThemeColorSlot::Border => self.border = color,
            ThemeColorSlot::Accent => self.accent = color,
            ThemeColorSlot::AccentHover => self.accent_hover = color,
            ThemeColorSlot::Selection => self.selection = color,
            ThemeColorSlot::Error => self.error = color,
            ThemeColorSlot::Warning => self.warning = color,
            ThemeColorSlot::Success => self.success = color,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeData {
    pub name: String,
    /// Built-in themes can't be edited or deleted.
    pub built_in: bool,
    pub colors: ThemeColors,
}

impl ThemeData {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            built_in: true,
            colors: ThemeColors {
                background: Color::rgb(0x1E, 0x1E, 0x1E),
                panel: Color::rgb(0x25, 0x25, 0x26),
                panel_header: Color::rgb(0x2D, 0x2D, 0x30),
                text: Color::rgb(0xE6, 0xE6, 0xE6),
                text_dim: Color::rgb(0x9A, 0x9A, 0x9A),
                border: Color::rgb(0x3C, 0x3C, 0x3C),
                accent: Color::rgb(0x3D, 0x8B, 0xFF),
                accent_hover: Color::rgb(0x5F, 0xA0, 0xFF),
                selection: Color::rgb(0x2F, 0x5D, 0x9E),
                error: Color::rgb(0xF0, 0x52, 0x4F),
                warning: Color::rgb(0xE5, 0xB0, 0x3A),
                success: Color::rgb(0x4C, 0xC3, 0x8A),
            },
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            built_in: true,
            colors: ThemeColors {
                background: Color::rgb(0xF2, 0xF2, 0xF2),
                panel: Color::rgb(0xFF, 0xFF, 0xFF),
                panel_header: Color::rgb(0xE4, 0xE4, 0xE4),
                text: Color::rgb(0x1E, 0x1E, 0x1E),
                text_dim: Color::rgb(0x6B, 0x6B, 0x6B),
                border: Color::rgb(0xC8, 0xC8, 0xC8),
                accent: Color::rgb(0x2A, 0x6F, 0xDB),
                accent_hover: Color::rgb(0x4A, 0x8B, 0xF0),
                selection: Color::rgb(0xBB, 0xD4, 0xF8),
                error: Color::rgb(0xD9, 0x30, 0x25),
                warning: Color::rgb(0xB7, 0x79, 0x1F),
                success: Color::rgb(0x1E, 0x8E, 0x5A),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeSettingsData {
    /// Name of the theme in use.
    pub active: String,
    pub themes: Vec<ThemeData>,
}

impl Default for ThemeSettingsData {
    fn default() -> Self {
        Self { active: "Dark".to_string(), themes: vec![ThemeData::dark(), ThemeData::light()] }
    }
}

impl Interface for ThemeSettingsData {
    const FORMAT: &'static str = "editor_themes";
    const VERSION: u32 = 1;
}
