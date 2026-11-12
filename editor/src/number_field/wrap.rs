//! Keeps the hidden pointer inside the window while a number is dragged.
//!
//! The pointer is invisible while dragging, so when it reaches the left or right edge of the
//! window it is moved to the other side and the drag simply goes on. That one move shows up as
//! a huge jump of the pointer, which must not change the value.

/// If the pointer is at the left or right edge, the x it should be moved to (the other side).
pub fn wrapped_x(x: f32, width: f32, margin: f32) -> Option<f32> {
    if width <= margin * 4.0 {
        return None;
    }

    if x <= margin {
        Some(width - margin * 2.0)
    } else if x >= width - margin {
        Some(margin * 2.0)
    } else {
        None
    }
}

/// A move of more than half the window in one frame is the wrap, not the user moving the mouse.
pub fn is_jump(delta_x: f32, width: f32) -> bool {
    delta_x.abs() > width * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pointer_jumps_to_the_other_side_at_an_edge() {
        assert_eq!(wrapped_x(2.0, 1000.0, 4.0), Some(992.0));
        assert_eq!(wrapped_x(998.0, 1000.0, 4.0), Some(8.0));
        assert_eq!(wrapped_x(500.0, 1000.0, 4.0), None);
    }

    #[test]
    fn a_tiny_window_is_left_alone() {
        assert_eq!(wrapped_x(1.0, 10.0, 4.0), None);
    }

    #[test]
    fn only_a_huge_move_counts_as_the_wrap_jump() {
        assert!(is_jump(900.0, 1000.0));
        assert!(is_jump(-900.0, 1000.0));
        assert!(!is_jump(40.0, 1000.0));
    }
}
