use anyhow::Result;
use asyncgit::sync::{
    RepoPath, ShowUntrackedFilesConfig,
    status::{StatusType, get_status},
};
use std::{
    fs::{File, create_dir},
    io::Write,
    path::{Path, PathBuf},
};

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
    pub repo: PathBuf,

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
    pub fn open(path: Option<&str>) -> Result<()> {
        let repo_path = RepoPath::Path(".".into());
        let status_type = StatusType::Both;
        let show_untracked = ShowUntrackedFilesConfig::default();

        let status =
            get_status(&repo_path, status_type, Some(show_untracked));

        println!("{:#?}", status);

        Ok(())
    }

    /// helper func to get branch name
    pub fn get_branch_name(&self) -> Result<Option<String>> {
        todo!()
    }
}

pub struct GaiGit {}
impl GaiGit {
    pub(crate) fn get_file_diffs_as_str(
        &self,
    ) -> std::collections::HashMap<String, String> {
        todo!()
    }

    pub(crate) fn get_repo_status_as_str(&self) -> &str {
        todo!()
    }

    pub(crate) fn get_repo_tree(&self) -> &str {
        todo!()
    }

    pub(crate) fn new(
        only_staged: bool,
        stage_hunks: bool,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> Self {
        Self {}
    }

    pub(crate) fn get_logs(
        &self,
        number: Option<usize>,
        reverse: bool,
    ) -> Result<()> {
        Ok(())
    }

    pub(crate) fn apply_commits(
        &self,
        commits: &[super::commit::GaiCommit],
    ) -> Result<()> {
        Ok(())
    }
}
