//! Starts the launcher window.

use eframe::egui;
use flux_launcher::ui::LauncherApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Flux Aphelion Launcher")
            .with_inner_size([1000.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Flux Aphelion Launcher",
        options,
        Box::new(|_creation_context| Ok(Box::new(LauncherApp::new()))),
    )
}
