use super::interface::{LifecycleData, MouseTrackerData};
use crate::mivc::Store;

pub type LifecycleModel = Store<LifecycleData>;
pub type MouseTrackerModel = Store<MouseTrackerData>;
