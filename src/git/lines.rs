use git2::{DiffLine, DiffLineType, Repository};
use std::{collections::HashSet, path::Path};

use crate::git::diffs::Hunk;

use super::{
    diffs::DiffLinePosition,
    errors::GitError,
    patches::{HunkLines, get_file_diff_patch, patch_get_hunklines},
    utils::is_newline,
};

/// helper fn to convert and get the changes from
/// raw git2::DiffLines, the only HunkLines that
/// should be passed here are regenerated diffs from
/// stage_hunks. these and the diffLines from
/// get_changes_from_gai will be compared later
pub fn get_changes_from_raw(
    hunk: &HunkLines
) -> Vec<super::diffs::DiffLine> {
    hunk.lines
        .iter()
        .filter(|l| {
            l.origin_value() == DiffLineType::Addition
                || l.origin_value() == DiffLineType::Deletion
        })
        .map(|l| {
            let line_type =
                super::diffs::DiffLineType::from(l.origin_value());

            let content = String::from_utf8_lossy(l.content())
                .trim_matches(is_newline)
                .into();

            super::diffs::DiffLine {
                line_type,
                content,
                position: DiffLinePosition::from(l),
            }
        })
        .collect()
}

/// helper fn to get ONLY the additions or deletions
/// within a specific specified hunk
/// this is called by stage_hunks()
/// and the only hunk that should be passed in
/// is the og_hunks from the og hunk database
/// THE INITIAL HUNKS >
/// THESE ARE FROM git::diffs::Hunk
pub fn get_changes_from_gai(
    hunk: &Hunk
) -> Vec<super::diffs::DiffLine> {
    hunk.lines
        .iter()
        .filter(|l| {
            matches!(
                l.line_type,
                super::diffs::DiffLineType::Add
                    | super::diffs::DiffLineType::Delete
            )
        })
        .map(|l| super::diffs::DiffLine {
            line_type: l.line_type,
            content: l.content.to_owned(),
            position: l.position,
        })
        .collect()
}

// lifted from asyncgit
// making some assumptions here
// so some of the original
// was stripped

/// used by hunk staging
/// modified from asyncgit's stage_lines()
/// stages matching lines from the
/// response commit hunks, which are matched
/// from the original diff database*
pub fn stage_lines(
    repo: &Repository,
    file_path: &str,
    lines: &[DiffLinePosition],
) -> anyhow::Result<()> {
    if lines.is_empty() {
        return Ok(());
    }

    let mut index = repo.index()?;
    index.read(true)?;

    let mut idx =
        index.get_path(Path::new(file_path), 0).ok_or_else(|| {
            GitError::Generic(String::from(
                "only non new files supported",
            ))
        })?;

    let blob = repo.find_blob(idx.id)?;

    let indexed_content = String::from_utf8(blob.content().into())?;

    let new_content = {
        let patch = get_file_diff_patch(repo, file_path)?;
        let hunks = patch_get_hunklines(&patch)?;
        let old_lines = indexed_content.lines().collect::<Vec<_>>();

        apply_selection(lines, &hunks, &old_lines)?
    };

    let blob_id = repo.blob(new_content.as_bytes())?;

    idx.id = blob_id;
    idx.file_size = u32::try_from(new_content.len())?;
    index.add(&idx)?;

    index.write()?;
    index.read(true)?;

    Ok(())
}

const NEWLINE: char = '\n';

// this is the heart of the per line discard,stage,unstage.
// heavily inspired by the great work in
// nodegit: https://github.com/nodegit/nodegit
fn apply_selection(
    lines: &[DiffLinePosition],
    hunks: &[HunkLines],
    old_lines: &[&str],
) -> anyhow::Result<String> {
    let mut new_content = NewFromOldContent::default();
    let lines = lines.iter().collect::<HashSet<_>>();

    let mut first_hunk_encountered = false;

    for hunk in hunks {
        let hunk_start = usize::try_from(hunk.hunk.old_start)?;

        if !first_hunk_encountered {
            let any_selection_in_hunk =
                hunk.lines.iter().any(|line| {
                    let line: DiffLinePosition = line.into();
                    lines.contains(&line)
                });

            first_hunk_encountered = any_selection_in_hunk;
        }

        if first_hunk_encountered {
            new_content.catchup_to_hunkstart(hunk_start, old_lines);

            for hunk_line in &hunk.lines {
                let hunk_line_pos: DiffLinePosition =
                    hunk_line.into();
                let selected_line = lines.contains(&hunk_line_pos);

                if hunk_line.origin_value()
                    == DiffLineType::DeleteEOFNL
                    || hunk_line.origin_value()
                        == DiffLineType::AddEOFNL
                {
                    break;
                }

                if selected_line {
                    // stage the line
                    if hunk_line.origin_value()
                        == DiffLineType::Addition
                    {
                        new_content.add_from_hunk(hunk_line)?;
                    } else if hunk_line.origin_value()
                        == DiffLineType::Deletion
                    {
                        new_content.skip_old_line();
                    } else {
                        new_content.add_old_line(old_lines);
                    }
                } else if hunk_line.origin_value()
                    != DiffLineType::Addition
                {
                    new_content.add_from_hunk(hunk_line)?;
                    new_content.skip_old_line();
                }
            }
        }
    }

    Ok(new_content.finish(old_lines))
}

#[derive(Default)]
struct NewFromOldContent {
    lines: Vec<String>,
    old_index: usize,
}

impl NewFromOldContent {
    fn add_from_hunk(
        &mut self,
        line: &DiffLine,
    ) -> anyhow::Result<()> {
        let line = String::from_utf8(line.content().into())?;

        let line = if line.ends_with(NEWLINE) {
            line[0..line.len() - 1].to_string()
        } else {
            line
        };

        self.lines.push(line);

        Ok(())
    }

    fn skip_old_line(&mut self) {
        self.old_index += 1;
    }

    fn add_old_line(
        &mut self,
        old_lines: &[&str],
    ) {
        self.lines.push(old_lines[self.old_index].to_string());
        self.old_index += 1;
    }

    fn catchup_to_hunkstart(
        &mut self,
        hunk_start: usize,
        old_lines: &[&str],
    ) {
        while hunk_start > self.old_index + 1 {
            self.add_old_line(old_lines);
        }
    }

    fn finish(
        mut self,
        old_lines: &[&str],
    ) -> String {
        for line in old_lines.iter().skip(self.old_index) {
            self.lines.push((*line).to_string());
        }
        let lines = self.lines.join("\n");
        if lines.ends_with(NEWLINE) {
            lines
        } else {
            let mut lines = lines;
            lines.push(NEWLINE);
            lines
        }
    }
}
