use crate::identity::{AssetUuid, ComponentId};
use crate::input::MouseButton;
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

/// Every event a script can react to. The name of the C# method is `name()`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LifecycleEvent {
    // ----- setup and teardown -----
    /// Editor only: script was added or "Reset" was pressed.
    Reset,
    /// Editor only: a value was changed in the inspector.
    OnValidate,
    /// Once, the first time the object is active. Even if the script is disabled.
    Awake,
    /// Every time the script becomes enabled (and the object active).
    OnEnable,
    /// Once, before the first update, only after the script is enabled.
    Start,
    OnDisable,
    OnDestroy,

    // ----- every frame -----
    FixedUpdate,
    Update,
    /// During animation, after Update. `layer` is the animator layer.
    OnIK { layer: u32 },
    LateUpdate,
    OnPreRender,
    OnPostRender,
    OnDrawGizmos,

    // ----- mouse (the object under the cursor) -----
    OnMouseDown { button: MouseButton },
    OnMouseUp { button: MouseButton },
    /// Pressed and released on the same object.
    OnMouseClick { button: MouseButton },
    OnMouseEnter,
    OnMouseExit,
    /// Every frame while the cursor is over the object.
    OnMouseOver,
    /// Every frame while a button that was pressed on the object is held.
    OnMouseDrag { button: MouseButton },

    // ----- physics (2D and 3D share these) -----
    OnCollisionEnter { other: AssetUuid },
    OnCollisionStay { other: AssetUuid },
    OnCollisionExit { other: AssetUuid },
    OnTriggerEnter { other: AssetUuid },
    OnTriggerStay { other: AssetUuid },
    OnTriggerExit { other: AssetUuid },
    OnParticleCollision { other: AssetUuid },
    OnJointBreak { force: f32 },

    // ----- visibility -----
    OnBecameVisible,
    OnBecameInvisible,

    // ----- hierarchy -----
    OnTransformParentChanged,
    OnTransformChildrenChanged,

    // ----- animation -----
    OnAnimationEvent { name: String },

    // ----- application -----
    OnApplicationFocus { has_focus: bool },
    OnApplicationPause { is_paused: bool },
    OnApplicationQuit,
}

impl LifecycleEvent {
    /// The name of the method in the C# script.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Reset => "Reset",
            Self::OnValidate => "OnValidate",
            Self::Awake => "Awake",
            Self::OnEnable => "OnEnable",
            Self::Start => "Start",
            Self::OnDisable => "OnDisable",
            Self::OnDestroy => "OnDestroy",
            Self::FixedUpdate => "FixedUpdate",
            Self::Update => "Update",
            Self::OnIK { .. } => "OnIK",
            Self::LateUpdate => "LateUpdate",
            Self::OnPreRender => "OnPreRender",
            Self::OnPostRender => "OnPostRender",
            Self::OnDrawGizmos => "OnDrawGizmos",
            Self::OnMouseDown { .. } => "OnMouseDown",
            Self::OnMouseUp { .. } => "OnMouseUp",
            Self::OnMouseClick { .. } => "OnMouseClick",
            Self::OnMouseEnter => "OnMouseEnter",
            Self::OnMouseExit => "OnMouseExit",
            Self::OnMouseOver => "OnMouseOver",
            Self::OnMouseDrag { .. } => "OnMouseDrag",
            Self::OnCollisionEnter { .. } => "OnCollisionEnter",
            Self::OnCollisionStay { .. } => "OnCollisionStay",
            Self::OnCollisionExit { .. } => "OnCollisionExit",
            Self::OnTriggerEnter { .. } => "OnTriggerEnter",
            Self::OnTriggerStay { .. } => "OnTriggerStay",
            Self::OnTriggerExit { .. } => "OnTriggerExit",
            Self::OnParticleCollision { .. } => "OnParticleCollision",
            Self::OnJointBreak { .. } => "OnJointBreak",
            Self::OnBecameVisible => "OnBecameVisible",
            Self::OnBecameInvisible => "OnBecameInvisible",
            Self::OnTransformParentChanged => "OnTransformParentChanged",
            Self::OnTransformChildrenChanged => "OnTransformChildrenChanged",
            Self::OnAnimationEvent { .. } => "OnAnimationEvent",
            Self::OnApplicationFocus { .. } => "OnApplicationFocus",
            Self::OnApplicationPause { .. } => "OnApplicationPause",
            Self::OnApplicationQuit => "OnApplicationQuit",
        }
    }
}

/// What we know about one script component while the game runs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceState {
    pub component: ComponentId,
    pub object: AssetUuid,
    pub script: AssetUuid,
    /// Awake has been called.
    pub awoken: bool,
    /// Enabled and active right now (OnEnable called, OnDisable not yet).
    pub running: bool,
    /// Start has been called.
    pub started: bool,
}

/// An event waiting for the next dispatch, aimed at one object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingEvent {
    pub object: AssetUuid,
    pub event: LifecycleEvent,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LifecycleData {
    pub instances: Vec<InstanceState>,
    pub pending_events: Vec<PendingEvent>,
}

impl Interface for LifecycleData {
    const FORMAT: &'static str = "lifecycle_state";
    const VERSION: u32 = 1;
}

/// "Call this event on this script component." The ScriptHost carries it out.
#[derive(Clone, Debug, PartialEq)]
pub struct Dispatch {
    pub object: AssetUuid,
    pub component: ComponentId,
    pub script: AssetUuid,
    pub event: LifecycleEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PressedOn {
    pub button: MouseButton,
    pub object: AssetUuid,
}

/// Remembers what the mouse was over and where each button was pressed.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MouseTrackerData {
    pub hovered: Option<AssetUuid>,
    pub pressed_on: Vec<PressedOn>,
}

impl Interface for MouseTrackerData {
    const FORMAT: &'static str = "mouse_tracker";
    const VERSION: u32 = 1;
}
