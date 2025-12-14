use git2::{DiffDelta, DiffHunk, DiffLine, DiffOptions};
use walkdir::WalkDir;

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

pub struct Hunk {
    pub index: usize,
    /// raw content of
    /// what the hunk contains
    pub raw: String,

    // copied from DiffHunk
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    // in bytes
    header: u8,
}

impl Hunk {
    pub fn from_diff_hunk(hunk: &DiffHunk) -> Self {
        Self {
            index: todo!(),
            raw: todo!(),
            old_start: todo!(),
            old_lines: todo!(),
            new_start: todo!(),
            new_lines: todo!(),
            header: todo!(),
        }
    }
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

impl Diffs {
    pub fn create(
        repo: &GitRepo,
        strategy: &DiffStrategy,
    ) -> anyhow::Result<Self> {
        let mut opts = DiffOptions::new();
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
    diff_delta: DiffDelta,
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

fn binary_cb(_delta: DiffDelta, _strategy: &DiffStrategy) -> bool {
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

#[derive(Debug, Clone)]
pub struct LineDiff {
    pub diff_type: DiffType,
    pub content: String,
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
        // start this puppy up
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let repo = &self.repo;

        let head = repo.head()?.peel_to_tree()?;
        let diff = if self.only_staged {
            repo.diff_tree_to_index(
                Some(&head),
                None,
                Some(&mut opts),
            )?
        } else {
            repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?
        };

        let mut gai_files: Vec<GaiFile> = Vec::new();

        // hilarious
        let files_to_truncate = if let Some(f) = truncate_files {
            f
        } else {
            &Vec::new()
        };

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            let path = delta
                .new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let should_truncate =
                files_to_truncate.iter().any(|f| path.ends_with(f));

            let gai_file =
                match gai_files.iter_mut().find(|g| g.path == path) {
                    Some(existing) => existing,
                    None => {
                        gai_files.push(GaiFile {
                            path: path.clone(),
                            should_truncate,
                            hunks: Vec::new(),
                        });
                        gai_files.last_mut().unwrap()
                    }
                };

            process_file_diff(&mut gai_file.hunks, &hunk, &line);

            true
        })?;

        if self.only_staged {
            return Ok(());
        }

        self.files = gai_files;

        // handle untracked files here
        for path in &self.status.u_new {
            let should_truncate =
                files_to_truncate.iter().any(|f| path.ends_with(f));

            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().is_file()
                    && let Ok(content) =
                        std::fs::read_to_string(entry.path())
                {
                    let path = entry.path().to_str().unwrap();
                    let lines: Vec<LineDiff> = content
                        .lines()
                        .map(|line| LineDiff {
                            diff_type: DiffType::Additions,
                            content: format!("{}\n", line),
                        })
                        .collect();

                    self.files.push(GaiFile {
                        path: path.to_owned(),
                        should_truncate,
                        hunks: vec![HunkDiff {
                            header: format!(
                                "New File {}",
                                lines.len()
                            ),
                            line_diffs: lines,
                        }],
                    });
                }
            }
        }

        self.files.sort_by_key(|g| g.should_truncate);

        Ok(())
    }
}

fn process_file_diff(
    diff_hunks: &mut Vec<HunkDiff>,
    hunk: &Option<DiffHunk>,
    line: &DiffLine,
) {
    if let Some(h) = hunk {
        let header = str::from_utf8(h.header())
            .unwrap_or("not a valid utf8 header from hunk")
            .to_owned();
        let content = str::from_utf8(line.content())
            .unwrap_or("not a valid utf8 line from hunk")
            .to_owned();

        let diff_type = match line.origin() {
            '+' => DiffType::Additions,
            '-' => DiffType::Deletions,
            ' ' => DiffType::Unchanged,
            _ => return,
        };

        let diff_line = LineDiff { diff_type, content };

        // instead of storing the different types.
        // we can just push line diffs in a clear order
        // if i want to filter it out, i can do that
        // later, this should just care about the diff itself
        match diff_hunks.iter_mut().find(|h| h.header == header) {
            Some(existing) => existing.line_diffs.push(diff_line),
            None => {
                diff_hunks.push(HunkDiff {
                    header,
                    line_diffs: vec![diff_line],
                });
            }
        }
    }
}
