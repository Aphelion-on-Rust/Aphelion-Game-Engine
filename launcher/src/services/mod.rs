//! Everything that touches the outside world: the network, zip files, folders, other programs.
//! The MIVC folders never do this, so their logic can be tested without any of it.

pub mod checksum;
pub mod clock;
pub mod download;
pub mod extract;
pub mod github;
pub mod install;
pub mod launch;
pub mod paths;

use crate::jobs::InstallEvent;

/// What background threads send back to the window.
#[derive(Debug)]
pub enum BackgroundEvent {
    /// The raw release list from GitHub (or an error text).
    ReleasesFetched(Result<String, String>),
    Install(InstallEvent),
}
