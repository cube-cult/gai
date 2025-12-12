use std::error::Error;

/// errors for git2rs
/// or wrapper related
/// error types
#[derive(Debug)]
pub enum GitError {
    Git2(git2::Error),
    BareRepo,
}

impl std::fmt::Display for GitError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            GitError::Git2(error) => write!(f, "{}", error),
            GitError::BareRepo => {
                write!(f, "This is a bare repository")
            }
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GitError::Git2(error) => Some(error),
            _ => None,
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::Git2(e)
    }
}
