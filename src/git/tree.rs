/// this is not related to ls-tree
/// this just a helper
/// to get current working directory structure
/// with respect to .gitignore
pub fn get_repo_tree(
    repo: &super::repo::GitRepo,
    workdir: &std::path::Path,
) -> String {
    let repo = &repo.repo;
    let mut repo_tree = String::new();

    repo_tree
}
