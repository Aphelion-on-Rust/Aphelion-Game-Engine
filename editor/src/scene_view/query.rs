use super::interface::SceneCameraData;
use flux_grp::frame::RenderCamera;

/// The camera to draw the Scene view with.
pub fn render_camera(data: &SceneCameraData, aspect: f32) -> RenderCamera {
    data.to_orbit().render_camera(aspect)
}
