pub mod installs;
pub mod projects;
pub mod settings;

/// 12.3 MB
pub fn format_size(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}
