//! The Scene and Game views: the scene drawn by the software renderer (`flux-grp`).
//!
//! The Game view uses the scene's camera (the ViewCamera with the highest priority). The Scene view
//! uses a built-in camera for now, the editor camera (orbit, pan, zoom) comes in the next step.
//!
//! Drawing on the CPU costs time, so the picture is only drawn again when the scene or the size
//! of the panel changed, and it is never drawn wider than 640 pixels (it is stretched to fit).
//! Run the editor in release mode (`cargo run --release -p flux-editor`) for a smooth view.

use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::mivc::Model;
use flux_engine::scene::SceneData;
use flux_grp::backend::RenderBackend;
use flux_grp::frame::{collect, fallback_camera, find_game_camera};
use flux_grp::software::SoftwareRenderer;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    Scene,
    Game,
}

/// The last picture of a view and what it was drawn from. Screen state, not data.
#[derive(Default)]
pub struct ViewCache {
    texture: Option<egui::TextureHandle>,
    last_scene: Option<SceneData>,
    last_size: [usize; 2],
}

const MAX_WIDTH: f32 = 640.0;

pub fn show(ui: &mut egui::Ui, cache: &mut ViewCache, kind: ViewKind, models: &EditorModels) {
    let available = ui.available_size();

    if available.x < 16.0 || available.y < 16.0 {
        return;
    }

    let scale = (MAX_WIDTH / available.x).min(1.0);
    let width = ((available.x * scale) as usize).max(8);
    let height = ((available.y * scale) as usize).max(8);
    let aspect = width as f32 / height as f32;
    let scene = models.scene.data();
    let game_camera = find_game_camera(scene, aspect);

    if kind == ViewKind::Game && game_camera.is_none() {
        ui.weak("No camera in the scene, showing a default view. Add one with + Create > Camera.");
    }

    let up_to_date =
        cache.texture.is_some() && cache.last_size == [width, height] && cache.last_scene.as_ref() == Some(scene);

    if !up_to_date {
        let camera = match kind {
            ViewKind::Game => game_camera.unwrap_or_else(|| fallback_camera(aspect)),
            ViewKind::Scene => fallback_camera(aspect),
        };
        let image = SoftwareRenderer.render(&collect(scene, camera), width, height);
        let picture = egui::ColorImage::from_rgba_unmultiplied([width, height], &image.rgba);

        match &mut cache.texture {
            Some(texture) => texture.set(picture, egui::TextureOptions::LINEAR),
            None => {
                let name = if kind == ViewKind::Scene { "scene_view" } else { "game_view" };

                cache.texture = Some(ui.ctx().load_texture(name, picture, egui::TextureOptions::LINEAR));
            }
        }

        cache.last_scene = Some(scene.clone());
        cache.last_size = [width, height];
    }

    if let Some(texture) = &cache.texture {
        ui.image((texture.id(), egui::vec2(width as f32 / scale, height as f32 / scale)));
    }
}
