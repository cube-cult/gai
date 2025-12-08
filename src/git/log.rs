use crate::git::commit::GaiCommit;

use super::repo::GaiGit;

impl GaiGit {
    pub fn get_logs(&self) -> anyhow::Result<Vec<GaiCommit>> {
        println!("printing commits");
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        macro_rules! filter_try {
            ($e:expr) => {
                match $e {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                }
            };
        }

        let revwalk = revwalk.map(|id| {
            let id = filter_try!(id);
            let commit = filter_try!(self.repo.find_commit(id));
            Ok(commit)
        });

        for commit in revwalk {
            let commit = commit?;
            println!(
                "commit {} by {} at {:?}",
                commit.id(),
                commit.author(),
                commit.author().when()
            );
        }

        Ok(Vec::new())
    }
}
