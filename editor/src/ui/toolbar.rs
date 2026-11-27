//! The Play / Pause / Stop buttons under the menu bar.

use super::commands::UiCommand;
use super::models::EditorModels;
use crate::play::query::{in_play_mode, label};
use crate::play::PlayState;
use eframe::egui;
use flux_engine::mivc::Model;

pub fn show(ctx: &egui::Context, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let play = models.play.data();

            if in_play_mode(play) {
                if ui.button("Stop").on_hover_text("Put the scene back as it was").clicked() {
                    commands.push(UiCommand::PlayStop);
                }

                let pause_text = if play.state == PlayState::Paused { "Resume" } else { "Pause" };

                if ui.button(pause_text).clicked() {
                    commands.push(UiCommand::PlayPause);
                }
            } else if ui.button("Play").on_hover_text("Run the scene with physics (Ctrl+P). Stop puts everything back.").clicked() {
                commands.push(UiCommand::PlayToggle);
            }

            ui.weak(label(play));

            if !play.last_event.is_empty() {
                ui.weak(format!("   |   {}", play.last_event));
            }
        });
    });
}
