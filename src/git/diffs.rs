use std::{cell::RefCell, path::Path, rc::Rc};

use git2::{
    Delta, Diff, DiffDelta, DiffFormat, DiffHunk, Patch, Repository,
};

use super::{
    errors::GitError,
    repo::GitRepo,
    settings::{DiffStrategy, StatusStrategy},
    status::get_status,
    utils::{get_head_repo, is_newline, new_file_content},
};

/// diff set
pub struct Diffs {
    pub files: Vec<FileDiff>,
}

/// helper struct for ez LLM hunk
/// designation, instead of copying
/// entire hunk headers, hunks are ordered
/// as they are found within a file
/// this converts to src/main.rs:0 for the
/// first hunk in a src/main.rs diff
pub struct HunkId {
    pub path: String,
    pub index: usize,
}

#[derive(Debug, Default)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub lines: usize,
    pub untracked: bool,
}

#[derive(Debug)]
pub struct Hunk {
    pub header: HunkHeader,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HunkHeader {
    // copied from DiffHunk
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    // full raw header
    //raw: String,
}

/// type of diff of a single line
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum DiffLineType {
    /// just surrounding line, no change
    #[default]
    None,
    /// header of the hunk
    Header,
    /// line added
    Add,
    /// line deleted
    Delete,
}

#[derive(Clone, Copy, Default, Hash, Debug, PartialEq, Eq)]
pub struct DiffLinePosition {
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Default, Clone, Hash, Debug)]
pub struct DiffLine {
    pub content: Box<str>,
    pub line_type: DiffLineType,
    pub position: DiffLinePosition,
}

impl From<git2::DiffLineType> for DiffLineType {
    fn from(line_type: git2::DiffLineType) -> Self {
        match line_type {
            git2::DiffLineType::HunkHeader => Self::Header,
            git2::DiffLineType::DeleteEOFNL
            | git2::DiffLineType::Deletion => Self::Delete,
            git2::DiffLineType::AddEOFNL
            | git2::DiffLineType::Addition => Self::Add,
            _ => Self::None,
        }
    }
}

impl From<&git2::DiffLine<'_>> for DiffLinePosition {
    fn from(line: &git2::DiffLine<'_>) -> Self {
        Self {
            old_lineno: line.old_lineno(),
            new_lineno: line.new_lineno(),
        }
    }
}

impl From<DiffHunk<'_>> for HunkHeader {
    fn from(h: DiffHunk) -> Self {
        Self {
            old_start: h.old_start(),
            old_lines: h.old_lines(),
            new_start: h.new_start(),
            new_lines: h.new_lines(),
            /* raw: String::from_utf8(h.header().to_vec())
            .unwrap_or_default(), */
        }
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

pub fn get_diffs(
    git_repo: &GitRepo,
    strategy: &DiffStrategy,
) -> anyhow::Result<Diffs> {
    let mut files = Vec::new();

    let status_strategy = if strategy.staged_only {
        StatusStrategy::Stage
    } else {
        StatusStrategy::Both
    };

    let status = get_status(&git_repo.repo, status_strategy)?;

    for file in status.statuses {
        let raw_diff =
            get_diff_raw(&git_repo.repo, &file.path, strategy)?;

        let file_diff = raw_diff_to_file_diff(
            &raw_diff,
            &file.path,
            &git_repo.workdir,
        )?;

        files.push(file_diff);
    }

    Ok(Diffs { files })
}

fn get_diff_raw<'a>(
    repo: &'a Repository,
    path: &str,
    strategy: &DiffStrategy,
) -> anyhow::Result<Diff<'a>> {
    let mut opt = git2::DiffOptions::new();

    opt.pathspec(path);

    let diff = if strategy.staged_only {
        // diff against head
        if let Ok(id) = get_head_repo(repo) {
            let parent = repo.find_commit(id)?;

            let tree = parent.tree()?;
            repo.diff_tree_to_index(
                Some(&tree),
                Some(&repo.index()?),
                Some(&mut opt),
            )?
        } else {
            repo.diff_tree_to_index(
                None,
                Some(&repo.index()?),
                Some(&mut opt),
            )?
        }
    } else {
        opt.include_untracked(true);
        opt.recurse_untracked_dirs(true);
        repo.diff_index_to_workdir(None, Some(&mut opt))?
    };

    Ok(diff)
}

// use original asyncgit to read
// diff per file then filter/process
// todo process all diffs together
// filter as you come acorss
fn raw_diff_to_file_diff(
    diff: &Diff,
    path: &str,
    work_dir: &Path,
) -> anyhow::Result<FileDiff> {
    let res = Rc::new(RefCell::new(FileDiff {
        path: path.to_owned(),
        ..Default::default()
    }));
    {
        let mut current_lines = Vec::new();
        let mut current_hunk: Option<HunkHeader> = None;

        let res_cell = Rc::clone(&res);
        let adder = move |header: &HunkHeader,
                          lines: &Vec<DiffLine>| {
            let mut res = res_cell.borrow_mut();
            res.hunks.push(Hunk {
                header: header.to_owned(),
                lines: lines.to_owned(),
            });
            res.lines += lines.len();
        };

        let mut put = |_: DiffDelta,
                       hunk: Option<DiffHunk>,
                       line: git2::DiffLine| {
            if let Some(hunk) = hunk {
                let hunk_header = HunkHeader::from(hunk);

                match current_hunk {
                    None => current_hunk = Some(hunk_header),
                    Some(h) => {
                        if h != hunk_header {
                            adder(&h, &current_lines);
                            current_lines.clear();
                            current_hunk = Some(hunk_header);
                        }
                    }
                }

                let diff_line = DiffLine {
                    position: DiffLinePosition::from(&line),
                    content: String::from_utf8_lossy(line.content())
                        //Note: trim await trailing newline characters
                        .trim_matches(is_newline)
                        .into(),
                    line_type: line.origin_value().into(),
                };

                current_lines.push(diff_line);
            }
        };

        let new_file_diff = if diff.deltas().len() == 1 {
            if let Some(delta) = diff.deltas().next() {
                if delta.status() == Delta::Untracked {
                    let relative_path =
                        delta.new_file().path().ok_or_else(|| {
                            GitError::Generic(
                                "new file path is unspecified."
                                    .to_string(),
                            )
                        })?;

                    let newfile_path = work_dir.join(relative_path);

                    if let Some(newfile_content) =
                        new_file_content(&newfile_path)
                    {
                        let mut patch = Patch::from_buffers(
                            &[],
                            None,
                            newfile_content.as_slice(),
                            Some(&newfile_path),
                            None,
                        )?;

                        patch.print(
							&mut |delta,
							      hunk: Option<DiffHunk>,
							      line: git2::DiffLine| {
								put(delta, hunk, line);
								true
							},
						)?;

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !new_file_diff {
            diff.print(
                DiffFormat::Patch,
                move |delta, hunk, line: git2::DiffLine| {
                    put(delta, hunk, line);
                    true
                },
            )?;
        }

        if !current_lines.is_empty() {
            adder(
                &current_hunk.map_or_else(
                    || {
                        Err(GitError::Generic(
                            "invalid hunk".to_owned(),
                        ))
                    },
                    Ok,
                )?,
                &current_lines,
            );
        }

        if new_file_diff {
            res.borrow_mut().untracked = true;
        }
    }

    let res = Rc::try_unwrap(res).map_err(|_| {
        GitError::Generic("rc unwrap error".to_owned())
    })?;

    Ok(res.into_inner())
}

/* // for tracked files
fn create_file_diff() -> anyhow::Result<FileDiff> {
    //let mut patch = Patch::from_blob()

    todo!()
}

// for untracked files
fn create_new_file_diff() -> anyhow::Result<FileDiff> {
    todo!()
} */
