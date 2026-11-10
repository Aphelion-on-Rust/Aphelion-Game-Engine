use super::interface::{GraphicsPipeline, ProjectData};
use super::model::ProjectModel;
use crate::error::{EngineError, EngineResult};
use crate::mivc::{Controller, Store};

pub struct ProjectController;

impl Controller for ProjectController {}

fn invalid(message: &str) -> EngineError {
    EngineError::InvalidOperation(message.to_string())
}

/// A project or scene name is also a folder or file name, so it has to be safe to use as one.
/// Returns the trimmed name.
pub fn validate_name(name: &str) -> EngineResult<String> {
    let name = name.trim();

    if name.is_empty() {
        return Err(invalid("the name can't be empty"));
    }

    if name.chars().count() > 64 {
        return Err(invalid("the name can be at most 64 characters"));
    }

    if name.chars().all(|letter| letter == '.') {
        return Err(invalid("the name can't be only dots"));
    }

    if name.chars().any(|letter| letter.is_control() || "\\/:*?\"<>|".contains(letter)) {
        return Err(invalid("the name can't contain  \\ / : * ? \" < > |"));
    }

    Ok(name.to_string())
}

impl ProjectController {
    /// Makes the data of a new project. The pipeline and Aphelion's Control are decided here,
    /// there is deliberately no way to change them afterwards.
    pub fn new_project(
        &self,
        name: &str,
        engine_version: &str,
        pipeline: GraphicsPipeline,
        aphelion_control: bool,
        today: &str,
    ) -> EngineResult<ProjectModel> {
        if engine_version.trim().is_empty() {
            return Err(invalid("a project needs an engine version"));
        }

        Ok(Store::new(ProjectData {
            name: validate_name(name)?,
            engine_version: engine_version.trim().to_string(),
            pipeline,
            aphelion_control,
            created_on: today.to_string(),
        }))
    }

    pub fn set_engine_version(&self, model: &mut ProjectModel, version: &str) -> EngineResult<()> {
        if version.trim().is_empty() {
            return Err(invalid("a project needs an engine version"));
        }

        self.write(model).engine_version = version.trim().to_string();
        Ok(())
    }

    pub fn rename(&self, model: &mut ProjectModel, name: &str) -> EngineResult<()> {
        let name = validate_name(name)?;

        self.write(model).name = name;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mivc::Model;

    #[test]
    fn names_must_be_safe_folder_names() {
        assert_eq!(validate_name("  My Game ").unwrap(), "My Game");

        for bad in ["", "   ", "..", "a/b", "a\\b", "what?", "a:b", "x|y", "tab\there"] {
            assert!(validate_name(bad).is_err(), "{bad:?} should be refused");
        }

        assert!(validate_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn a_new_project_keeps_what_was_chosen() {
        let model = ProjectController
            .new_project("Space Game", "0.8.0", GraphicsPipeline::Heavy, true, "2026-11-10")
            .unwrap();

        assert_eq!(model.data().name, "Space Game");
        assert_eq!(model.data().pipeline, GraphicsPipeline::Heavy);
        assert!(model.data().aphelion_control);
    }

    #[test]
    fn a_project_needs_an_engine_version() {
        assert!(ProjectController.new_project("A", "  ", GraphicsPipeline::Simple, false, "").is_err());
    }

    #[test]
    fn the_engine_version_and_name_can_change() {
        let mut model = ProjectController.new_project("A", "0.8.0", GraphicsPipeline::Simple, false, "").unwrap();

        ProjectController.set_engine_version(&mut model, "0.9.0").unwrap();
        ProjectController.rename(&mut model, "B").unwrap();

        assert_eq!(model.data().engine_version, "0.9.0");
        assert_eq!(model.data().name, "B");
        assert!(ProjectController.rename(&mut model, "a/b").is_err());
    }
}
