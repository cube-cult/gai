use serde::{Deserialize, Serialize};

/// for different types
/// of adding/staging per commit
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum StagingStrategy {
    /// as individual hunks
    Hunks,

    /// only stage one file PER commit
    OneFilePerCommit,

    /// small group of files
    /// files SHOULD represent
    /// a DISTINCT change per commit
    #[default]
    AtomicFiles,

    /// stage all changes together
    /// as monolithic commit
    /// best to enable allow_body
    /// so the LLM can generate
    /// a descriptive commit description
    AllFilesOneCommit,
}

// populated after
// loading config
// but can be modified
// dont think passing around
// config is needed for this case
/// diffing strategy
#[derive(Default)]
pub struct DiffStrategy {
    /// send the diffs with the
    /// staged files ONLy
    pub staged_only: bool,

    /// files to truncate
    /// will show as
    /// "TRUNCATED FILE"
    /// ideally this could be set
    /// automatically
    pub truncated_files: Vec<String>,

    /// files to ignore separate
    /// from .gitignore
    pub ignored_files: Vec<String>,
}

/// status strategy when running
/// get_status
#[derive(Default)]
pub enum StatusStrategy {
    /// only get status
    /// of working dir
    WorkingDir,
    /// only get status
    /// of what's currently staged
    Stage,
    /// both
    #[default]
    Both,
}
