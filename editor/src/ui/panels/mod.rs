//! The content of each panel. Every panel is a view: `&EditorModels` in, commands out.

pub mod control;
pub mod hierarchy;
pub mod history;
pub mod inspector;
pub mod placeholder;
pub mod project;

use super::commands::UiCommand;
use super::models::EditorModels;
use crate::hierarchy::HierarchyView;
use crate::history::HistoryView;
use crate::layout::PanelKind;
use eframe::egui;
use flux_engine::control::ControlView;

pub const ALL_PANELS: [PanelKind; 12] = [
    PanelKind::Hierarchy,
    PanelKind::Scene,
    PanelKind::Game,
    PanelKind::Inspector,
    PanelKind::Project,
    PanelKind::Console,
    PanelKind::Animator,
    PanelKind::Timeline,
    PanelKind::Terrain,
    PanelKind::Profiler,
    PanelKind::History,
    PanelKind::Control,
];

pub fn panel_title(panel: PanelKind) -> &'static str {
    match panel {
        PanelKind::Hierarchy => "Hierarchy",
        PanelKind::Scene => "Scene",
        PanelKind::Game => "Game",
        PanelKind::Inspector => "Inspector",
        PanelKind::Project => "Project",
        PanelKind::Console => "Console",
        PanelKind::Animator => "Animator",
        PanelKind::Timeline => "Timeline",
        PanelKind::Terrain => "Terrain",
        PanelKind::Profiler => "Profiler",
        PanelKind::History => "History",
        PanelKind::Control => "Aphelion's Control",
    }
}

/// Screen state the panels keep between frames (not data).
#[derive(Default)]
pub struct PanelViews {
    pub hierarchy: HierarchyView,
    pub rename: hierarchy::RenameBuffer,
    pub history: HistoryView,
    pub control: ControlView,
}

pub fn show_panel(
    panel: PanelKind,
    ui: &mut egui::Ui,
    models: &EditorModels,
    views: &mut PanelViews,
    commands: &mut Vec<UiCommand>,
) {
    match panel {
        PanelKind::Hierarchy => hierarchy::show(ui, &mut views.hierarchy, &mut views.rename, models, commands),
        PanelKind::Inspector => inspector::show(ui, models, commands),
        PanelKind::Project => project::show(ui, models, commands),
        PanelKind::Control => control::show(ui, &mut views.control, models, commands),
        PanelKind::History => history::show(ui, &mut views.history, models, commands),
        other => placeholder::show(ui, other),
    }
}
