use super::interface::{Dispatch, InstanceState, LifecycleEvent, PendingEvent};
use super::model::LifecycleModel;
use crate::identity::{AssetUuid, ComponentId};
use crate::mivc::{Controller, Model};
use crate::scene::{query, ComponentData, GameObjectData, SceneModel};

/// Decides which script component gets which event.
pub struct LifecycleController;

impl Controller for LifecycleController {}

/// A script component as it is in the scene right now.
struct Wanted {
    component: ComponentId,
    object: AssetUuid,
    script: AssetUuid,
    object_active: bool,
    script_enabled: bool,
}

fn gather_scripts(objects: &[GameObjectData]) -> Vec<Wanted> {
    let mut list = Vec::new();

    for object in objects {
        for entry in &object.components {
            if let ComponentData::Script(script) = &entry.data {
                list.push(Wanted {
                    component: entry.id,
                    object: object.uuid,
                    script: script.script,
                    object_active: query::is_active_in_hierarchy(objects, &object.uuid),
                    script_enabled: script.enabled,
                });
            }
        }
    }

    list
}

fn dispatch(state: &InstanceState, event: LifecycleEvent) -> Dispatch {
    Dispatch { object: state.object, component: state.component, script: state.script, event }
}

impl LifecycleController {
    /// Compares the scene with what we know and returns the events that follow:
    /// OnDisable/OnDestroy for scripts that went away, Awake/OnEnable for new or
    /// re-enabled ones, OnDisable for switched-off ones, and Start for scripts that
    /// are running but have not started yet.
    ///
    /// Call it at the start of every frame. If nothing changed it returns nothing.
    pub fn sync(&self, model: &mut LifecycleModel, scene: &SceneModel) -> Vec<Dispatch> {
        let wanted = gather_scripts(&scene.data().objects);
        let data = self.write(model);
        let mut out = Vec::new();

        // 1. Scripts whose component or object no longer exists.
        let mut remaining = Vec::new();

        for state in data.instances.drain(..) {
            if wanted.iter().any(|entry| entry.component == state.component) {
                remaining.push(state);
                continue;
            }

            if state.running {
                out.push(dispatch(&state, LifecycleEvent::OnDisable));
            }

            if state.awoken {
                out.push(dispatch(&state, LifecycleEvent::OnDestroy));
            }
        }

        data.instances = remaining;
        data.pending_events.retain(|pending| wanted.iter().any(|entry| entry.object == pending.object));

        // 2. New scripts, and scripts that were switched on or off.
        for entry in &wanted {
            let index = match data.instances.iter().position(|state| state.component == entry.component) {
                Some(index) => index,
                None => {
                    data.instances.push(InstanceState {
                        component: entry.component,
                        object: entry.object,
                        script: entry.script,
                        awoken: false,
                        running: false,
                        started: false,
                    });
                    data.instances.len() - 1
                }
            };

            let state = &mut data.instances[index];

            if entry.object_active && !state.awoken {
                state.awoken = true;
                out.push(dispatch(state, LifecycleEvent::Awake));
            }

            let should_run = entry.object_active && entry.script_enabled;

            if should_run && !state.running {
                state.running = true;
                out.push(dispatch(state, LifecycleEvent::OnEnable));
            } else if !should_run && state.running {
                state.running = false;
                out.push(dispatch(state, LifecycleEvent::OnDisable));
            }
        }

        // 3. Start comes after every Awake and OnEnable above.
        for state in data.instances.iter_mut() {
            if state.running && !state.started {
                state.started = true;
                out.push(dispatch(state, LifecycleEvent::Start));
            }
        }

        out
    }

    /// For events every running script gets: Update, FixedUpdate, OnIK, LateUpdate ...
    /// Only scripts that already had Start called are included.
    pub fn dispatch_to_running(&self, model: &LifecycleModel, event: &LifecycleEvent) -> Vec<Dispatch> {
        model
            .data()
            .instances
            .iter()
            .filter(|state| state.running && state.started)
            .map(|state| dispatch(state, event.clone()))
            .collect()
    }

    /// For application events (focus, pause, quit): every running script gets it.
    pub fn broadcast(&self, model: &LifecycleModel, event: &LifecycleEvent) -> Vec<Dispatch> {
        model
            .data()
            .instances
            .iter()
            .filter(|state| state.running)
            .map(|state| dispatch(state, event.clone()))
            .collect()
    }

    /// Queues an event for the scripts on one object (mouse, collisions, ...).
    /// It is delivered by the next `flush_object_events`.
    pub fn queue_object_event(&self, model: &mut LifecycleModel, object: AssetUuid, event: LifecycleEvent) {
        self.write(model).pending_events.push(PendingEvent { object, event });
    }

    /// Delivers every queued event to the running scripts on its object.
    /// Events for objects with no running script are dropped.
    pub fn flush_object_events(&self, model: &mut LifecycleModel) -> Vec<Dispatch> {
        let data = self.write(model);
        let pending = std::mem::take(&mut data.pending_events);
        let mut out = Vec::new();

        for item in pending {
            for state in data.instances.iter().filter(|s| s.running && s.object == item.object) {
                out.push(dispatch(state, item.event.clone()));
            }
        }

        out
    }

    /// The game is closing: OnApplicationQuit, then OnDisable and OnDestroy for everything.
    pub fn shutdown(&self, model: &mut LifecycleModel) -> Vec<Dispatch> {
        let mut out = self.broadcast(model, &LifecycleEvent::OnApplicationQuit);
        let data = self.write(model);

        for state in data.instances.drain(..) {
            if state.running {
                out.push(dispatch(&state, LifecycleEvent::OnDisable));
            }

            if state.awoken {
                out.push(dispatch(&state, LifecycleEvent::OnDestroy));
            }
        }

        data.pending_events.clear();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::scene::{SceneController, ScriptData};

    fn names(list: &[Dispatch]) -> Vec<&'static str> {
        list.iter().map(|item| item.event.name()).collect()
    }

    /// A scene with one object that has one script on it.
    fn scene_with_script(enabled: bool) -> (SceneModel, IdRegistryModel, AssetUuid) {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let object = SceneController.create_object(&mut scene, &mut ids, "Player", None).unwrap();
        let script = IdController.new_uuid(&mut ids);

        SceneController
            .add_component(&mut scene, &mut ids, &object, ComponentData::Script(ScriptData::new(script, enabled)))
            .unwrap();

        (scene, ids, object)
    }

    #[test]
    fn new_script_gets_awake_then_enable_then_start() {
        let (scene, _ids, _object) = scene_with_script(true);
        let mut model = LifecycleModel::default();

        let events = LifecycleController.sync(&mut model, &scene);

        assert_eq!(names(&events), vec!["Awake", "OnEnable", "Start"]);
        assert!(LifecycleController.sync(&mut model, &scene).is_empty());
    }

    #[test]
    fn disabled_script_is_awake_but_never_enabled_or_started() {
        let (scene, _ids, _object) = scene_with_script(false);
        let mut model = LifecycleModel::default();

        assert_eq!(names(&LifecycleController.sync(&mut model, &scene)), vec!["Awake"]);
        assert!(LifecycleController.dispatch_to_running(&model, &LifecycleEvent::Update).is_empty());
    }

    #[test]
    fn switching_the_object_off_and_on_again() {
        let (mut scene, _ids, object) = scene_with_script(true);
        let mut model = LifecycleModel::default();
        LifecycleController.sync(&mut model, &scene);

        SceneController.set_active(&mut scene, &object, false).unwrap();
        assert_eq!(names(&LifecycleController.sync(&mut model, &scene)), vec!["OnDisable"]);

        SceneController.set_active(&mut scene, &object, true).unwrap();
        // Awake and Start are only called once. OnEnable is called again.
        assert_eq!(names(&LifecycleController.sync(&mut model, &scene)), vec!["OnEnable"]);
    }

    #[test]
    fn destroying_the_object_calls_disable_then_destroy() {
        let (mut scene, _ids, object) = scene_with_script(true);
        let mut model = LifecycleModel::default();
        LifecycleController.sync(&mut model, &scene);

        SceneController.destroy_object(&mut scene, &object).unwrap();

        assert_eq!(names(&LifecycleController.sync(&mut model, &scene)), vec!["OnDisable", "OnDestroy"]);
    }

    #[test]
    fn queued_events_only_reach_running_scripts_on_that_object() {
        let (scene, _ids, object) = scene_with_script(true);
        let mut model = LifecycleModel::default();
        LifecycleController.sync(&mut model, &scene);

        LifecycleController.queue_object_event(&mut model, object, LifecycleEvent::OnMouseEnter);
        let events = LifecycleController.flush_object_events(&mut model);

        assert_eq!(names(&events), vec!["OnMouseEnter"]);
        assert!(LifecycleController.flush_object_events(&mut model).is_empty());
    }

    #[test]
    fn shutdown_quits_disables_and_destroys() {
        let (scene, _ids, _object) = scene_with_script(true);
        let mut model = LifecycleModel::default();
        LifecycleController.sync(&mut model, &scene);

        let events = LifecycleController.shutdown(&mut model);

        assert_eq!(names(&events), vec!["OnApplicationQuit", "OnDisable", "OnDestroy"]);
        assert!(model.data().instances.is_empty());
    }
}
