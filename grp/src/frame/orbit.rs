//! The editor's Scene view camera: it circles a target point, like holding the scene in your hand.
//! Yaw turns it left / right, pitch looks up / down (positive = looking down), distance is how far
//! it stands from the target. Same coordinates as everything else: left-handed, Y up, Z forward.

use super::extract::projection_matrix;
use super::interface::RenderCamera;
use flux_engine::components::ViewCameraData;
use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self { target: Vec3::new(0.0, 0.5, 0.0), yaw_degrees: 0.0, pitch_degrees: 15.0, distance: 9.0 }
    }
}

impl OrbitCamera {
    /// The direction the camera looks.
    pub fn forward(&self) -> Vec3 {
        let (yaw, pitch) = (self.yaw_degrees.to_radians(), self.pitch_degrees.to_radians());

        Vec3::new(yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos())
    }

    pub fn right(&self) -> Vec3 {
        Vec3::Y.cross(self.forward()).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.forward().cross(self.right())
    }

    /// Where the camera stands.
    pub fn eye(&self) -> Vec3 {
        self.target - self.forward() * self.distance
    }

    pub fn render_camera(&self, aspect: f32) -> RenderCamera {
        let lens = ViewCameraData::default();

        RenderCamera {
            object: None,
            view: Mat4::look_to_lh(self.eye(), self.forward(), Vec3::Y),
            projection: projection_matrix(&lens, aspect),
            position: self.eye(),
            clear_color: [lens.clear_color.r, lens.clear_color.g, lens.clear_color.b, lens.clear_color.a],
            near: lens.near,
            far: lens.far,
        }
    }

    /// Drag with the mouse (in pixels): the scene turns with the mouse.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw_degrees = (self.yaw_degrees + dx * 0.3) % 360.0;
        self.pitch_degrees = (self.pitch_degrees + dy * 0.3).clamp(-89.0, 89.0);
    }

    /// Drag with the mouse (in pixels): the scene slides with the mouse.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let step = self.distance * 0.0015;

        self.target += -self.right() * dx * step + self.up() * dy * step;
    }

    /// Scroll up zooms in, scroll down zooms out. The distance stays between 0.5 and 500.
    pub fn zoom(&mut self, scroll: f32) {
        self.distance = (self.distance * (-scroll * 0.002).exp()).clamp(0.5, 500.0);
    }

    /// Look at a point from a distance.
    pub fn focus(&mut self, point: Vec3, distance: f32) {
        self.target = point;
        self.distance = distance.clamp(0.5, 500.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn looking_straight_ahead_the_camera_stands_behind_its_target() {
        let camera = OrbitCamera { target: Vec3::new(1.0, 2.0, 3.0), yaw_degrees: 0.0, pitch_degrees: 0.0, distance: 4.0 };

        assert!(close(camera.forward(), Vec3::Z));
        assert!(close(camera.right(), Vec3::X));
        assert!(close(camera.up(), Vec3::Y));
        assert!(close(camera.eye(), Vec3::new(1.0, 2.0, -1.0)));
    }

    #[test]
    fn the_camera_always_looks_at_the_middle_of_the_picture() {
        for (yaw, pitch) in [(0.0, 15.0), (70.0, -30.0), (200.0, 60.0)] {
            let camera = OrbitCamera { target: Vec3::new(2.0, 1.0, -3.0), yaw_degrees: yaw, pitch_degrees: pitch, distance: 7.0 };
            let render = camera.render_camera(1.5);
            let ndc = (render.projection * render.view).project_point3(camera.target);

            assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4, "yaw {yaw}, pitch {pitch}");
            assert!(ndc.z > 0.0 && ndc.z < 1.0);
        }
    }

    #[test]
    fn dragging_right_turns_the_camera_to_the_right_and_dragging_down_looks_more_down() {
        let mut camera = OrbitCamera::default();
        let (yaw, pitch) = (camera.yaw_degrees, camera.pitch_degrees);

        camera.orbit(10.0, 10.0);

        assert!(camera.yaw_degrees > yaw);
        assert!(camera.pitch_degrees > pitch);
    }

    #[test]
    fn the_pitch_never_goes_past_straight_up_or_down() {
        let mut camera = OrbitCamera::default();

        camera.orbit(0.0, 100_000.0);
        assert_eq!(camera.pitch_degrees, 89.0);

        camera.orbit(0.0, -1_000_000.0);
        assert_eq!(camera.pitch_degrees, -89.0);
    }

    #[test]
    fn panning_slides_the_target_with_the_mouse() {
        let mut camera = OrbitCamera { target: Vec3::ZERO, yaw_degrees: 0.0, pitch_degrees: 0.0, distance: 10.0 };

        camera.pan(100.0, 0.0); // drag right: the scene moves right, so the camera moves left
        assert!(camera.target.x < 0.0 && camera.target.y.abs() < 1e-6);

        camera.pan(0.0, 100.0); // drag down: the scene moves down, so the camera moves up
        assert!(camera.target.y > 0.0);
    }

    #[test]
    fn zooming_changes_the_distance_and_stays_inside_its_limits() {
        let mut camera = OrbitCamera::default();
        let start = camera.distance;

        camera.zoom(100.0);
        assert!(camera.distance < start);

        camera.zoom(-100_000.0);
        assert_eq!(camera.distance, 500.0);

        camera.zoom(1_000_000.0);
        assert_eq!(camera.distance, 0.5);
    }

    #[test]
    fn focusing_moves_the_target_and_sets_the_distance() {
        let mut camera = OrbitCamera::default();

        camera.focus(Vec3::new(5.0, 0.0, 5.0), 3.0);

        assert_eq!((camera.target, camera.distance), (Vec3::new(5.0, 0.0, 5.0), 3.0));
    }
}
