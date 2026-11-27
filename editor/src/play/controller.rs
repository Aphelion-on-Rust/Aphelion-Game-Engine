use super::interface::{PlayState, FIXED_STEP, MAX_STEPS_PER_FRAME};
use super::model::PlayModel;
use flux_engine::math::Vec3 as SceneVec3;
use flux_engine::mivc::{Controller, Model};
use flux_engine::physics::{PhysicsEvent, PhysicsEventKind, PhysicsSettings, PhysicsWorld};
use flux_engine::scene::transform::world_matrix;
use flux_engine::scene::{query, ComponentData, SceneController, SceneData, SceneModel};
use flux_grp::glam::Mat4;

pub struct PlayController;

impl Controller for PlayController {}

impl PlayController {
    /// Saves a copy of the scene and builds the physics world from it. The app keeps the world until Stop.
    pub fn start(&self, play: &mut PlayModel, scene: &SceneModel) -> PhysicsWorld {
        let data = self.write(play);

        data.state = PlayState::Playing;
        data.snapshot = Some(scene.data().clone());
        data.elapsed = 0.0;
        data.accumulator = 0.0;
        data.last_event.clear();

        PhysicsWorld::from_scene(scene.data(), PhysicsSettings::default())
    }

    pub fn toggle_pause(&self, play: &mut PlayModel) {
        let data = self.write(play);

        data.state = match data.state {
            PlayState::Playing => PlayState::Paused,
            PlayState::Paused => PlayState::Playing,
            PlayState::Stopped => PlayState::Stopped,
        };
    }

    /// Puts the scene back exactly as it was when Play was pressed.
    pub fn stop(&self, play: &mut PlayModel, scene: &mut SceneModel) {
        let snapshot = self.write(play).snapshot.take();

        if let Some(saved) = snapshot {
            *self.write(scene) = saved;
        }

        let data = self.write(play);

        data.state = PlayState::Stopped;
        data.elapsed = 0.0;
        data.accumulator = 0.0;
        data.last_event.clear();
    }

    /// Moves time forward by `dt` seconds (in fixed steps) and puts the new positions into the scene.
    /// Does nothing unless playing. A very long frame is cut short (see `MAX_STEPS_PER_FRAME`).
    pub fn advance(&self, play: &mut PlayModel, scene: &mut SceneModel, world: &mut PhysicsWorld, dt: f32) -> Vec<PhysicsEvent> {
        if play.data().state != PlayState::Playing {
            return Vec::new();
        }

        self.write(play).accumulator += dt.clamp(0.0, 0.1);

        let mut steps = 0;
        let mut events = Vec::new();

        while play.data().accumulator >= FIXED_STEP && steps < MAX_STEPS_PER_FRAME {
            events.extend(world.step(FIXED_STEP));

            let data = self.write(play);

            data.accumulator -= FIXED_STEP;
            data.elapsed += FIXED_STEP;
            steps += 1;
        }

        // Behind by more than we can catch up: forget the rest instead of falling further behind.
        if play.data().accumulator >= FIXED_STEP {
            self.write(play).accumulator = 0.0;
        }

        if steps > 0 {
            self.apply_positions(scene, world);
        }

        if let Some(last) = events.last() {
            let text = describe(scene.data(), last);

            self.write(play).last_event = text;
        }

        events
    }

    /// Writes where the bodies are into the Transforms. The world gives positions in the world, a Transform
    /// wants a position inside its parent.
    fn apply_positions(&self, scene: &mut SceneModel, world: &PhysicsWorld) {
        for (object, position) in world.positions() {
            let objects = &scene.data().objects;
            let Some(found) = query::find_object(objects, &object) else {
                continue;
            };
            let Some(transform) = found.components.iter().find(|entry| matches!(entry.data, ComponentData::Transform(_))) else {
                continue;
            };

            let component = transform.id;
            let parent = found.parent.and_then(|parent| world_matrix(objects, &parent)).unwrap_or(Mat4::IDENTITY);
            let local = parent.inverse().transform_point3(position);
            let unchanged = match &transform.data {
                ComponentData::Transform(current) => {
                    (current.position.x - local.x).abs() < 1e-6 && (current.position.y - local.y).abs() < 1e-6 && (current.position.z - local.z).abs() < 1e-6
                }
                _ => true,
            };

            if unchanged {
                continue;
            }

            let _ = SceneController.update_component(scene, &object, component, |data| {
                if let ComponentData::Transform(transform) = data {
                    transform.position = SceneVec3::new(local.x, local.y, local.z);
                }
            });
        }
    }
}

/// One line about an event, with the names of the objects.
fn describe(scene: &SceneData, event: &PhysicsEvent) -> String {
    let name = |object| query::find_object(&scene.objects, &object).map_or("?".to_string(), |found| found.name.clone());
    let what = match event.kind {
        PhysicsEventKind::CollisionEnter => "Hit",
        PhysicsEventKind::CollisionExit => "Apart",
        PhysicsEventKind::TriggerEnter => "Trigger enter",
        PhysicsEventKind::TriggerExit => "Trigger exit",
    };

    format!("{what}: {} / {}", name(event.a), name(event.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::interface::PlayData;
    use crate::play::query::{in_play_mode, is_running, label};
    use flux_engine::identity::{AssetUuid, IdRegistryModel};
    use flux_engine::scene::ObjectPreset;

    fn scene_with_physics() -> (SceneModel, IdRegistryModel, AssetUuid) {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");

        SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::Floor, None).unwrap();

        let cube = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();

        (scene, ids, cube)
    }

    fn height(scene: &SceneModel, object: &AssetUuid) -> f32 {
        let found = query::find_object(&scene.data().objects, object).unwrap();

        match &found.components[0].data {
            ComponentData::Transform(transform) => transform.position.y,
            other => panic!("expected a transform, got {other:?}"),
        }
    }

    fn play_for(controller: &PlayController, play: &mut PlayModel, scene: &mut SceneModel, world: &mut PhysicsWorld, seconds: f32) {
        for _ in 0..(seconds * 60.0) as usize {
            controller.advance(play, scene, world, 1.0 / 60.0);
        }
    }

    #[test]
    fn the_cube_falls_in_the_scene_while_playing_and_stop_puts_everything_back() {
        let (mut scene, _ids, cube) = scene_with_physics();
        let before = scene.data().clone();
        let mut play = PlayModel::default();
        let mut world = PlayController.start(&mut play, &scene);

        assert!(is_running(play.data()) && in_play_mode(play.data()));

        play_for(&PlayController, &mut play, &mut scene, &mut world, 3.0);

        // It fell from 5 to rest on the floor (half the cube above y = 0).
        assert!((height(&scene, &cube) - 0.5).abs() < 0.05, "rests at {}", height(&scene, &cube));
        assert!(play.data().elapsed > 2.9);

        PlayController.stop(&mut play, &mut scene);

        assert_eq!(scene.data(), &before);
        assert!(!in_play_mode(play.data()));
        assert_eq!(play.data(), &PlayData::default());
    }

    #[test]
    fn a_paused_game_does_not_move_and_resuming_continues() {
        let (mut scene, _ids, cube) = scene_with_physics();
        let mut play = PlayModel::default();
        let mut world = PlayController.start(&mut play, &scene);

        play_for(&PlayController, &mut play, &mut scene, &mut world, 0.5);

        let paused_at = height(&scene, &cube);

        PlayController.toggle_pause(&mut play);
        assert_eq!(label(play.data()), "Paused at 0.5 s");
        play_for(&PlayController, &mut play, &mut scene, &mut world, 0.5);
        assert_eq!(height(&scene, &cube), paused_at);

        PlayController.toggle_pause(&mut play);
        play_for(&PlayController, &mut play, &mut scene, &mut world, 0.5);
        assert!(height(&scene, &cube) < paused_at);
    }

    #[test]
    fn a_long_frame_is_cut_short_instead_of_freezing_the_game() {
        let (mut scene, _ids, _cube) = scene_with_physics();
        let mut play = PlayModel::default();
        let mut world = PlayController.start(&mut play, &scene);

        PlayController.advance(&mut play, &mut scene, &mut world, 5.0); // a frame that lasted 5 seconds

        // At most 5 steps of 1/60 s were made, and the time that could not be caught up is forgotten.
        assert!((play.data().elapsed - 5.0 / 60.0).abs() < 1e-4, "elapsed {}", play.data().elapsed);
        assert_eq!(play.data().accumulator, 0.0);
    }

    #[test]
    fn nothing_moves_unless_playing() {
        let (mut scene, _ids, cube) = scene_with_physics();
        let mut play = PlayModel::default();
        let mut world = PhysicsWorld::from_scene(scene.data(), PhysicsSettings::default());
        let start = height(&scene, &cube);

        play_for(&PlayController, &mut play, &mut scene, &mut world, 1.0);

        assert_eq!(height(&scene, &cube), start);
        assert_eq!(label(play.data()), "Stopped");
    }

    #[test]
    fn a_child_of_a_moved_parent_keeps_its_position_inside_the_parent() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");

        SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::Floor, None).unwrap();

        let parent = SceneController.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        let component = query::find_object(&scene.data().objects, &parent).unwrap().components[0].id;

        SceneController
            .update_component(&mut scene, &parent, component, |data| {
                if let ComponentData::Transform(transform) = data {
                    transform.position = SceneVec3::new(3.0, 0.0, 2.0);
                }
            })
            .unwrap();

        let child = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, Some(parent)).unwrap();
        let mut play = PlayModel::default();
        let mut world = PlayController.start(&mut play, &scene);

        play_for(&PlayController, &mut play, &mut scene, &mut world, 3.0);

        let found = query::find_object(&scene.data().objects, &child).unwrap();

        match &found.components[0].data {
            ComponentData::Transform(transform) => {
                // It fell straight down inside its parent: x and z are unchanged, y is the height above the parent (at y = 0).
                assert!(transform.position.x.abs() < 0.01 && transform.position.z.abs() < 0.01, "{:?}", transform.position);
                assert!((transform.position.y - 0.5).abs() < 0.05);
            }
            other => panic!("expected a transform, got {other:?}"),
        }
    }

    #[test]
    fn events_are_described_with_the_names_of_the_objects() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let ball = SceneController.create_object(&mut scene, &mut ids, "Ball", None).unwrap();
        let zone = SceneController.create_object(&mut scene, &mut ids, "Zone", None).unwrap();
        let text = describe(scene.data(), &PhysicsEvent { kind: PhysicsEventKind::TriggerEnter, a: ball, b: zone });

        assert_eq!(text, "Trigger enter: Ball / Zone");
    }
}
