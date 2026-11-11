use super::interface::HistoryData;

pub fn can_undo(data: &HistoryData) -> bool {
    !data.undo.is_empty()
}

pub fn can_redo(data: &HistoryData) -> bool {
    !data.redo.is_empty()
}

pub fn next_undo_label(data: &HistoryData) -> Option<&str> {
    data.undo.last().map(|entry| entry.label.as_str())
}

pub fn next_redo_label(data: &HistoryData) -> Option<&str> {
    data.redo.last().map(|entry| entry.label.as_str())
}
