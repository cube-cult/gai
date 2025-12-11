use crate::providers::provider::{ProviderConfigs, ProviderKind};

use super::{AiConfig, CommitConfig, RuleConfig};

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Gai,
            system_prompt: None,
            commit_convention: None,
            include_convention: true,
            include_file_tree: true,
            include_git_status: true,
            include_untracked: true,
            files_to_truncate: vec![],
            rules: RuleConfig::default(),
            provider_configs: ProviderConfigs::default(),
            hint: None,
        }
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            group_related_files: true,
            no_file_splitting: true,
            separate_by_purpose: true,
            verbose_descriptions: true,
            exclude_extension_in_scope: true,
            allow_empty_scope: true,
            max_header_length: 52,
            allow_body: true,
            max_body_length: 72,
        }
    }
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            capitalize_prefix: false,
            include_scope: true,
            include_breaking: true,
            breaking_symbol: None,
        }
    }
}
