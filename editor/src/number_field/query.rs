use super::interface::{CursorIcon, DragPhase, NumberFieldData};

pub fn cursor_icon(data: &NumberFieldData) -> CursorIcon {
    match data.phase {
        DragPhase::Idle => CursorIcon::Default,
        DragPhase::Hovering => CursorIcon::HorizontalResize,
        DragPhase::Dragging => CursorIcon::Hidden,
    }
}

pub fn is_pointer_hidden(data: &NumberFieldData) -> bool {
    data.phase == DragPhase::Dragging
}
