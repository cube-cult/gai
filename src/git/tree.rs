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

    for e in walkdir::WalkDir::new(workdir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(rel_path) = e.path().strip_prefix(workdir)
            && !repo.status_should_ignore(rel_path).unwrap()
        {
            let path = rel_path.display().to_string();

            repo_tree.push_str(&format!("{}\n", path));
        }
    }

    repo_tree
}
