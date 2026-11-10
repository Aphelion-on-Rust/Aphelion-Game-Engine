use super::interface::{InstallJob, JobPhase, JobsData};

pub fn find<'a>(data: &'a JobsData, version: &str) -> Option<&'a InstallJob> {
    data.jobs.iter().find(|job| job.version == version)
}

/// Downloading, verifying or extracting right now.
pub fn is_active(data: &JobsData, version: &str) -> bool {
    find(data, version).map_or(false, |job| {
        matches!(job.phase, JobPhase::Downloading | JobPhase::Verifying | JobPhase::Extracting)
    })
}

pub fn any_active(data: &JobsData) -> bool {
    data.jobs.iter().any(|job| is_active(data, &job.version))
}

/// 0.0 to 1.0 for a progress bar.
pub fn fraction(job: &InstallJob) -> f32 {
    match job.phase {
        JobPhase::Downloading if job.total_bytes > 0 => (job.downloaded_bytes as f32 / job.total_bytes as f32).min(1.0),
        JobPhase::Verifying | JobPhase::Extracting | JobPhase::Finished => 1.0,
        _ => 0.0,
    }
}
