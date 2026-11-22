use super::interface::{AssetEntry, AssetKind, AssetLibraryData};
use crate::identity::AssetUuid;

pub fn find_by_uuid<'a>(data: &'a AssetLibraryData, uuid: &AssetUuid) -> Option<&'a AssetEntry> {
    data.assets.iter().find(|asset| asset.uuid == *uuid)
}

pub fn find_by_path<'a>(data: &'a AssetLibraryData, path: &str) -> Option<&'a AssetEntry> {
    data.assets.iter().find(|asset| asset.path == path)
}

pub fn of_kind(data: &AssetLibraryData, kind: AssetKind) -> Vec<&AssetEntry> {
    data.assets.iter().filter(|asset| asset.kind == kind).collect()
}
