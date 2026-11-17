//! The lighting of a lit surface: a little ambient light plus every light in the scene.

use crate::frame::RenderLight;
use flux_engine::components::LightKind;
use glam::Vec3;

/// Light that is everywhere, so the side facing away from the lights is not pure black.
pub const AMBIENT: f32 = 0.15;

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}

/// How much one light shines on a surface point `point` with normal `normal` (red, green, blue).
fn light_on(light: &RenderLight, point: Vec3, normal: Vec3) -> Vec3 {
    let (to_light, falloff) = match light.kind {
        LightKind::Directional => (-light.direction, 1.0),
        LightKind::Point | LightKind::Spot => {
            let offset = light.position - point;
            let distance = offset.length();

            if distance < 1e-6 || light.range <= 0.0 || distance > light.range {
                return Vec3::ZERO;
            }

            let to_light = offset / distance;
            let reach = (1.0 - distance / light.range).powi(2);
            let cone = if light.kind == LightKind::Spot {
                let half = (light.spot_angle_degrees * 0.5).to_radians();
                let cos_outer = half.cos();
                let cos_inner = (half * 0.8).cos();

                smoothstep(cos_outer, cos_inner, light.direction.dot(-to_light))
            } else {
                1.0
            };

            (to_light, reach * cone)
        }
    };

    let facing = normal.dot(to_light).max(0.0);

    Vec3::from(light.color) * light.intensity * facing * falloff
}

/// The color of a lit surface: its own color times (ambient + all the lights), kept between 0 and 1.
pub fn lit_color(base: [f32; 4], point: Vec3, normal: Vec3, lights: &[RenderLight]) -> [f32; 3] {
    let mut total = Vec3::splat(AMBIENT);

    for light in lights {
        total += light_on(light, point, normal);
    }

    let color = Vec3::new(base[0], base[1], base[2]) * total;

    [color.x.clamp(0.0, 1.0), color.y.clamp(0.0, 1.0), color.z.clamp(0.0, 1.0)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_engine::identity::{IdController, IdRegistryModel};

    fn light(kind: LightKind, position: Vec3, direction: Vec3) -> RenderLight {
        RenderLight {
            object: IdController.new_uuid(&mut IdRegistryModel::default()),
            kind,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            position,
            direction,
            range: 10.0,
            spot_angle_degrees: 40.0,
            shadows: false,
        }
    }

    fn red(value: [f32; 3]) -> f32 {
        value[0]
    }

    #[test]
    fn with_no_lights_only_the_ambient_light_shows() {
        let color = lit_color([1.0, 0.5, 0.0, 1.0], Vec3::ZERO, Vec3::Y, &[]);

        assert!((color[0] - AMBIENT).abs() < 1e-6);
        assert!((color[1] - AMBIENT * 0.5).abs() < 1e-6);
        assert_eq!(color[2], 0.0);
    }

    #[test]
    fn a_directional_light_lights_surfaces_that_face_it_and_not_the_others() {
        let sun = [light(LightKind::Directional, Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0))];
        let facing = lit_color([0.5, 0.5, 0.5, 1.0], Vec3::ZERO, Vec3::Y, &sun);
        let away = lit_color([0.5, 0.5, 0.5, 1.0], Vec3::ZERO, Vec3::NEG_Y, &sun);

        assert!((red(facing) - 0.5 * (AMBIENT + 1.0)).abs() < 1e-5);
        assert!((red(away) - 0.5 * AMBIENT).abs() < 1e-5);
    }

    #[test]
    fn a_point_light_gets_weaker_with_distance_and_stops_at_its_range() {
        let lamp = [light(LightKind::Point, Vec3::new(0.0, 2.0, 0.0), Vec3::Z)];
        let near = lit_color([1.0; 4], Vec3::new(0.0, 1.0, 0.0), Vec3::Y, &lamp);
        let far = lit_color([1.0; 4], Vec3::new(0.0, -6.0, 0.0), Vec3::Y, &lamp);
        let out_of_reach = lit_color([1.0; 4], Vec3::new(0.0, -20.0, 0.0), Vec3::Y, &lamp);

        assert!(red(near) > red(far));
        assert!(red(far) > AMBIENT);
        assert!((red(out_of_reach) - AMBIENT).abs() < 1e-6);
    }

    #[test]
    fn a_spot_light_lights_inside_its_cone_and_not_outside() {
        let spot = [light(LightKind::Spot, Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0))];
        let inside = lit_color([1.0; 4], Vec3::ZERO, Vec3::Y, &spot);
        let outside = lit_color([1.0; 4], Vec3::new(4.0, 0.0, 0.0), Vec3::Y, &spot);

        assert!(red(inside) > AMBIENT + 0.1);
        assert!((red(outside) - AMBIENT).abs() < 1e-5);
    }

    #[test]
    fn light_colors_tint_the_surface_and_the_result_never_goes_above_one() {
        let mut sun = light(LightKind::Directional, Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0));

        sun.color = [1.0, 0.0, 0.0];
        sun.intensity = 5.0;

        let color = lit_color([1.0; 4], Vec3::ZERO, Vec3::Y, &[sun]);

        assert_eq!(color[0], 1.0);
        assert!((color[1] - AMBIENT).abs() < 1e-6);
    }
}
