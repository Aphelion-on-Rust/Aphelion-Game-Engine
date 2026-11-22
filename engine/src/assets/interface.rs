use crate::identity::AssetUuid;
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    /// A picture (png, jpg, jpeg).
    Texture,
}

impl AssetKind {
    /// The kind of asset a file extension stands for (any capitals), or None for files that are not assets.
    pub fn from_extension(extension: &str) -> Option<AssetKind> {
        match extension.to_ascii_lowercase().as_str() {
            "png" | "jpg" | "jpeg" => Some(AssetKind::Texture),
            _ => None,
        }
    }

    /// The name used in field descriptions (`FieldKind::AssetRef { asset }`).
    pub fn name(&self) -> &'static str {
        match self {
            AssetKind::Texture => "Texture",
        }
    }
}

/// How a texture is read when it is drawn bigger or smaller than it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilter {
    /// Smooth. Good for most pictures.
    #[default]
    Linear,
    /// Hard pixels. For pixel art.
    Nearest,
}

/// The contents of a `<file>.meta.yaml`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetMetaData {
    /// Never changes.
    pub uuid: AssetUuid,
    #[serde(default)]
    pub filter: TextureFilter,
}

impl Interface for AssetMetaData {
    const FORMAT: &'static str = "asset_meta";
    const VERSION: u32 = 1;
}

/// One asset of the open project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetEntry {
    pub uuid: AssetUuid,
    /// Inside the project, written with `/`, like `Assets/Art/Hero.png`.
    pub path: String,
    pub kind: AssetKind,
    pub filter: TextureFilter,
}

/// All assets of the open project, sorted by path.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetLibraryData {
    pub assets: Vec<AssetEntry>,
}

impl Interface for AssetLibraryData {
    const FORMAT: &'static str = "asset_library";
    const VERSION: u32 = 1;
}
