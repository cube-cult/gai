use git2::{Diff, DiffLine, Patch, Repository};

use super::{diffs::HunkHeader, errors::GitError};

// funcs here are used internally, specifically for
// hunk staging.
// these were ripped from asyncgit
// attribution should be in the readme (pls remind if no)
// stripped some uneccessary vars
// like is_staged
// reverse
// since stage_lines() can only RUN
// on non-staged files as well as FORWARD

pub struct HunkLines<'a> {
    pub hunk: HunkHeader,
    pub lines: Vec<DiffLine<'a>>,
}

pub fn get_file_diff_patch<'a>(
    repo: &'a Repository,
    file: &str,
) -> anyhow::Result<Patch<'a>> {
    let diff = get_diff_lines(repo, file)?;
    let patches = get_patches(&diff)?;
    if patches.len() > 1 {
        return Err(GitError::PatchError.into());
    }

    let patch = patches.into_iter().next().ok_or_else(|| {
        GitError::Generic(String::from("no patch found"))
    })?;

    Ok(patch)
}

//
pub fn patch_get_hunklines<'a>(
    patch: &'a Patch<'a>
) -> anyhow::Result<Vec<HunkLines<'a>>> {
    let count_hunks = patch.num_hunks();
    let mut res = Vec::with_capacity(count_hunks);
    for hunk_idx in 0..count_hunks {
        let (hunk, _) = patch.hunk(hunk_idx)?;

        let count_lines = patch.num_lines_in_hunk(hunk_idx)?;

        let mut hunk = HunkLines {
            hunk: HunkHeader::from(hunk),
            lines: Vec::with_capacity(count_lines),
        };

        for line_idx in 0..count_lines {
            let line = patch.line_in_hunk(hunk_idx, line_idx)?;
            hunk.lines.push(line);
        }

        res.push(hunk);
    }

    Ok(res)
}

fn get_diff_lines<'a>(
    repo: &'a Repository,
    path: &str,
) -> anyhow::Result<Diff<'a>> {
    let mut opt = git2::DiffOptions::new();

    opt.pathspec(path);
    opt.include_untracked(true);
    opt.recurse_untracked_dirs(true);

    // asyncgit uses 1
    opt.context_lines(3);

    let diff = repo.diff_index_to_workdir(None, Some(&mut opt))?;

    Ok(diff)
}

//
fn get_patches<'a>(
    diff: &Diff<'a>
) -> anyhow::Result<Vec<Patch<'a>>> {
    let count = diff.deltas().len();

    let mut res = Vec::with_capacity(count);
    for idx in 0..count {
        let p = Patch::from_diff(diff, idx)?;
        if let Some(p) = p {
            res.push(p);
        }
    }

    Ok(res)
}
