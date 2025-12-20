use anyhow::Result;
use git2::Repository;
use std::path::PathBuf;

use super::errors::GitError;

// honestly not sure
// why i named the struct
// gaigit
//
// wrapper for git2rs repo
// ideally, what gets passed
// around instead of a humongous
// struct
pub struct GitRepo {
    /// git2 based Repo
    pub repo: Repository,

    /// current workdir path
    /// will error on bare
    /// intentional methinks
    /// idt we should handle ANY operation
    /// for bare repos
    pub workdir: PathBuf,
}

impl GitRepo {
    /// attempt to open repo,
    /// if no path specified it'll
    /// walk up FROM the CURRENT dir
    /// to find a valid repo, otherwise
    /// search from the path supplied
    pub fn open(path: Option<&str>) -> Result<Self> {
        let repo = if let Some(p) = path {
            Repository::discover(p)?
        } else {
            Repository::discover(".")?
        };

        let workdir =
            repo.workdir().ok_or(GitError::BareRepo)?.to_path_buf();

        Ok(Self { repo, workdir })
    }
}
