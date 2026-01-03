// https://github.com/gitui-org/gitui/blob/master/asyncgit/src/sync/staging/mod.rs

// ripped from asyncgit
// necessary for staging math
// since we're getting multiple
// commits at a time
// we're gonna need to have to stage
// individual hunks as specified
// in the response separately
// depending on the accuracy,
// will make these operations separate
//
// NOPE, i dont think the patch math works or is
// necessary for this,
// i was thinking of doing something
// similar to reassemble_patch from
// https://github.com/git/git/blob/master/add-patch.c
// but like before, the count will change when we apply
// the commit
//
// new plan.
// we store the old diffs, in a sort of
// database in state.rs.
// the LLM replies with the hunkids
// then we do a match for that hunkid
// in the database. grab the relevant lines,
// addition, deletion, etc.
// then compare those changes with each new diff
// if it matches take that hunk and subtract it
// from the old diff db. We do this for each file
// so that means all relevant hunks need to grouped
// per file per commit
//
// an issue might come up, if for example there are multiple
// of the same changes
// do we try to increase the diff context in this case?
// to try and get a bigger match?
// for now lets bail
//

/// for different types
/// of adding/staging per commit
#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize,
)]
pub enum StagingStrategy {
    /// as individual hunks
    Hunks,

    /// only stage one file PER commit
    OneFilePerCommit,

    /// small group of files
    /// files SHOULD represent
    /// a DISTINCT change per commit
    #[default]
    AtomicCommits,

    /// stage all changes together
    /// as monolithic commit
    /// best to enable allow_body
    /// so the LLM can generate
    /// a descriptive commit description
    AllFilesOneCommit,
}

use git2::{IndexAddOption, Repository};
use std::{collections::HashSet, path::Path};

use crate::git::lines::{get_changes_from_gai, get_changes_from_raw};

use super::{
    diffs::Hunk,
    errors::GitError,
    lines::stage_lines,
    patches::{get_file_diff_patch, patch_get_hunklines},
};

/// for hunk staging, this will
/// return a list of hunk ids that
/// were succesfully processed
/// HUNKS that are passed here are what
/// need to be processed from
/// this file_path
/// these HUNKS should be taken
/// directly from og_hunk_diffs
pub fn stage_hunks(
    repo: &Repository,
    file_path: &str,
    og_hunks_to_stage: &[Hunk],
) -> anyhow::Result<Vec<usize>> {
    if og_hunks_to_stage.is_empty() {
        return Err(GitError::Generic(
            "cannot stage empty hunk list".to_owned(),
        )
        .into());
    }

    // regen diff
    let patch = get_file_diff_patch(repo, file_path)?;
    let new_hunks = patch_get_hunklines(&patch)?;

    let mut already_used_new_hunks: HashSet<usize> = HashSet::new();

    let mut used = Vec::new();
    let mut lines = Vec::new();

    for hunk in og_hunks_to_stage {
        let gai_diff_lines = get_changes_from_gai(hunk);

        // attempt to find matching hunk
        let matching_hunk = new_hunks
            .iter()
            .enumerate()
            .filter(|(idx, _new_hunk)| {
                !already_used_new_hunks.contains(idx)
            })
            .find(|(_idx, new_hunk)| {
                let raw_diff_lines = get_changes_from_raw(new_hunk);
                if gai_diff_lines.len() != raw_diff_lines.len() {
                    false
                } else {
                    gai_diff_lines
                        .iter()
                        .zip(raw_diff_lines.iter())
                        .all(|(a, b)| {
                            a.line_type == b.line_type
                                && a.content == b.content
                        })
                }
            });

        match matching_hunk {
            Some((idx, matching_hunk)) => {
                already_used_new_hunks.insert(idx);
                for line in &matching_hunk.lines {
                    let origin = line.origin_value();
                    if origin == git2::DiffLineType::Addition
                        || origin == git2::DiffLineType::Deletion
                    {
                        lines.push(super::diffs::DiffLinePosition {
                            old_lineno: line.old_lineno(),
                            new_lineno: line.new_lineno(),
                        });
                    }
                }
                used.push(hunk.id);
            }
            None => {
                return Err(GitError::Generic(
                    "no matching hunk found".to_owned(),
                )
                .into());
            }
        }
    }

    if lines.is_empty() {
        return Err(GitError::Generic(
            "did you we not store any DiffLinePositions?".to_owned(),
        )
        .into());
    }

    // todo we regen diff here again
    // collapse this fucntion to here
    // and optimize
    stage_lines(repo, file_path, &lines)?;

    Ok(used)
}

/// for atomic commits
pub fn stage_file(
    repo: &Repository,
    path: &str,
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    match index.add_path(Path::new(path)) {
        Ok(_) => {}
        // OMG LOL, this is super cursed
        // todo do not do this, instead
        // handle this upwards
        Err(_) => remove_file(repo, path)?,
    }

    index.write()?;

    Ok(())
}

/// used for deletions, renames, etc
pub fn remove_file(
    repo: &Repository,
    path: &str,
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.remove_path(Path::new(path))?;
    index.write()?;

    Ok(())
}

/// for allfilesonecommit
pub fn stage_all(
    repo: &Repository,
    pattern: &str,
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.add_all(vec![pattern], IndexAddOption::DEFAULT, None)?;

    index.write()?;

    Ok(())
}
