use crate::identity::AssetUuid;
use crate::math::{Rgba, Vec2, Vec3};
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

/// What kind of value a field holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum FieldKind {
    Bool,
    Int,
    Float,
    Text,
    Vec2,
    Vec3,
    Color,
    /// One of a fixed list of names.
    Enum { options: Vec<String> },
    /// A reference to another object in the scene.
    ObjectRef,
    /// A list. Every element has the same kind.
    List { element: Box<FieldKind> },
    /// A group of named fields: a [Serializable] class or struct.
    Struct { fields: Vec<FieldDescriptor> },
}

/// Something that changes how a field behaves or looks. They match the attributes a C# script
/// can put on a field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "attribute")]
pub enum FieldAttribute {
    /// [Range(min, max)]: shown as a slider, the value always stays between the two.
    Range { min: f64, max: f64 },
    /// [Min(value)]: the value never goes below it.
    Min { value: f64 },
    /// The name shown instead of the field name.
    Label { text: String },
    Tooltip { text: String },
    /// [Header("...")]: a title above the field.
    Header { text: String },
    /// [Space]: empty room above the field.
    Space { height: f32 },
    /// Shown, but can't be changed.
    ReadOnly,
    /// [HideInInspector]
    Hide,
    /// A text field with several lines.
    Multiline { lines: u32 },
    /// Fields with the same group name are shown together in one foldout.
    Group { name: String },
    /// Only shown while another field of the same object has this value.
    ShowIf { field: String, equals: FieldValue },
    /// Hidden while another field of the same object has this value.
    HideIf { field: String, equals: FieldValue },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub name: String,
    pub kind: FieldKind,
    /// The value a new instance starts with. Without it the kind's own default is used.
    #[serde(default)]
    pub default: Option<FieldValue>,
    #[serde(default)]
    pub attributes: Vec<FieldAttribute>,
}

/// A value of a field.
///
/// Adjacent tagging (`type` + `value`) is on purpose: YAML's `!Tag` form can't be read back
/// from inside the component data, which is read through a flattened, tagged enum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FieldValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Vec2(Vec2),
    Vec3(Vec3),
    Color(Rgba),
    Enum(String),
    ObjectRef(Option<AssetUuid>),
    List(Vec<FieldValue>),
    Struct(Vec<FieldEntry>),
}

/// A named value, used for the fields of a script and inside a Struct value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldEntry {
    pub name: String,
    pub value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathSegment {
    /// A field of the component or of a struct.
    Field(String),
    /// An element of a list.
    Index(usize),
}

/// Where a value is: `stats` > `health`, or `waypoints` > element 2.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldPath(Vec<PathSegment>);

impl FieldPath {
    pub fn field(name: &str) -> Self {
        Self(vec![PathSegment::Field(name.to_string())])
    }

    pub fn child_field(&self, name: &str) -> Self {
        let mut segments = self.0.clone();

        segments.push(PathSegment::Field(name.to_string()));
        Self(segments)
    }

    pub fn child_index(&self, index: usize) -> Self {
        let mut segments = self.0.clone();

        segments.push(PathSegment::Index(index));
        Self(segments)
    }

    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }
}

/// The fields of one script. Saved in `Library/script_schemas/<script uuid>.yaml`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptSchemaData {
    pub script: AssetUuid,
    pub class_name: String,
    pub fields: Vec<FieldDescriptor>,
}

impl Interface for ScriptSchemaData {
    const FORMAT: &'static str = "script_schema";
    const VERSION: u32 = 1;
}

/// The schemas of all scripts of the open project, in memory.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SchemaLibraryData {
    pub scripts: Vec<ScriptSchemaData>,
}

impl Interface for SchemaLibraryData {
    const FORMAT: &'static str = "schema_library";
    const VERSION: u32 = 1;
}
