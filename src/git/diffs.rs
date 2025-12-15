use super::{
    repo::{GaiGit, GitRepo},
    settings::DiffStrategy,
};

// https://libgit2.org/docs/reference/main/diff/git_diff_delta.html

pub struct Diffs {
    pub files: Vec<FileDiff>,
}

pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub status: DiffDeltaStatus,
}

// diffdelta status
// ignoring others
pub enum DiffDeltaStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct LineDiff {
    pub kind: LineKind,
    pub content: String,
}

pub struct Hunk {
    pub index: usize,
    pub header: HunkHeader,
    /// raw content of
    /// what the hunk contains
    pub raw: String,
    pub lines: Vec<LineDiff>,
}

pub struct HunkHeader {
    // copied from DiffHunk
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    // full raw header
    raw: String,
}

/// taken from diffline::origin
#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub enum LineKind {
    #[default]
    Unchanged,
    Additions,
    Deletions,
}

impl LineKind {
    pub fn from_diff_line(c: char) -> Option<Self> {
        match c {
            ' ' => Some(Self::Unchanged),
            '+' => Some(Self::Additions),
            '-' => Some(Self::Deletions),
            _ => None,
        }
    }

    pub fn prefix(&self) -> char {
        match self {
            LineKind::Unchanged => ' ',
            LineKind::Additions => '+',
            LineKind::Deletions => '-',
        }
    }
}

impl Hunk {
    ///helper funcs for future ui/tui
    pub fn additions(&self) -> Vec<&LineDiff> {
        self.lines
            .iter()
            .filter(|l| l.kind == LineKind::Additions)
            .collect()
    }

    pub fn deletions(&self) -> Vec<&LineDiff> {
        self.lines
            .iter()
            .filter(|l| l.kind == LineKind::Additions)
            .collect()
    }
}

impl Diffs {
    pub fn create(
        repo: &GitRepo,
        strategy: &DiffStrategy,
    ) -> anyhow::Result<Self> {
        let mut opts = git2::DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let repo = &repo.repo;
        let head = repo.head()?.peel_to_tree()?;

        let diff = if strategy.staged_only {
            repo.diff_tree_to_index(
                Some(&head),
                None,
                Some(&mut opts),
            )?
        } else {
            repo.diff_tree_to_workdir_with_index(
                Some(&head),
                Some(&mut opts),
            )?
        };

        let mut files = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(file_diff) = file_cb(delta, strategy) {
                    files.push(file_diff);
                }
                true
            },
            Some(&mut |delta, _| binary_cb(delta, strategy)),
            Some(&mut |delta, hunk| true),
            Some(&mut |delta, hunk, line| true),
        )?;

        Ok(Self { files })
    }
}

// file callback for diff delta loop
fn file_cb(
    diff_delta: git2::DiffDelta,
    strategy: &DiffStrategy,
) -> Option<FileDiff> {
    let path = diff_delta
        .new_file()
        .path()
        .and_then(|p| p.to_str())
        .unwrap_or_default()
        .to_owned();

    if strategy.ignored_files.iter().any(|p| p == &path) {
        return None;
    }

    let status = match diff_delta.status() {
        git2::Delta::Added => DiffDeltaStatus::Added,
        git2::Delta::Deleted => DiffDeltaStatus::Deleted,
        git2::Delta::Modified => DiffDeltaStatus::Modified,
        git2::Delta::Renamed => DiffDeltaStatus::Renamed,
        _ => return None,
    };

    Some(FileDiff {
        path,
        hunks: Vec::new(),
        status,
    })
}

fn binary_cb(
    _delta: git2::DiffDelta,
    _strategy: &DiffStrategy,
) -> bool {
    true
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

/// taken from diffline::origin
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DiffType {
    Unchanged,
    Additions,
    Deletions,
}

impl GaiGit {
    pub fn create_diffs(
        &mut self,
        truncate_files: Option<&[String]>,
    ) -> Result<(), git2::Error> {
        Ok(())
    }
}
