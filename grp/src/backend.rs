use crate::frame::RenderFrame;
use crate::software::Image;

/// Something that can draw a `RenderFrame` into a picture.
/// The software renderer is one. The GPU renderer (step 7d) will be another.
pub trait RenderBackend {
    fn render(&mut self, frame: &RenderFrame, width: usize, height: usize) -> Image;
}
