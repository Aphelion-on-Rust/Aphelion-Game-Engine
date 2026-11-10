use super::callbacks::{FrameContext, FrameReport, LoopCallbacks};
use crate::gc::{GcController, GcModel};
use crate::input::{InputController, InputModel};
use crate::mivc::Controller;
use crate::time::{TimeController, TimeModel};
use std::time::Instant;

pub struct GameLoop {
    last_tick: Option<Instant>,
}

impl Controller for GameLoop {}

impl GameLoop {
    pub fn new() -> Self {
        Self { last_tick: None }
    }

    /// Runs one frame, measuring the real time since the last frame.
    pub fn tick(
        &mut self,
        time: &mut TimeModel,
        input: &mut InputModel,
        gc: &mut GcModel,
        game: &mut dyn LoopCallbacks,
    ) -> FrameReport {
        let now = Instant::now();
        let real_delta = self
            .last_tick
            .map_or(0.0, |last| now.duration_since(last).as_secs_f64());

        self.last_tick = Some(now);
        self.tick_with_delta(real_delta, time, input, gc, game)
    }

    /// Runs one frame with a time we choose. Tests and replays use this.
    pub fn tick_with_delta(
        &self,
        real_delta_seconds: f64,
        time: &mut TimeModel,
        input: &mut InputModel,
        gc: &mut GcModel,
        game: &mut dyn LoopCallbacks,
    ) -> FrameReport {
        TimeController.begin_frame(time, real_delta_seconds);
        GcController.begin_frame(gc);

        let mut fixed_steps = 0;

        while TimeController.take_fixed_step(time) {
            game.fixed_update(&FrameContext { time: &*time, input: &*input });
            fixed_steps += 1;
        }

        game.update(&FrameContext { time: &*time, input: &*input });
        game.render(&FrameContext { time: &*time, input: &*input });

        let collected = GcController.collect(gc);

        if !collected.is_empty() {
            game.on_resources_collected(&collected);
        }

        InputController.end_frame(input);

        FrameReport { fixed_steps, collected_resources: collected.len() as u32 }
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gc::ResourceKind;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::input::KeyCode;

    /// Remembers the order things were called in.
    #[derive(Default)]
    struct Recorder {
        calls: Vec<&'static str>,
        saw_space_pressed: bool,
        freed: usize,
    }

    impl LoopCallbacks for Recorder {
        fn fixed_update(&mut self, _context: &FrameContext) {
            self.calls.push("fixed");
        }

        fn update(&mut self, context: &FrameContext) {
            self.calls.push("update");
            self.saw_space_pressed = context.input.data().keys_pressed.contains(&KeyCode::Space);
        }

        fn render(&mut self, _context: &FrameContext) {
            self.calls.push("render");
        }

        fn on_resources_collected(&mut self, collected: &[crate::gc::CollectedResource]) {
            self.freed += collected.len();
        }
    }

    use crate::mivc::Model;

    #[test]
    fn frame_runs_fixed_then_update_then_render() {
        let mut time = TimeModel::default();
        let mut input = InputModel::default();
        let mut gc = GcModel::default();
        let mut game = Recorder::default();

        TimeController.set_fixed_step(&mut time, 0.01).unwrap();

        let report = GameLoop::new().tick_with_delta(0.035, &mut time, &mut input, &mut gc, &mut game);

        assert_eq!(report.fixed_steps, 3);
        assert_eq!(game.calls, vec!["fixed", "fixed", "fixed", "update", "render"]);
    }

    #[test]
    fn update_sees_input_and_then_input_is_cleared() {
        let mut time = TimeModel::default();
        let mut input = InputModel::default();
        let mut gc = GcModel::default();
        let mut game = Recorder::default();

        InputController.key_event(&mut input, KeyCode::Space, true);
        GameLoop::new().tick_with_delta(0.016, &mut time, &mut input, &mut gc, &mut game);

        assert!(game.saw_space_pressed);
        assert!(input.data().keys_pressed.is_empty());
        assert!(input.data().keys_down.contains(&KeyCode::Space));
    }

    #[test]
    fn game_is_told_when_resources_are_freed() {
        let mut time = TimeModel::default();
        let mut input = InputModel::default();
        let mut gc = GcModel::default();
        let mut game = Recorder::default();

        GcController.set_grace_frames(&mut gc, 2);
        let uuid = IdController.new_uuid(&mut IdRegistryModel::default());
        GcController.register(&mut gc, uuid, ResourceKind::Texture);

        let game_loop = GameLoop::new();
        game_loop.tick_with_delta(0.016, &mut time, &mut input, &mut gc, &mut game);
        let report = game_loop.tick_with_delta(0.016, &mut time, &mut input, &mut gc, &mut game);

        assert_eq!(report.collected_resources, 1);
        assert_eq!(game.freed, 1);
    }
}
