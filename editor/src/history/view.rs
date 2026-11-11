use super::model::HistoryModel;
use flux_engine::mivc::{Model, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryRowKind {
    /// A state before the current one. Click to undo back to it.
    Past,
    /// Where the scene is now.
    Current,
    /// A state that was undone. Click to redo up to it.
    Future,
}

/// One line of the history panel. The rows are the states the scene went through.
#[derive(Clone, Debug, PartialEq)]
pub struct HistoryRow {
    pub label: String,
    pub kind: HistoryRowKind,
    /// How many undo steps (Past) or redo steps (Future) it takes to reach this state.
    pub steps: usize,
}

/// Builds the rows of the history panel. It only reads.
#[derive(Default)]
pub struct HistoryView {
    rows: Vec<HistoryRow>,
}

impl HistoryView {
    pub fn rows(&self) -> &[HistoryRow] {
        &self.rows
    }
}

impl View for HistoryView {
    type Models<'a> = &'a HistoryModel;

    fn render(&mut self, model: Self::Models<'_>) {
        let undo = &model.data().undo;
        let redo = &model.data().redo;
        let count = undo.len();

        self.rows.clear();

        // The state before the first step.
        self.rows.push(HistoryRow {
            label: "Start of history".to_string(),
            kind: if count == 0 { HistoryRowKind::Current } else { HistoryRowKind::Past },
            steps: count,
        });

        // The state after each step. The last one is where the scene is now.
        for (index, entry) in undo.iter().enumerate() {
            let position = index + 1;

            self.rows.push(HistoryRow {
                label: entry.label.clone(),
                kind: if position == count { HistoryRowKind::Current } else { HistoryRowKind::Past },
                steps: count - position,
            });
        }

        // States that were undone, nearest first.
        for (index, entry) in redo.iter().rev().enumerate() {
            self.rows.push(HistoryRow { label: entry.label.clone(), kind: HistoryRowKind::Future, steps: index + 1 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryController;
    use flux_engine::identity::IdRegistryModel;
    use flux_engine::scene::SceneController;

    #[test]
    fn rows_go_from_the_start_through_now_to_the_undone_steps() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let mut history = HistoryModel::default();

        for label in ["First", "Second", "Third"] {
            let before = scene.data().clone();

            SceneController.create_object(&mut scene, &mut ids, label, None).unwrap();
            HistoryController.record(&mut history, label, None, before);
        }

        HistoryController.undo(&mut history, &mut scene);

        let mut view = HistoryView::default();
        view.render(&history);

        let rows: Vec<(&str, HistoryRowKind, usize)> =
            view.rows().iter().map(|row| (row.label.as_str(), row.kind, row.steps)).collect();

        assert_eq!(
            rows,
            vec![
                ("Start of history", HistoryRowKind::Past, 2),
                ("First", HistoryRowKind::Past, 1),
                ("Second", HistoryRowKind::Current, 0),
                ("Third", HistoryRowKind::Future, 1),
            ]
        );
    }

    #[test]
    fn an_empty_history_has_one_current_row() {
        let mut view = HistoryView::default();

        view.render(&HistoryModel::default());

        assert_eq!(view.rows().len(), 1);
        assert_eq!(view.rows()[0].kind, HistoryRowKind::Current);
    }
}
