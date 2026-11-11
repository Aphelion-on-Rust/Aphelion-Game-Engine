use super::panel_title;
use crate::layout::PanelKind;
use eframe::egui;

/// Panels whose real content arrives in a later step.
pub fn show(ui: &mut egui::Ui, panel: PanelKind) {
    let note = match panel {
        PanelKind::Scene | PanelKind::Game => "The 3D/2D view arrives with the renderer (step 7).",
        PanelKind::Console => "Log output will appear here.",
        PanelKind::Animator | PanelKind::Timeline => "Animation tools arrive in step 11.",
        PanelKind::Terrain => "The terrain tool arrives in step 14.",
        PanelKind::Profiler => "Performance numbers will appear here.",
        PanelKind::Hierarchy | PanelKind::Inspector | PanelKind::Project | PanelKind::History => "",
    };

    ui.label(egui::RichText::new(panel_title(panel)).strong());
    ui.weak(note);
}
