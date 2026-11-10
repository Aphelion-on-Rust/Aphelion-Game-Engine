use super::interface::{ProjectEntry, ProjectsData};

pub fn find<'a>(data: &'a ProjectsData, path: &str) -> Option<&'a ProjectEntry> {
    data.projects.iter().find(|project| project.path == path)
}

/// Most recently opened first. Projects never opened come last.
pub fn most_recent_first(data: &ProjectsData) -> Vec<&ProjectEntry> {
    let mut list: Vec<&ProjectEntry> = data.projects.iter().collect();

    list.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    list
}
