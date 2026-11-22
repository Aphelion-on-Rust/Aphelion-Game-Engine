//! The Scene and Game views: the scene drawn by the software renderer (`flux-grp`).
//!
//! Game: the scene's camera (the ViewCamera with the highest priority).
//! Scene: your own orbit camera. Right-drag orbits, middle-drag pans, the wheel zooms, a click
//! selects the object under the mouse (it is drawn tinted orange), F focuses on the selection.
//!
//! Drawing on the CPU costs time, so the picture is only drawn again when the scene, the size of the
//! panel, the camera or the selection changed, and it is never drawn wider than 640 pixels (it is
//! stretched). Run the editor in release mode (`cargo run --release -p flux-editor`) for a smooth view.

use crate::assets_io::load_texture;
use crate::scene_view::query::render_camera;
use crate::scene_view::SceneCameraData;
use crate::ui::commands::UiCommand;
use crate::ui::models::EditorModels;
use eframe::egui;
use flux_engine::assets::{query as asset_query, AssetLibraryData, TextureFilter};
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::Model;
use flux_engine::physics::extract_colliders;
use flux_engine::scene::{ComponentData, SceneData};
use flux_grp::backend::RenderBackend;
use flux_grp::frame::{collect, fallback_camera, find_game_camera, TextureImage};
use flux_grp::gizmos::collider_lines;
use flux_grp::software::SoftwareRenderer;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

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
    last_camera: Option<SceneCameraData>,
    last_selected: Vec<AssetUuid>,
    last_assets: Option<AssetLibraryData>,
    /// The pictures read from disk so far, so they are not read again for every frame.
    textures: HashMap<AssetUuid, LoadedTexture>,
}

/// A picture that was read from disk (`image` is None if it could not be read).
struct LoadedTexture {
    path: String,
    nearest: bool,
    modified: Option<SystemTime>,
    image: Option<Arc<TextureImage>>,
}

const MAX_WIDTH: f32 = 640.0;

pub fn show(ui: &mut egui::Ui, cache: &mut ViewCache, kind: ViewKind, models: &EditorModels, commands: &mut Vec<UiCommand>) {
    let available = ui.available_size();

    if available.x < 16.0 || available.y < 16.0 {
        return;
    }

    let scale = (MAX_WIDTH / available.x).min(1.0);
    let width = ((available.x * scale) as usize).max(8);
    let height = ((available.y * scale) as usize).max(8);
    let aspect = width as f32 / height as f32;
    let scene = models.scene.data();
    let camera_data = models.scene_camera.data();
    let selected = &models.hierarchy_ui.data().selected;
    let game_camera = find_game_camera(scene, aspect);

    match kind {
        ViewKind::Game if game_camera.is_none() => {
            ui.weak("No camera in the scene, showing a default view. Add one with + Create > Camera.");
        }
        ViewKind::Scene => {
            ui.weak("Right-drag: orbit   Middle-drag: pan   Wheel: zoom   Click: select   F: focus");
        }
        ViewKind::Game => {}
    }

    // The Scene view also depends on its camera and the selection, the Game view does not.
    let same_view = kind == ViewKind::Game || (cache.last_camera.as_ref() == Some(camera_data) && cache.last_selected == *selected);
    let up_to_date = cache.texture.is_some() && cache.last_size == [width, height] && cache.last_scene.as_ref() == Some(scene) && same_view
        && cache.last_assets.as_ref() == Some(models.assets.data());

    if !up_to_date {
        let textures = load_textures(cache, models, scene);
        let mut frame = match kind {
            ViewKind::Game => collect(scene, game_camera.unwrap_or_else(|| fallback_camera(aspect))),
            ViewKind::Scene => collect(scene, render_camera(camera_data, aspect)),
        };

        if kind == ViewKind::Scene {
            frame.selected = selected.clone();

            // The outline of the colliders of the selected objects: green, or cyan for a trigger.
            for collider in extract_colliders(scene).iter().filter(|collider| selected.contains(&collider.object)) {
                let color = if collider.trigger { [0.2, 0.85, 1.0, 1.0] } else { [0.3, 1.0, 0.4, 1.0] };

                frame.lines.extend(collider_lines(collider, color));
            }
        }

        frame.textures = textures;

        let image = SoftwareRenderer.render(&frame, width, height);
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
        cache.last_camera = Some(camera_data.clone());
        cache.last_selected = selected.clone();
        cache.last_assets = Some(models.assets.data().clone());
    }

    if let Some(texture) = &cache.texture {
        let size = egui::vec2(width as f32 / scale, height as f32 / scale);
        let response = ui.add(egui::Image::new(egui::load::SizedTexture::new(texture.id(), size)).sense(egui::Sense::click_and_drag()));

        if kind == ViewKind::Scene {
            mouse_controls(ui, &response, aspect, commands);
        }
    }
}

/// Turns what the mouse does over the Scene view into commands.
fn mouse_controls(ui: &egui::Ui, response: &egui::Response, aspect: f32, commands: &mut Vec<UiCommand>) {
    if response.dragged_by(egui::PointerButton::Secondary) {
        let drag = response.drag_delta();

        commands.push(UiCommand::SceneViewOrbit { dx: drag.x, dy: drag.y });
    }

    if response.dragged_by(egui::PointerButton::Middle) {
        let drag = response.drag_delta();

        commands.push(UiCommand::SceneViewPan { dx: drag.x, dy: drag.y });
    }

    if response.hovered() {
        let scroll = ui.input(|state| state.raw_scroll_delta.y);

        if scroll != 0.0 {
            commands.push(UiCommand::SceneViewZoom(scroll));
        }
    }

    if response.clicked_by(egui::PointerButton::Primary) {
        if let Some(position) = response.interact_pointer_pos() {
            let rect = response.rect;
            let additive = ui.input(|state| state.modifiers.ctrl || state.modifiers.command);

            commands.push(UiCommand::PickAt {
                ndc_x: (position.x - rect.left()) / rect.width() * 2.0 - 1.0,
                // The picture's y goes down, the ray's goes up.
                ndc_y: 1.0 - (position.y - rect.top()) / rect.height() * 2.0,
                aspect,
                additive,
            });
        }
    }
}

/// The pictures the scene's meshes and sprites use. They are read from disk the first time, and again
/// when the file changed on disk, its path changed, or its filter changed.
fn load_textures(cache: &mut ViewCache, models: &EditorModels, scene: &SceneData) -> HashMap<AssetUuid, Arc<TextureImage>> {
    let mut wanted: Vec<AssetUuid> = Vec::new();

    for object in &scene.objects {
        for entry in &object.components {
            let texture = match &entry.data {
                ComponentData::MeshDraw(mesh) => mesh.texture,
                ComponentData::SpriteDraw(sprite) => sprite.texture,
                _ => None,
            };

            if let Some(uuid) = texture {
                if !wanted.contains(&uuid) {
                    wanted.push(uuid);
                }
            }
        }
    }

    let mut found = HashMap::new();
    let Some(project) = &models.project else {
        return found;
    };

    for uuid in wanted {
        let Some(asset) = asset_query::find_by_uuid(models.assets.data(), &uuid) else {
            continue;
        };
        let file = project.root.join(&asset.path);
        let nearest = asset.filter == TextureFilter::Nearest;
        let modified = std::fs::metadata(&file).and_then(|info| info.modified()).ok();
        let fresh = cache
            .textures
            .get(&uuid)
            .map_or(false, |loaded| loaded.path == asset.path && loaded.nearest == nearest && loaded.modified == modified);

        if !fresh {
            let image = load_texture(&file, nearest).ok().map(Arc::new);

            cache.textures.insert(uuid, LoadedTexture { path: asset.path.clone(), nearest, modified, image });
        }

        if let Some(image) = cache.textures.get(&uuid).and_then(|loaded| loaded.image.clone()) {
            found.insert(uuid, image);
        }
    }

    found
}
