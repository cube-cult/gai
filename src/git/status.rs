use std::{fmt, path::Path};

use git2::{Delta, Repository, Status, StatusOptions, StatusShow};

/// status strategy when running
/// get_status
#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize,
)]
pub enum StatusStrategy {
    /// only get status
    /// of working dir
    WorkingDir,
    /// only get status
    /// of what's currently staged
    Stage,
    /// both, this does not differentiate between
    /// the two, meaning wt and index are shown
    /// as one status
    #[default]
    Both,
}

#[derive(Debug, Default)]
pub struct GitStatus {
    pub statuses: Vec<FileStatus>,
}

#[derive(Debug)]
pub struct FileStatus {
    pub path: String,
    pub status: StatusItemType,
}

#[derive(strum::Display, Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum StatusItemType {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
}

// opts.show
impl From<StatusStrategy> for StatusShow {
    fn from(s: StatusStrategy) -> Self {
        match s {
            StatusStrategy::WorkingDir => Self::Workdir,
            StatusStrategy::Stage => Self::Index,
            StatusStrategy::Both => Self::IndexAndWorkdir,
        }
    }
}

impl From<Status> for StatusItemType {
    fn from(s: Status) -> Self {
        if s.is_index_new() || s.is_wt_new() {
            Self::New
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            Self::Deleted
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            Self::Renamed
        } else if s.is_index_typechange() || s.is_wt_typechange() {
            Self::Typechange
        } else if s.is_conflicted() {
            Self::Conflicted
        } else {
            Self::Modified
        }
    }
}

impl From<Delta> for StatusItemType {
    fn from(d: Delta) -> Self {
        match d {
            Delta::Added => Self::New,
            Delta::Deleted => Self::Deleted,
            Delta::Renamed => Self::Renamed,
            Delta::Typechange => Self::Typechange,
            _ => Self::Modified,
        }
    }
}

// helper ONLY FOR LLM REQUESTS
// not for pretty print status
impl fmt::Display for GitStatus {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut s = String::new();

        for git_status in &self.statuses {
            s.push_str(&format!(
                "{}:{}",
                git_status.status, git_status.path
            ));
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

pub fn is_workdir_clean(repo: &Repository) -> anyhow::Result<bool> {
    if repo.is_bare() && !repo.is_worktree() {
        return Ok(true);
    }

    let mut options = StatusOptions::default();
    options
        .show(StatusShow::Workdir)
        .update_index(true)
        .include_untracked(true)
        .renames_head_to_index(true)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut options))?;

    Ok(statuses.is_empty())
}

pub fn get_status(
    repo: &Repository,
    strategy: StatusStrategy,
) -> anyhow::Result<GitStatus> {
    let mut opts = StatusOptions::default();

    // filter
    opts.show(strategy.into());

    opts.update_index(true);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    opts.renames_head_to_index(true);
    opts.renames_index_to_workdir(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut statuses: Vec<FileStatus> = statuses
        .iter()
        .filter_map(|entry| {
            Some(FileStatus {
                path: entry.path()?.to_string(),
                status: entry.status().into(),
            })
        })
        .collect();

    statuses
        .sort_by(|a, b| Path::new(&a.path).cmp(Path::new(&b.path)));

    Ok(GitStatus { statuses })
}
