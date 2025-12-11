use super::{
    CommitSettings, ContextSettings, PromptRules, PromptSettings,
    TuiSettings,
};

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            system_prompt: None,
            commit_convention: None,
            hint: None,
            extra: None,
        }
    }
}

impl Default for PromptRules {
    fn default() -> Self {
        Self {
            group_related_files: true,
            separate_by_purpose: true,
            verbose_descriptions: true,
            extension_in_scope: false,
            allow_empty_scope: true,
            max_header_length: 52,
            allow_body: false,
            max_body_length: 72,
        }
    }
}
impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            include_convention: false,
            include_file_tree: false,
            include_git_status: true,
            include_untracked: true,
            files_to_truncate: None,
            include_log: true,
            log_amount: 5,
        }
    }
}

impl Default for CommitSettings {
    fn default() -> Self {
        Self {
            only_staged: false,
            stage_hunks: false,
            capitalize_prefix: false,
            include_scope: true,
            include_breaking: true,
            breaking_symbol: '!',
        }
    }
}

impl Default for TuiSettings {
    fn default() -> Self {
        Self {}
    }
}
