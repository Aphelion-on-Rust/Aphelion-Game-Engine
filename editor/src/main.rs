//! Starts the editor window.
//!
//! The launcher starts it as `flux-editor --project <folder>`.
//! The editor opens that project's scene. Without it the editor starts with a demo scene.

use eframe::egui;
use flux_editor::ui::EditorApp;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let project = std::env::args().skip_while(|argument| argument != "--project").nth(1);
    let title = "Flux Aphelion".to_string();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title(title).with_inner_size([1600.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Flux Aphelion",
        options,
        Box::new(move |_creation_context| Ok(Box::new(EditorApp::new(project.map(PathBuf::from))))),
    )
}
