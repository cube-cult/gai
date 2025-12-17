use std::fmt;

use chrono::DateTime;
use git2::Repository;

#[derive(Debug, Default)]
pub struct Logs {
    pub git_logs: Vec<GitLog>,
}

#[derive(Debug, Default)]
pub struct GitLog {
    pub prefix: Option<String>,
    pub breaking: bool,
    pub scope: Option<String>,
    pub header: Option<String>,
    pub body: Option<String>,

    // raw git commit message
    // used when we could not parse
    // prefix, scope, or header
    pub raw: String,

    pub date: String,
    pub author: String,
    pub commit_hash: String,
}

impl From<&[u8]> for GitLog {
    fn from(value: &[u8]) -> Self {
        let raw = String::from_utf8(value.to_owned())
            .unwrap_or("Failed to convert msg from utf8".to_owned());

        GitLog {
            raw,
            ..Default::default()
        }
    }
}

impl fmt::Display for Logs {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut s = String::new();

        for log in &self.git_logs {
            s.push_str(&format!("Author: {}", &log.author));
            s.push('\n');
            s.push_str(&format!("Message: {}", &log.raw));
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

pub fn get_logs(
    repo: &Repository,
    count: usize,
    reverse: bool,
) -> anyhow::Result<Logs> {
    let mut revwalk = repo.revwalk()?;

    if reverse {
        revwalk.set_sorting(git2::Sort::REVERSE)?;
    }

    revwalk.push_head()?;
    let cont = if count == 0 { !0 } else { count };
    let revwalk = revwalk.take(cont);

    let mut git_logs = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let mut log: GitLog = commit.message_bytes().into();

        let author = commit.author();
        log.author =
            author.name().unwrap_or("unknown author").to_string();
        log.commit_hash = oid.to_string();
        log.date =
            DateTime::from_timestamp(author.when().seconds(), 0)
                .map(|dt| dt.format("%m/%d/%Y %H:%M:%S").to_string())
                .unwrap_or_default();

        git_logs.push(log);
    }

    Ok(Logs { git_logs })
}
