use super::interface::{InstallEvent, InstallJob, JobPhase};
use super::model::JobsModel;
use super::query;
use crate::error::{LauncherError, LauncherResult};
use flux_core::mivc::{Controller, Model};

pub struct JobsController;

impl Controller for JobsController {}

impl JobsController {
    /// Starts tracking an install. Refused if that version is already being installed.
    pub fn start(&self, model: &mut JobsModel, version: &str) -> LauncherResult<()> {
        if query::is_active(model.data(), version) {
            return Err(LauncherError::Invalid(format!("version {version} is already being installed")));
        }

        let data = self.write(model);

        data.jobs.retain(|job| job.version != version);
        data.jobs.push(InstallJob {
            version: version.to_string(),
            phase: JobPhase::Downloading,
            downloaded_bytes: 0,
            total_bytes: 0,
            message: String::new(),
        });
        Ok(())
    }

    /// Applies what the background thread reported. Events for unknown versions are ignored.
    pub fn apply(&self, model: &mut JobsModel, event: &InstallEvent) {
        let data = self.write(model);
        let Some(job) = data.jobs.iter_mut().find(|job| job.version == event.version()) else {
            return;
        };

        match event {
            InstallEvent::Progress { downloaded, total, .. } => {
                job.phase = JobPhase::Downloading;
                job.downloaded_bytes = *downloaded;
                job.total_bytes = *total;
            }
            InstallEvent::Phase { phase, .. } => job.phase = *phase,
            InstallEvent::Failed { message, .. } => {
                job.phase = JobPhase::Failed;
                job.message = message.clone();
            }
            InstallEvent::Finished { .. } => job.phase = JobPhase::Finished,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_version_can_not_be_installed_twice_at_once() {
        let mut model = JobsModel::default();

        JobsController.start(&mut model, "0.7.0").unwrap();

        assert!(JobsController.start(&mut model, "0.7.0").is_err());
        assert!(JobsController.start(&mut model, "0.8.0").is_ok());
    }

    #[test]
    fn progress_moves_the_bar_and_finishing_ends_the_job() {
        let mut model = JobsModel::default();
        JobsController.start(&mut model, "0.7.0").unwrap();

        JobsController.apply(&mut model, &InstallEvent::Progress { version: "0.7.0".to_string(), downloaded: 50, total: 200 });
        assert_eq!(query::fraction(query::find(model.data(), "0.7.0").unwrap()), 0.25);
        assert!(query::any_active(model.data()));

        JobsController.apply(&mut model, &InstallEvent::Finished { version: "0.7.0".to_string(), folder: String::new() });
        assert!(!query::is_active(model.data(), "0.7.0"));
    }

    #[test]
    fn a_failed_job_can_be_started_again() {
        let mut model = JobsModel::default();
        JobsController.start(&mut model, "0.7.0").unwrap();

        JobsController.apply(&mut model, &InstallEvent::Failed { version: "0.7.0".to_string(), message: "no network".to_string() });

        assert_eq!(query::find(model.data(), "0.7.0").unwrap().message, "no network");
        assert!(JobsController.start(&mut model, "0.7.0").is_ok());
    }

    #[test]
    fn events_for_unknown_versions_are_ignored() {
        let mut model = JobsModel::default();

        JobsController.apply(&mut model, &InstallEvent::Finished { version: "9.9.9".to_string(), folder: String::new() });

        assert!(model.data().jobs.is_empty());
    }
}
