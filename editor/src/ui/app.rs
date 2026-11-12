use super::commands::{action_to_command, apply_all, ControllerState, UiCommand};
use super::dock_view::DockView;
use super::models::EditorModels;
use super::panels::PanelViews;
use super::{input_bridge, menu_bar, prefab_flow, theme_apply};
use crate::keybinds::query::triggered_actions;
use crate::number_field::query::cursor_icon;
use crate::number_field::{wrapped_x, CursorIcon, DragPhase, NumberFieldController};
use crate::project_io;
use crate::theme::query::active_theme;
use crate::theme::ThemeData;
use eframe::egui;
use flux_engine::input::InputController;
use flux_engine::mivc::{Model, Store};
use flux_engine::scene::SceneData;
use flux_engine::storage::SettingsStore;
use std::path::PathBuf;

/// The editor window. It owns the models and runs the frame:
/// read input -> draw (collect commands) -> apply commands -> save / cursor / title.
pub struct EditorApp {
    models: EditorModels,
    state: ControllerState,
    dock_view: DockView,
    panel_views: PanelViews,
    store: SettingsStore,
    /// The theme egui was last given, so we only re-apply when it changes.
    applied_theme: Option<ThemeData>,
    pointer_hidden: bool,
    /// The scene as it is on disk. If the scene differs from it, there are unsaved changes.
    saved_scene: Option<SceneData>,
    last_title: String,
    /// The "unsaved changes" question is showing.
    close_prompt: bool,
    /// The question was answered, the window may close now.
    allow_close: bool,
}

impl EditorApp {
    /// Opens the project in `project_folder`, or starts with a demo scene if there is none.
    pub fn new(project_folder: Option<PathBuf>) -> Self {
        let store = SettingsStore::for_app("editor");

        let (mut models, status, saved_scene) = match project_folder {
            Some(root) => match project_io::open_project(&root) {
                Ok(opened) => {
                    let saved = opened.scene.data().clone();
                    let status = match opened.warnings.first() {
                        Some(first) => format!(
                            "Opened project {}, but {} file(s) could not be read: {first}",
                            opened.data.name,
                            opened.warnings.len()
                        ),
                        None => format!("Opened project {}", opened.data.name),
                    };

                    (EditorModels::from_opened(opened), status, Some(saved))
                }
                Err(error) => (EditorModels::new_demo(), format!("could not open the project: {error}"), None),
            },
            None => (
                EditorModels::new_demo(),
                "No project open (demo scene). Start the editor from the launcher.".to_string(),
                None,
            ),
        };

        models.layout = Store::new(store.load_or_default("layout.yaml"));
        models.theme = Store::new(store.load_or_default("themes.yaml"));
        models.keybinds = Store::new(store.load_or_default("keybinds.yaml"));

        Self {
            models,
            state: ControllerState { status, ..Default::default() },
            dock_view: DockView::new(),
            panel_views: PanelViews::default(),
            store,
            applied_theme: None,
            pointer_hidden: false,
            saved_scene,
            last_title: String::new(),
            close_prompt: false,
            allow_close: false,
        }
    }

    fn save_settings(&mut self) {
        let results = [
            self.store.save("layout.yaml", self.models.layout.data()),
            self.store.save("themes.yaml", self.models.theme.data()),
            self.store.save("keybinds.yaml", self.models.keybinds.data()),
        ];

        if let Some(Err(error)) = results.into_iter().find(|result| result.is_err()) {
            self.state.status = format!("could not save settings: {error}");
        }

        self.state.settings_dirty = false;
    }

    fn scene_is_dirty(&self) -> bool {
        match &self.saved_scene {
            Some(saved) => saved != self.models.scene.data(),
            None => false,
        }
    }

    fn window_title(&self) -> String {
        let name = match &self.models.project {
            Some(project) => project.data.data().name.clone(),
            None => "no project".to_string(),
        };
        let mark = if self.scene_is_dirty() { " *" } else { "" };

        format!("{name}{mark} - Flux Aphelion")
    }

    /// Writes the scene and the id registry. Both files are written safely (temp file, then rename).
    fn save_scene(&mut self) {
        let Some(project) = &self.models.project else {
            self.state.status = "No project is open, so there is nothing to save to.".to_string();
            return;
        };

        let result = project_io::save_scene(&project.scene_path, &self.models.scene)
            .and_then(|()| project_io::save_ids(&project.root, &self.models.ids));

        match result {
            Ok(()) => {
                self.state.status = format!("Saved {}", project.scene_path.display());
                self.saved_scene = Some(self.models.scene.data().clone());
            }
            Err(error) => self.state.status = format!("could not save the scene: {error}"),
        }
    }

    /// Moves the pointer to the other side of the window when it reaches an edge,
    /// so a long drag never runs out of room.
    fn wrap_pointer(&self, ctx: &egui::Context) {
        let Some(position) = ctx.input(|state| state.pointer.latest_pos()) else {
            return;
        };

        if let Some(x) = wrapped_x(position.x, ctx.screen_rect().width(), 4.0) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorPosition(egui::pos2(x, position.y)));
        }
    }

    /// "The scene has unsaved changes" with Save and close / Close without saving / Cancel.
    fn show_close_prompt(&mut self, ctx: &egui::Context) {
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("The scene has changes that are not saved.");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Save and close").clicked() {
                        self.save_scene();
                        self.close_prompt = false;

                        // If saving failed the status bar says why, and the editor stays open.
                        if !self.scene_is_dirty() {
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    if ui.button("Close without saving").clicked() {
                        self.allow_close = true;
                        self.close_prompt = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if ui.button("Cancel").clicked() {
                        self.close_prompt = false;
                    }
                });
            });
    }

    /// Shows the <-> cursor over number fields and hides the pointer while dragging.
    fn update_cursor(&mut self, ctx: &egui::Context) {
        let icon = cursor_icon(self.models.number_field.data());
        let should_hide = icon == CursorIcon::Hidden;

        if icon == CursorIcon::HorizontalResize {
            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }

        // While the pointer is hidden for a number drag, it wraps around the window edges.
        if should_hide {
            self.wrap_pointer(ctx);
        }

        if should_hide != self.pointer_hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(!should_hide));
            self.pointer_hidden = should_hide;
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new(None)
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 0. Closing the window with unsaved changes asks first.
        if ctx.input(|state| state.viewport().close_requested()) && !self.allow_close && self.scene_is_dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_prompt = true;
        }

        // 1. Theme.
        let theme = active_theme(self.models.theme.data()).clone();

        if self.applied_theme.as_ref() != Some(&theme) {
            theme_apply::apply(ctx, &theme);
            self.applied_theme = Some(theme);
        }

        // 2. Keyboard -> engine input state, then shortcuts -> commands.
        input_bridge::feed(ctx, &mut self.models.input);

        let mut commands: Vec<UiCommand> = Vec::new();

        if !ctx.wants_keyboard_input() {
            let actions: Vec<String> = triggered_actions(self.models.keybinds.data(), self.models.input.data())
                .into_iter()
                .map(str::to_string)
                .collect();

            for action in actions {
                match action_to_command(&action, &self.models) {
                    Some(command) => commands.push(command),
                    None => self.state.status = format!("\"{action}\" is not available yet"),
                }
            }
        }

        if self.models.number_field.data().phase == DragPhase::Dragging
            && ctx.input(|state| state.key_pressed(egui::Key::Escape))
        {
            commands.push(UiCommand::NumberDragCancel);
        }

        // 3. Draw. The views only read the models and push commands.
        menu_bar::show(ctx, &self.models, &mut commands);

        let dirty = self.scene_is_dirty();

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.weak(&self.state.status);

                if dirty {
                    ui.weak("   |   unsaved changes (Ctrl+S)");
                }
            });
        });

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            self.dock_view.show(ui, &self.models, &mut self.panel_views, &mut commands);
        });

        if self.close_prompt {
            self.show_close_prompt(ctx);
        }

        // 4. If no number field reported a hover this frame, the pointer left them all.
        if !commands.iter().any(|command| matches!(command, UiCommand::NumberHover)) {
            NumberFieldController.hover_exit(&mut self.models.number_field);
        }

        // 5. Apply the commands through the controllers.
        apply_all(commands, &mut self.models, &mut self.state);

        // 6. Saving the scene, prefab work that needs the disk, the cursor, and the window title.
        if self.state.save_requested {
            self.state.save_requested = false;
            self.save_scene();
        }

        let requests = std::mem::take(&mut self.state.pending_io);

        if !requests.is_empty() {
            let today = flux_core::clock::today();

            for request in requests {
                self.state.status = match prefab_flow::run(request, &mut self.models, &today) {
                    Ok(message) => message,
                    Err(error) => error.to_string(),
                };
            }
        }

        self.update_cursor(ctx);

        let title = self.window_title();

        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }

        // Settings are saved when changed, but not while a mouse button is held (dragging a divider).
        if self.state.settings_dirty && !ctx.input(|state| state.pointer.any_down()) {
            self.save_settings();
        }

        // 7. Clear the one-frame input values.
        InputController.end_frame(&mut self.models.input);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
}
