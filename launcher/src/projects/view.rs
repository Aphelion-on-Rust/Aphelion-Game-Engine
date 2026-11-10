use super::model::ProjectsModel;
use super::query;
use crate::installs::{query as installs_query, InstallsModel};
use flux_core::mivc::{Model, View};

/// One line of the projects list.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectRow {
    pub name: String,
    pub path: String,
    pub engine_version: String,
    pub last_opened: String,
    /// False means "Open" will download and install the version first.
    pub engine_installed: bool,
}

#[derive(Default)]
pub struct ProjectsView {
    rows: Vec<ProjectRow>,
}

impl ProjectsView {
    pub fn rows(&self) -> &[ProjectRow] {
        &self.rows
    }
}

impl View for ProjectsView {
    type Models<'a> = (&'a ProjectsModel, &'a InstallsModel);

    fn render(&mut self, models: Self::Models<'_>) {
        let (projects, installs) = models;

        self.rows = query::most_recent_first(projects.data())
            .into_iter()
            .map(|project| ProjectRow {
                name: project.name.clone(),
                path: project.path.clone(),
                engine_version: project.engine_version.clone(),
                last_opened: project.last_opened.clone(),
                engine_installed: installs_query::is_installed(installs.data(), &project.engine_version),
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installs::{InstalledVersion, InstallsController};
    use crate::projects::{ProjectEntry, ProjectsController};

    #[test]
    fn rows_know_if_the_engine_version_is_installed() {
        let mut projects = ProjectsModel::default();
        let mut installs = InstallsModel::default();

        for (name, version) in [("A", "0.7.0"), ("B", "0.8.0")] {
            ProjectsController
                .add(
                    &mut projects,
                    ProjectEntry {
                        name: name.to_string(),
                        path: format!("D:/{name}"),
                        engine_version: version.to_string(),
                        last_opened: String::new(),
                    },
                )
                .unwrap();
        }

        InstallsController.register(
            &mut installs,
            InstalledVersion { version: "0.7.0".to_string(), folder: String::new(), installed_on: String::new() },
        );

        let mut view = ProjectsView::default();
        view.render((&projects, &installs));

        let installed: Vec<(String, bool)> = view.rows().iter().map(|r| (r.name.clone(), r.engine_installed)).collect();

        assert!(installed.contains(&("A".to_string(), true)));
        assert!(installed.contains(&("B".to_string(), false)));
    }
}
