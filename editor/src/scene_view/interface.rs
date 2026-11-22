use flux_engine::math::Vec3;
use flux_engine::mivc::Interface;
use flux_grp::frame::OrbitCamera;
use flux_grp::glam;
use serde::{Deserialize, Serialize};

/// Where the Scene view camera is: it looks at `target` from `distance` away.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneCameraData {
    pub target: Vec3,
    /// Turns the camera left / right.
    pub yaw_degrees: f32,
    /// Looks up / down (positive = looking down).
    pub pitch_degrees: f32,
    pub distance: f32,
}

impl SceneCameraData {
    pub fn to_orbit(&self) -> OrbitCamera {
        OrbitCamera {
            target: glam::Vec3::new(self.target.x, self.target.y, self.target.z),
            yaw_degrees: self.yaw_degrees,
            pitch_degrees: self.pitch_degrees,
            distance: self.distance,
        }
    }

    pub fn from_orbit(orbit: &OrbitCamera) -> Self {
        Self {
            target: Vec3::new(orbit.target.x, orbit.target.y, orbit.target.z),
            yaw_degrees: orbit.yaw_degrees,
            pitch_degrees: orbit.pitch_degrees,
            distance: orbit.distance,
        }
    }
}

impl Default for SceneCameraData {
    fn default() -> Self {
        Self::from_orbit(&OrbitCamera::default())
    }
}

impl Interface for SceneCameraData {
    const FORMAT: &'static str = "scene_camera";
    const VERSION: u32 = 1;
}
