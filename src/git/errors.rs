use std::error::Error;

/// errors for git2rs
/// or wrapper related
/// error types
#[derive(Debug)]
pub enum GitError {
    BareRepo,
    InvalidHunk { hunk: String },
    NoHead,
    Generic(String),
}

impl std::fmt::Display for GitError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            GitError::BareRepo => {
                write!(f, "This is a bare repository")
            }
            GitError::InvalidHunk { hunk } => {
                write!(f, "Invalid Hunk:{}", hunk)
            }
            GitError::NoHead => write!(f, "No Head found"),
            GitError::Generic(e) => write!(f, "{}", e),
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            _ => None,
        }
    }
}
