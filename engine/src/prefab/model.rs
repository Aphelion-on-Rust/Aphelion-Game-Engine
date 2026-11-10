use super::interface::PrefabData;
use crate::mivc::Store;

/// Stores one prefab. No logic here, the PrefabController does the work.
pub type PrefabModel = Store<PrefabData>;
