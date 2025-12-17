use git2::{Oid, Repository};

use crate::{
    git::utils::get_head_repo, providers::schema::ResponseCommit,
};

#[derive(Debug)]
pub struct GitCommit {
    pub files: Vec<String>,
    pub hunk_ids: Vec<String>,
    pub message: String,
}

// post processing happens before this
// parsing the ResponseCommit
// wont need any setting vars
impl From<ResponseCommit> for GitCommit {
    fn from(r: ResponseCommit) -> Self {
        let breaking = if r.breaking { "!" } else { "" };

        let scope = if !r.scope.is_empty() {
            format!("({})", r.scope)
        } else {
            "".to_owned()
        };

        let message = format!(
            "{}{}{}: {}\n{}",
            r.prefix, breaking, scope, r.header, r.body
        );

        let files = r.files;
        let hunk_ids = r.hunk_ids;

        Self {
            files,
            hunk_ids,
            message,
        }
    }
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

#[derive(Debug)]
pub struct GaiCommit {
    pub files: Vec<String>,
    pub hunk_ids: Vec<String>,
    pub message: String,
}

impl GaiCommit {
    pub fn from_response(
        response: &ResponseCommit,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> Self {
        let message = {
            let prefix = if capitalize_prefix {
                format!("{:?}", response.prefix).to_uppercase()
            } else {
                format!("{:?}", response.prefix).to_lowercase()
            };

            let breaking = if response.breaking { "!" } else { "" };
            let scope = if include_scope && !response.scope.is_empty()
            {
                // gonna set it to lowercase PERMA
                // sometimes the AI responds with a scope
                // that includes the file extension and is capitalized
                // like (Respfileonse.rs) which looks ridiculous imo
                // the only way i can think of is to make it a rule to not include
                // extension names
                format!("({})", response.scope.to_lowercase())
            } else {
                "".to_owned()
            };

            format!(
                "{}{}{}: {}\n{}",
                prefix,
                breaking,
                scope,
                response.header,
                if response.body.is_empty() {
                    String::new()
                } else {
                    format!("\n\n{}", response.body)
                }
            )
        };
        GaiCommit {
            files: response.files.to_owned(),
            hunk_ids: response.hunk_ids.to_owned(),
            message,
        }
    }
}
