use super::interface::{PrefabLibraryData, PrefabListing};
use flux_engine::identity::AssetUuid;

pub fn find_by_uuid<'a>(data: &'a PrefabLibraryData, uuid: &AssetUuid) -> Option<&'a PrefabListing> {
    data.prefabs.iter().find(|listing| listing.uuid == *uuid)
}

pub fn find_by_name<'a>(data: &'a PrefabLibraryData, name: &str) -> Option<&'a PrefabListing> {
    data.prefabs.iter().find(|listing| listing.name == name)
}
