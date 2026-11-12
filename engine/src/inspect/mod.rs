//! The data and rules behind the Eclipse Inspector (the inspector UI is drawn in the editor).
//!
//! A component or script describes its fields (name, kind, attributes like [Range]) and the
//! Inspector draws widgets from that description, so a new component needs no new Inspector code.
//!
//! - interface.rs   field kinds, attributes, values, paths, script schemas (all YAML)
//! - rules.rs       what a value may be: kind checks, [Range], [Min], enum options, read-only
//! - access.rs      reading and writing a value anywhere inside nested structs and lists
//! - defaults.rs    default values, filling in missing script fields
//! - builtin.rs     the descriptions of Transform and RectTransform
//! - query.rs       schema lookup, labels, show/hide rules, the layout (headers, groups)
//! - controller.rs  changing fields of a component in a scene, with all the rules applied
//!
//! Where the descriptions of script fields come from: the scripting host (step 9) reads the
//! C# classes ([Serializable], [SerializeField], [Range] ...) and writes one schema file per script
//! into `Library/script_schemas`. Until then the files can be written by hand.

pub mod access;
pub mod builtin;
pub mod controller;
pub mod defaults;
pub mod interface;
pub mod model;
pub mod query;
pub mod rules;

pub use controller::InspectController;
pub use interface::{
    FieldAttribute, FieldDescriptor, FieldEntry, FieldKind, FieldPath, FieldValue, PathSegment,
    SchemaLibraryData, ScriptSchemaData,
};
pub use model::SchemaLibraryModel;
