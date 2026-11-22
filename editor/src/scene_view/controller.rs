use super::interface::SceneCameraData;
use super::model::SceneCameraModel;
use flux_engine::mivc::{Controller, Model};
use flux_grp::frame::OrbitCamera;

/// Moves the Scene view camera.
pub struct SceneCameraController;

impl Controller for SceneCameraController {}

impl SceneCameraController {
    fn change(&self, model: &mut SceneCameraModel, change: impl FnOnce(&mut OrbitCamera)) {
        let mut orbit = model.data().to_orbit();

        change(&mut orbit);
        *self.write(model) = SceneCameraData::from_orbit(&orbit);
    }

    /// Drag with the right mouse button (pixels).
    pub fn orbit(&self, model: &mut SceneCameraModel, dx: f32, dy: f32) {
        self.change(model, |orbit| orbit.orbit(dx, dy));
    }

    /// Drag with the middle mouse button (pixels).
    pub fn pan(&self, model: &mut SceneCameraModel, dx: f32, dy: f32) {
        self.change(model, |orbit| orbit.pan(dx, dy));
    }

    /// The mouse wheel. Up zooms in.
    pub fn zoom(&self, model: &mut SceneCameraModel, scroll: f32) {
        self.change(model, |orbit| orbit.zoom(scroll));
    }

    /// Look at a point from `distance` away (the point is the position of the selected object).
    pub fn focus(&self, model: &mut SceneCameraModel, point: [f32; 3], distance: f32) {
        self.change(model, |orbit| orbit.focus(flux_grp::glam::Vec3::from(point), distance));
    }

    pub fn reset(&self, model: &mut SceneCameraModel) {
        *self.write(model) = SceneCameraData::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_view::query::render_camera;

    #[test]
    fn orbiting_panning_and_zooming_change_the_camera_and_reset_brings_it_back() {
        let mut model = SceneCameraModel::default();

        SceneCameraController.orbit(&mut model, 50.0, 20.0);
        SceneCameraController.pan(&mut model, 30.0, 10.0);
        SceneCameraController.zoom(&mut model, 100.0);

        assert_ne!(model.data(), &SceneCameraData::default());
        assert!(model.data().distance < SceneCameraData::default().distance);

        SceneCameraController.reset(&mut model);
        assert_eq!(model.data(), &SceneCameraData::default());
    }

    #[test]
    fn focusing_puts_the_point_in_the_middle_of_the_picture() {
        let mut model = SceneCameraModel::default();

        SceneCameraController.focus(&mut model, [3.0, 1.0, -2.0], 4.0);

        let camera = render_camera(model.data(), 1.6);
        let ndc = (camera.projection * camera.view).project_point3(flux_grp::glam::Vec3::new(3.0, 1.0, -2.0));

        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4);
        assert_eq!(model.data().distance, 4.0);
    }

    #[test]
    fn the_camera_data_survives_yaml() {
        use flux_engine::storage::yaml::{from_yaml, to_yaml};

        let mut model = SceneCameraModel::default();

        SceneCameraController.orbit(&mut model, 12.0, -8.0);

        let loaded: SceneCameraData = from_yaml(&to_yaml(model.data()).unwrap()).unwrap();

        assert_eq!(&loaded, model.data());
    }
}
