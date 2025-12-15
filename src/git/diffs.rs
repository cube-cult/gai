use super::{
    errors::GitError, repo::GitRepo, settings::DiffStrategy,
};

// https://libgit2.org/docs/reference/main/diff/git_diff_delta.html

pub struct GitDiffs {
    pub files: Vec<FileDiff>,
}

#[derive(Default)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub lines: usize,
    pub untracked: bool,
}

// diffdelta status
// ignoring others

#[derive(Debug, Clone)]
pub struct LineDiff {
    pub kind: LineKind,
    pub content: String,
}

pub struct HunkId {
    pub path: String,
    pub index: usize,
}

pub struct Hunk {
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

impl HunkId {
    pub fn parse(from_str: &str) -> anyhow::Result<Self> {
        let (path, index) =
            from_str.split_once(':').ok_or_else(|| {
                GitError::InvalidHunk {
                    hunk: from_str.to_owned(),
                }
            })?;

        let path = path.to_owned();
        let index =
            index.parse().map_err(|_| GitError::InvalidHunk {
                hunk: from_str.to_owned(),
            })?;

        Ok(Self { path, index })
    }
}

impl HunkHeader {}

impl GitDiffs {
    pub fn create(
        git_repo: &GitRepo,
        strategy: &DiffStrategy,
    ) -> anyhow::Result<Self> {
        let files = Vec::new();

        Ok(Self { files })
    }
}
