use super::interface::SceneData;
use crate::mivc::Store;

/// Stores one scene. No logic here, the SceneController does the work.
pub type SceneModel = Store<SceneData>;
