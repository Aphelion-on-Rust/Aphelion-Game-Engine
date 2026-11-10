use super::interface::ProjectEntry;
use super::model::ProjectsModel;
use super::query;
use crate::error::{LauncherError, LauncherResult};
use flux_core::mivc::{Controller, Model};

pub struct ProjectsController;

impl Controller for ProjectsController {}

fn not_found(path: &str) -> LauncherError {
    LauncherError::NotFound(format!("no project at {path}"))
}

impl ProjectsController {
    pub fn add(&self, model: &mut ProjectsModel, entry: ProjectEntry) -> LauncherResult<()> {
        if entry.name.trim().is_empty() || entry.path.trim().is_empty() {
            return Err(LauncherError::Invalid("a project needs a name and a folder".to_string()));
        }

        if query::find(model.data(), &entry.path).is_some() {
            return Err(LauncherError::Invalid("that project is already in the list".to_string()));
        }

        self.write(model).projects.push(entry);
        Ok(())
    }

    pub fn remove(&self, model: &mut ProjectsModel, path: &str) -> LauncherResult<()> {
        if query::find(model.data(), path).is_none() {
            return Err(not_found(path));
        }

        self.write(model).projects.retain(|project| project.path != path);
        Ok(())
    }

    pub fn set_engine_version(&self, model: &mut ProjectsModel, path: &str, version: &str) -> LauncherResult<()> {
        let project = self
            .write(model)
            .projects
            .iter_mut()
            .find(|project| project.path == path)
            .ok_or_else(|| not_found(path))?;

        project.engine_version = version.to_string();
        Ok(())
    }

    /// Remembers when a project was opened.
    pub fn touch(&self, model: &mut ProjectsModel, path: &str, today: &str) -> LauncherResult<()> {
        let project = self
            .write(model)
            .projects
            .iter_mut()
            .find(|project| project.path == path)
            .ok_or_else(|| not_found(path))?;

        project.last_opened = today.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, path: &str) -> ProjectEntry {
        ProjectEntry {
            name: name.to_string(),
            path: path.to_string(),
            engine_version: "0.7.0".to_string(),
            last_opened: String::new(),
        }
    }

    #[test]
    fn a_project_can_only_be_added_once() {
        let mut model = ProjectsModel::default();

        assert!(ProjectsController.add(&mut model, entry("A", "D:/a")).is_ok());
        assert!(ProjectsController.add(&mut model, entry("A again", "D:/a")).is_err());
        assert!(ProjectsController.add(&mut model, entry("", "D:/b")).is_err());
    }

    #[test]
    fn the_most_recent_project_comes_first() {
        let mut model = ProjectsModel::default();

        ProjectsController.add(&mut model, entry("Old", "D:/old")).unwrap();
        ProjectsController.add(&mut model, entry("New", "D:/new")).unwrap();
        ProjectsController.touch(&mut model, "D:/old", "2026-11-01").unwrap();
        ProjectsController.touch(&mut model, "D:/new", "2026-11-09").unwrap();

        let names: Vec<&str> = query::most_recent_first(model.data()).iter().map(|p| p.name.as_str()).collect();

        assert_eq!(names, vec!["New", "Old"]);
    }

    #[test]
    fn the_engine_version_of_a_project_can_change() {
        let mut model = ProjectsModel::default();

        ProjectsController.add(&mut model, entry("A", "D:/a")).unwrap();
        ProjectsController.set_engine_version(&mut model, "D:/a", "0.8.0").unwrap();

        assert_eq!(query::find(model.data(), "D:/a").unwrap().engine_version, "0.8.0");
        assert!(ProjectsController.set_engine_version(&mut model, "D:/missing", "0.8.0").is_err());
    }
}
