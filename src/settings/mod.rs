pub mod defaults;
pub mod load;

use serde::{Deserialize, Serialize};

use crate::providers::provider::{ProviderKind, ProviderSettings};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub provider: ProviderKind,
    pub providers: ProviderSettings,

    /// custom prompt stuff
    pub prompt: PromptSettings,

    /// llm response rules
    pub rules: PromptRules,

    /// additional context
    pub context: ContextSettings,

    /// commit process settings after receiving
    /// llm generated commits
    pub commit: CommitSettings,

    /// styling, keybinds, etc
    pub tui: TuiSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptSettings {
    /// this is what tells the llm
    /// how to behave
    /// Defaults to NONE and will use
    /// the default_sys_prompt
    /// this is only for overriding
    pub system_prompt: Option<String>,

    /// Add custom convention
    /// this is separate from the built-in
    /// commit_convention_v1. Using both
    /// this and settings include_convention to true
    /// will take up a lot of tokens!
    pub commit_convention: Option<String>,

    /// add additional
    /// information to send
    /// along with the system_prompt
    /// different from the hint, since thats
    /// specific to running the commands
    /// the extra, gets sent every time
    /// we send the prompt
    pub extra: Option<String>,

    /// optional hinting for LLM's to lean on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptRules {
    /// group related files into logical commits based on the type of prefix
    pub group_related_files: bool,

    /// create SEPARATE commits when changes serve different purposes
    /// as in dont lump unrelated changes into one commit
    pub separate_by_purpose: bool,

    /// llm based verbosity vs concise
    pub verbose_descriptions: bool,

    /// file extensions in scope feat(git.rs) vs feat(git)
    pub extension_in_scope: bool,

    /// scope can be "" in the response
    pub allow_empty_scope: bool,

    // todo add hard validation
    /// max length of commit headers
    pub max_header_length: u16,

    /// allows the creation of commit bodies
    pub allow_body: bool,

    // todo add hard validation
    /// max length of commit body
    pub max_body_length: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSettings {
    /// include commit convention v1
    /// this is different from commit_convetion field
    /// from PromptSettings, as this is strictly
    /// the COMMITCONVENTION string
    /// (note: this takes a lot of tokens)
    pub include_convention: bool,

    /// include git repo file tree in request
    pub include_file_tree: bool,

    /// include git status
    pub include_git_status: bool,

    /// should we send untracked files as well?
    pub include_untracked: bool,

    // todo
    /// include past git logs
    /// just the commit headers
    /// (with prefixtype, etc)
    pub include_log: bool,

    /// if including log, how much?
    /// defaults to 5
    /// if 0 then its all
    pub log_amount: u64,

    /// files that gai will be TRUNCATED
    /// you can use this to add specific files
    /// that are not really relevant to send to the AI provider
    /// such as a Cargo.lock or package-lock.json file
    /// which may take up valuable token space
    pub files_to_truncate: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitSettings {
    /// only generate commits for staged files
    pub only_staged: bool,

    // todo this needs to be reworked
    // likely an enum, where we implement
    // tool calling
    /// should we apply as hunks?
    pub stage_hunks: bool,

    /// prefix will be capitalized like feat -> Feat
    pub capitalize_prefix: bool,

    /// the ai can respond with scopes
    /// instead of making it optional in the Schema
    /// (not all models support this)
    /// define it here before we apply the commit
    pub include_scope: bool,

    /// use breaking symbol
    pub include_breaking: bool,

    /// breaking override defaults to "!"
    pub breaking_symbol: char,
    // todo make hashmap for customizable prefix types
    // todo allow user customizable format
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TuiSettings {
    // todo, implement keybinds
    // and style settings
}
