use flux_engine::mivc::Interface;
use flux_engine::scene::SceneData;
use serde::{Deserialize, Serialize};

/// One step in the history.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// What the step did, like "Create object".
    pub label: String,
    /// Steps with the same key that follow each other are merged into one (typing a name).
    pub merge_key: Option<String>,
    /// The scene to go back to when this step is undone.
    pub scene: SceneData,
}

/// A step that is still going on (dragging a number). It becomes one entry when it ends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenGroup {
    pub label: String,
    pub before: SceneData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryData {
    /// Oldest first. The last one is the next to undo.
    pub undo: Vec<HistoryEntry>,
    /// The last one is the next to redo.
    pub redo: Vec<HistoryEntry>,
    pub open_group: Option<OpenGroup>,
    /// The oldest steps are dropped beyond this.
    pub limit: usize,
}

impl Default for HistoryData {
    fn default() -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), open_group: None, limit: 100 }
    }
}

impl Interface for HistoryData {
    const FORMAT: &'static str = "edit_history";
    const VERSION: u32 = 1;
}
