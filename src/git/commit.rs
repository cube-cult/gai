use git2::{Oid, Repository};

use crate::git::utils::get_head_repo;

#[derive(Debug)]
pub struct GitCommit {
    pub files: Vec<String>,
    pub hunk_ids: Vec<String>,
    pub message: String,
}

pub fn commit(
    repo: &Repository,
    commit: &GitCommit,
) -> anyhow::Result<Oid> {
    let mut index = repo.index()?;

    let signature = repo.signature()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let parents = if let Ok(id) = get_head_repo(repo) {
        vec![repo.find_commit(id)?]
    } else {
        Vec::new()
    };

    let parents = parents.iter().collect::<Vec<_>>();

    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit.message,
        &tree,
        parents.as_slice(),
    )?;

    Ok(oid)
}
