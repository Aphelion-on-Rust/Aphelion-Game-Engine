//! Reading a color out of a picture at a point (u, v).
//!
//! (0, 0) is the bottom left corner of the picture and (1, 1) the top right. Outside that range the
//! picture repeats. A picture with `nearest` set gives hard pixels, otherwise the four nearest pixels
//! are blended.

use crate::frame::TextureImage;
use glam::Vec2;

/// One pixel as red, green, blue, alpha from 0 to 1. Coordinates outside the picture wrap around.
fn texel(texture: &TextureImage, x: i64, y: i64) -> [f32; 4] {
    let x = x.rem_euclid(texture.width as i64) as usize;
    let y = y.rem_euclid(texture.height as i64) as usize;
    let start = (y * texture.width + x) * 4;

    [
        texture.rgba[start] as f32 / 255.0,
        texture.rgba[start + 1] as f32 / 255.0,
        texture.rgba[start + 2] as f32 / 255.0,
        texture.rgba[start + 3] as f32 / 255.0,
    ]
}

/// The color of a texture at (u, v).
pub fn sample(texture: &TextureImage, uv: Vec2) -> [f32; 4] {
    if texture.width == 0 || texture.height == 0 || texture.rgba.len() < texture.width * texture.height * 4 {
        return [1.0, 1.0, 1.0, 1.0];
    }

    // Row 0 of the picture is the top, so v is turned upside down.
    let x = uv.x * texture.width as f32;
    let y = (1.0 - uv.y) * texture.height as f32;

    if texture.nearest {
        return texel(texture, x.floor() as i64, y.floor() as i64);
    }

    // Blend the four pixels around the point. Pixel centers are half a pixel in.
    let (x, y) = (x - 0.5, y - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (top_left, top_right) = (texel(texture, x0, y0), texel(texture, x0 + 1, y0));
    let (bottom_left, bottom_right) = (texel(texture, x0, y0 + 1), texel(texture, x0 + 1, y0 + 1));
    let mut color = [0.0; 4];

    for channel in 0..4 {
        let top = top_left[channel] * (1.0 - tx) + top_right[channel] * tx;
        let bottom = bottom_left[channel] * (1.0 - tx) + bottom_right[channel] * tx;

        color[channel] = top * (1.0 - ty) + bottom * ty;
    }

    color
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2 x 2: red and green on the top row, blue and white below.
    fn checker(nearest: bool) -> TextureImage {
        TextureImage {
            width: 2,
            height: 2,
            rgba: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255],
            nearest,
        }
    }

    fn close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn nearest_gives_hard_pixels_with_the_top_left_at_the_top_left() {
        let texture = checker(true);

        assert!(close(sample(&texture, Vec2::new(0.25, 0.75)), [1.0, 0.0, 0.0, 1.0])); // top left
        assert!(close(sample(&texture, Vec2::new(0.75, 0.75)), [0.0, 1.0, 0.0, 1.0])); // top right
        assert!(close(sample(&texture, Vec2::new(0.25, 0.25)), [0.0, 0.0, 1.0, 1.0])); // bottom left
        assert!(close(sample(&texture, Vec2::new(0.75, 0.25)), [1.0, 1.0, 1.0, 1.0])); // bottom right
    }

    #[test]
    fn the_picture_repeats_outside_zero_to_one() {
        let texture = checker(true);

        assert_eq!(sample(&texture, Vec2::new(1.25, 0.75)), sample(&texture, Vec2::new(0.25, 0.75)));
        assert_eq!(sample(&texture, Vec2::new(-0.75, 2.75)), sample(&texture, Vec2::new(0.25, 0.75)));
    }

    #[test]
    fn linear_gives_the_pixel_in_the_middle_of_a_pixel_and_a_blend_between_two() {
        let texture = checker(false);

        assert!(close(sample(&texture, Vec2::new(0.25, 0.75)), [1.0, 0.0, 0.0, 1.0]));

        // Exactly between the red and the green pixel of the top row.
        assert!(close(sample(&texture, Vec2::new(0.5, 0.75)), [0.5, 0.5, 0.0, 1.0]));
    }

    #[test]
    fn a_broken_picture_gives_white_instead_of_a_crash() {
        let broken = TextureImage { width: 4, height: 4, rgba: vec![0; 3], nearest: true };
        let empty = TextureImage { width: 0, height: 0, rgba: Vec::new(), nearest: false };

        assert_eq!(sample(&broken, Vec2::new(0.5, 0.5)), [1.0; 4]);
        assert_eq!(sample(&empty, Vec2::new(0.5, 0.5)), [1.0; 4]);
    }
}
