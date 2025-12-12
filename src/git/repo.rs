use anyhow::Result;
use git2::Repository;
use std::{collections::HashMap, path::PathBuf};
use walkdir::WalkDir;

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
    pub fn open(path: Option<&str>) -> Result<GitRepo> {
        let repo = if let Some(p) = path {
            Repository::discover(p)?
        } else {
            Repository::discover(".")?
        };

        let workdir =
            repo.workdir().ok_or(GitError::BareRepo)?.to_path_buf();

        Ok(Self { repo, workdir })
    }

    /// helper func to get branch name
    pub fn get_branch(&self) -> Result<Option<String>> {
        match self.repo.head() {
            Ok(head) => Ok(head.shorthand().map(String::from)),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct GaiGit {
    /// Diffs
    pub files: Vec<GaiFile>,
    /// ideally this can be configured as well for global
    /// ignores that are not part of .gitignore.
    pub ignored: Vec<String>,

    /// git2 based Repo
    pub repo: Repository,

    pub status: GaiStatus,

    pub only_staged: bool,
    pub stage_hunks: bool,
    pub capitalize_prefix: bool,
    pub include_scope: bool,
}

/// helper to store paths for the files
/// marked along status
pub struct GaiStatus {
    pub s_new: Vec<String>,
    pub s_modified: Vec<String>,
    pub s_deleted: Vec<String>,
    // old -> new
    pub s_renamed: Vec<(String, String)>,

    // unstaged
    pub u_new: Vec<String>,
    pub u_modified: Vec<String>,
    pub u_deleted: Vec<String>,
    pub u_renamed: Vec<(String, String)>,
}

/// a sort of DiffDelta struct
#[derive(Debug, Clone)]
pub struct GaiFile {
    pub path: String,
    pub should_truncate: bool,
    pub hunks: Vec<HunkDiff>,
}

#[derive(Debug, Clone)]
pub struct HunkDiff {
    /// example key (header)
    /// @@ -12,8 +12,9 @@
    pub header: String,

    pub line_diffs: Vec<LineDiff>,
}

#[derive(Debug, Clone)]
pub struct LineDiff {
    pub diff_type: DiffType,
    pub content: String,
}

/// taken from diffline::origin
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
pub enum DiffType {
    #[default]
    Unchanged,
    Additions,
    Deletions,
}

impl GaiGit {
    /// todo: this could fail on an unitialized directory
    /// for now, im not gonna handle those and we
    /// just straight up panic if we failed to open
    pub fn new(
        only_staged: bool,
        stage_hunks: bool,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> Result<Self> {
        let repo = Repository::open_from_env()?;
        let status = Self::build_status(&repo)?;

        Ok(GaiGit {
            repo,
            files: Vec::new(),
            ignored: Vec::new(),
            status,
            only_staged,
            stage_hunks,
            capitalize_prefix,
            include_scope,
        })
    }

    pub fn get_branch(&self) -> String {
        let head = match self.repo.head() {
            Ok(h) => Some(h),
            Err(e) => return format!("bad branch {e}"),
        };

        let head = head.as_ref().and_then(|h| h.shorthand());

        head.unwrap_or("HEAD").to_string()
    }

    pub fn get_repo_tree(&self) -> String {
        let repo_root =
            self.repo.workdir().ok_or("not a workdir").unwrap();

        let mut repo_tree = String::new();

        for e in WalkDir::new(repo_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(rel_path) = e.path().strip_prefix(repo_root)
                && !self.repo.status_should_ignore(rel_path).unwrap()
            {
                let path = rel_path.display().to_string();

                if self.ignored.contains(&path) {
                    continue;
                }

                repo_tree.push_str(&format!("{}\n", path));
            }
        }

        repo_tree
    }

    pub fn get_repo_status_as_str(&self) -> String {
        let mut staged = String::new();
        let mut unstaged = String::new();

        // oh lord of repetition grant me the power of the LLM!!
        for path in &self.status.s_new {
            if !self.ignored.contains(path) {
                staged.push_str(&format!("A  {}\n", path));
            }
        }

        for path in &self.status.s_modified {
            if !self.ignored.contains(path) {
                staged.push_str(&format!("M  {}\n", path));
            }
        }

        for path in &self.status.s_deleted {
            if !self.ignored.contains(path) {
                staged.push_str(&format!("D  {}\n", path));
            }
        }

        for (old, new) in &self.status.s_renamed {
            if !self.ignored.contains(old)
                && !self.ignored.contains(new)
            {
                staged.push_str(&format!("R  {} -> {}\n", old, new));
            }
        }

        for path in &self.status.u_new {
            if !self.ignored.contains(path) {
                unstaged.push_str(&format!("? {}\n", path));
            }
        }

        for path in &self.status.u_modified {
            if !self.ignored.contains(path) {
                unstaged.push_str(&format!("M {}\n", path));
            }
        }

        for path in &self.status.u_deleted {
            if !self.ignored.contains(path) {
                unstaged.push_str(&format!("D {}\n", path));
            }
        }

        for (old, new) in &self.status.u_renamed {
            if !self.ignored.contains(old)
                && !self.ignored.contains(new)
            {
                unstaged.push_str(&format!("R {} -> {}\n", old, new));
            }
        }

        let mut status_str = String::new();

        if !staged.is_empty() {
            status_str.push_str("Staged:\n");
            status_str.push_str(&staged);
            status_str.push('\n');
        }

        if !unstaged.is_empty() {
            status_str.push_str("Unstaged:\n");
            status_str.push_str(&unstaged);
        }

        status_str
    }

    pub fn get_file_diffs_as_str(&self) -> HashMap<String, String> {
        let mut file_diffs = HashMap::new();
        for gai_file in &self.files {
            let mut diff_str = String::new();
            if gai_file.should_truncate {
                diff_str.push_str("Truncated File");
                file_diffs.insert(gai_file.path.to_owned(), diff_str);
                continue;
            }

            for (i, hunk) in gai_file.hunks.iter().enumerate() {
                diff_str.push_str(&format!(
                    "Hunk_id[{}:{}]\n",
                    gai_file.path, i
                ));
                diff_str.push_str(&hunk.header);
                diff_str.push('\n');

                for line in &hunk.line_diffs {
                    let prefix = match line.diff_type {
                        DiffType::Unchanged => ' ',
                        DiffType::Additions => '+',
                        DiffType::Deletions => '-',
                    };
                    diff_str.push(prefix);
                    diff_str.push_str(&line.content);
                }
                diff_str.push('\n');
            }
            if !diff_str.trim().is_empty() {
                file_diffs.insert(gai_file.path.to_owned(), diff_str);
            }
        }

        file_diffs
    }
}
