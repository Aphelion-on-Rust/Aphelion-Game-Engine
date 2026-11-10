use super::interface::IdRegistryData;
use crate::mivc::Store;

/// Stores every id in use. No logic here, the IdController does the work.
pub type IdRegistryModel = Store<IdRegistryData>;
