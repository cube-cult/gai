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

const NEWLINE: char = '\n';

struct NewFromOldContent {
    lines: Vec<String>,
    old_index: usize,
}

impl NewFromOldContent {
    fn add_from_hunk(
        &mut self,
        line: &git2::DiffLine,
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
