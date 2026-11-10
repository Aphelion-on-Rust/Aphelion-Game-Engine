use super::controller::LifecycleController;
use super::interface::{Dispatch, LifecycleEvent};
use super::model::{LifecycleModel, MouseTrackerModel};
use super::mouse_controller::MouseEventController;
use crate::game_loop::{FrameContext, LoopCallbacks};
use crate::identity::AssetUuid;
use crate::mivc::Model;
use crate::scene::SceneModel;

/// Carries out a Dispatch: calls the real script method.
/// The C# host (step 9) implements this. Tests use a recorder.
pub trait ScriptHost {
    fn invoke(&mut self, dispatch: &Dispatch);
}

/// Plugs scripts into the game loop. Give it to `GameLoop::tick` as the callbacks.
///
/// Per frame it does:
///   fixed_update: sync, FixedUpdate
///   update:       sync, mouse events, Update, OnIK, LateUpdate
///   render:       OnPreRender
pub struct ScriptRunner<H: ScriptHost> {
    scene: SceneModel,
    lifecycle: LifecycleModel,
    mouse: MouseTrackerModel,
    host: H,
    hovered: Option<AssetUuid>,
}

impl<H: ScriptHost> ScriptRunner<H> {
    pub fn new(scene: SceneModel, host: H) -> Self {
        Self {
            scene,
            lifecycle: LifecycleModel::default(),
            mouse: MouseTrackerModel::default(),
            host,
            hovered: None,
        }
    }

    pub fn scene(&self) -> &SceneModel {
        &self.scene
    }

    /// Controllers use this to change the scene while the game runs.
    pub fn scene_mut(&mut self) -> &mut SceneModel {
        &mut self.scene
    }

    pub fn host(&self) -> &H {
        &self.host
    }

    /// The object under the cursor. Picking sets this every frame (physics step).
    pub fn set_hovered(&mut self, object: Option<AssetUuid>) {
        self.hovered = object;
    }

    /// Queue an event for one object (collisions, triggers, animation events ...).
    pub fn queue_event(&mut self, object: AssetUuid, event: LifecycleEvent) {
        LifecycleController.queue_object_event(&mut self.lifecycle, object, event);
    }

    /// Send an event to every running script (focus, pause, ...).
    pub fn broadcast(&mut self, event: LifecycleEvent) {
        let list = LifecycleController.broadcast(&self.lifecycle, &event);
        self.run(list);
    }

    /// Call when the game closes.
    pub fn shutdown(&mut self) {
        let list = LifecycleController.shutdown(&mut self.lifecycle);
        self.run(list);
    }

    fn run(&mut self, list: Vec<Dispatch>) {
        for item in &list {
            self.host.invoke(item);
        }
    }

    fn sync_scripts(&mut self) {
        let list = LifecycleController.sync(&mut self.lifecycle, &self.scene);
        self.run(list);
    }

    fn send_to_running(&mut self, event: LifecycleEvent) {
        let list = LifecycleController.dispatch_to_running(&self.lifecycle, &event);
        self.run(list);
    }
}

impl<H: ScriptHost> LoopCallbacks for ScriptRunner<H> {
    fn fixed_update(&mut self, _context: &FrameContext) {
        self.sync_scripts();
        self.send_to_running(LifecycleEvent::FixedUpdate);
    }

    fn update(&mut self, context: &FrameContext) {
        self.sync_scripts();

        MouseEventController.update(&mut self.mouse, &mut self.lifecycle, context.input.data(), self.hovered);
        let mouse_events = LifecycleController.flush_object_events(&mut self.lifecycle);
        self.run(mouse_events);

        self.send_to_running(LifecycleEvent::Update);
        self.send_to_running(LifecycleEvent::OnIK { layer: 0 });
        self.send_to_running(LifecycleEvent::LateUpdate);
    }

    fn render(&mut self, _context: &FrameContext) {
        self.send_to_running(LifecycleEvent::OnPreRender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_loop::GameLoop;
    use crate::gc::GcModel;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::input::{InputController, InputModel, MouseButton};
    use crate::scene::{ComponentData, SceneController, ScriptData};
    use crate::time::TimeModel;

    #[derive(Default)]
    struct Recorder {
        calls: Vec<&'static str>,
    }

    impl ScriptHost for Recorder {
        fn invoke(&mut self, dispatch: &Dispatch) {
            self.calls.push(dispatch.event.name());
        }
    }

    struct World {
        runner: ScriptRunner<Recorder>,
        object: AssetUuid,
        ids: IdRegistryModel,
        time: TimeModel,
        input: InputModel,
        gc: GcModel,
        game_loop: GameLoop,
    }

    impl World {
        fn new() -> Self {
            let mut ids = IdRegistryModel::default();
            let mut scene = SceneController.create_scene(&mut ids, "Test");
            let object = SceneController.create_object(&mut scene, &mut ids, "Player", None).unwrap();
            let script = IdController.new_uuid(&mut ids);
            let component = ComponentData::Script(ScriptData { script, enabled: true });

            SceneController.add_component(&mut scene, &mut ids, &object, component).unwrap();

            World {
                runner: ScriptRunner::new(scene, Recorder::default()),
                object,
                ids,
                time: TimeModel::default(),
                input: InputModel::default(),
                gc: GcModel::default(),
                game_loop: GameLoop::new(),
            }
        }

        /// Runs one frame of 0.02 seconds (enough for exactly one fixed step).
        fn frame(&mut self) {
            self.game_loop.tick_with_delta(0.02, &mut self.time, &mut self.input, &mut self.gc, &mut self.runner);
        }

        fn calls(&mut self) -> Vec<&'static str> {
            std::mem::take(&mut self.runner.host.calls)
        }
    }

    #[test]
    fn first_frame_runs_the_whole_lifecycle_in_order() {
        let mut world = World::new();
        world.frame();

        assert_eq!(
            world.calls(),
            vec![
                "Awake", "OnEnable", "Start", "FixedUpdate", "Update", "OnIK", "LateUpdate", "OnPreRender"
            ]
        );
    }

    #[test]
    fn later_frames_only_run_the_frame_events() {
        let mut world = World::new();
        world.frame();
        world.calls();

        world.frame();

        assert_eq!(world.calls(), vec!["FixedUpdate", "Update", "OnIK", "LateUpdate", "OnPreRender"]);
    }

    #[test]
    fn a_switched_off_object_stops_getting_updates() {
        let mut world = World::new();
        world.frame();
        world.calls();

        let object = world.object;
        SceneController.set_active(world.runner.scene_mut(), &object, false).unwrap();
        world.frame();

        assert_eq!(world.calls(), vec!["OnDisable"]);
    }

    #[test]
    fn destroying_the_object_ends_with_on_destroy() {
        let mut world = World::new();
        world.frame();
        world.calls();

        let object = world.object;
        SceneController.destroy_object(world.runner.scene_mut(), &object).unwrap();
        world.frame();

        assert_eq!(world.calls(), vec!["OnDisable", "OnDestroy"]);
    }

    #[test]
    fn a_mouse_press_on_the_hovered_object_reaches_the_script() {
        let mut world = World::new();
        world.frame();
        world.calls();

        world.runner.set_hovered(Some(world.object));
        InputController.mouse_button_event(&mut world.input, MouseButton::Left, true);
        world.frame();

        let calls = world.calls();

        assert!(calls.contains(&"OnMouseEnter"));
        assert!(calls.contains(&"OnMouseDown"));
    }

    #[test]
    fn shutdown_sends_quit_disable_destroy() {
        let mut world = World::new();
        world.frame();
        world.calls();

        world.runner.shutdown();

        assert_eq!(world.calls(), vec!["OnApplicationQuit", "OnDisable", "OnDestroy"]);
        let _ = &world.ids;
    }
}
