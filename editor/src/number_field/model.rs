use super::interface::NumberFieldData;
use flux_engine::mivc::Store;

/// One per field being used. The inspector keeps the one that is hovered or dragged.
pub type NumberFieldModel = Store<NumberFieldData>;
