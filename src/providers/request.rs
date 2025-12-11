use std::{collections::HashMap, fmt};

use crate::{
    git::repo::GaiGit,
    settings::{PromptRules, Settings},
    utils::consts::*,
};

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub prompt: String,
    pub diffs: String,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Request Prompt:")?;
        writeln!(f, "{}", self.prompt)?;

        writeln!(f)?;

        writeln!(f, "Request Diffs:")?;
        writeln!(f, "{}", self.diffs)?;

        writeln!(f)
    }
}

pub fn build_request(
    cfg: &Settings,
    gai: &GaiGit,
    spinner: &crate::utils::print::SpinDeez,
) -> Request {
    spinner.start("Building Request...");
    let mut req = Request::default();
    req.build_prompt(cfg, gai);
    req.build_diffs_string(gai.get_file_diffs_as_str());
    spinner.stop(None);
    req
}

impl Request {
    pub fn build_diffs_string(
        &mut self,
        diffs: HashMap<String, String>,
    ) {
        let mut diffs_str = String::new();

        for (file, diff) in diffs {
            let file_diff = format!(
                "File Name:{}\nDiff Content:{}\n\n",
                file, diff
            );

            diffs_str.push_str(&file_diff);
        }

        self.diffs = diffs_str;
    }

    pub fn build_prompt(&mut self, cfg: &Settings, gai: &GaiGit) {
        let mut prompt = String::new();

        let rules = build_rules(&cfg.rules);

        if let Some(sys_prompt) = &cfg.prompt.system_prompt {
            prompt.push_str(sys_prompt);
        } else {
            prompt.push_str(DEFAULT_SYS_PROMPT);
        };

        prompt.push('\n');

        if let Some(hint) = &cfg.prompt.hint {
            prompt.push_str(
                format!(
                    "USE THIS IS A HINT FOR YOUR COMMITS: {}",
                    hint
                )
                .as_str(),
            );
            prompt.push('\n');
        }

        if cfg.commit.only_staged {
            prompt.push_str(PROMPT_ONLY_STAGED);
        }

        prompt.push_str(&rules);
        prompt.push('\n');

        if let Some(commit_conv) = &cfg.prompt.commit_convention {
            prompt.push_str(commit_conv);
        }

        if cfg.context.include_convention {
            prompt.push_str(COMMIT_CONVENTION);
        }

        if cfg.commit.stage_hunks {
            prompt.push_str(PROMPT_STAGE_HUNKS);
        } else {
            prompt.push_str(PROMPT_STAGE_FILES);
        }

        prompt.push('\n');

        if cfg.context.include_file_tree {
            prompt.push_str("Current File Tree: \n");
            prompt.push_str(&gai.get_repo_tree());
            prompt.push('\n');
        }

        if cfg.context.include_git_status {
            prompt.push_str("Current Git Status: \n");
            prompt.push_str(&gai.get_repo_status_as_str());
        }

        self.prompt = prompt;
    }
}

fn build_rules(cfg: &PromptRules) -> String {
    let mut rules = String::new();

    if cfg.group_related_files {
        rules.push_str(RULE_GROUP_FILES);
    }

    if cfg.separate_by_purpose {
        rules.push_str(RULE_SEPARATE_BY_PURPOSE);
    }

    rules.push_str(RULE_COMMIT_MESSAGE_HEADER);
    rules.push_str(RULE_PREFIX);

    let scope_rule =
        match (cfg.allow_empty_scope, cfg.extension_in_scope) {
            (true, true) => RULE_SCOPE_ALLOW_EMPTY_NO_EXTENSION,
            (true, false) => RULE_SCOPE_ALLOW_EMPTY_WITH_EXTENSION,
            (false, true) => RULE_SCOPE_REQUIRED_NO_EXTENSION,
            (false, false) => RULE_SCOPE_REQUIRED_WITH_EXTENSION,
        };
    rules.push_str(scope_rule);

    rules.push_str(RULE_BREAKING);

    rules.push_str(RULE_HEADER_BASE);
    rules.push_str(&format!(
        "    - CRITICAL: Maximum length is {} characters\n",
        cfg.max_header_length
    ));

    if cfg.allow_body {
        rules.push_str(RULE_BODY_BASE);
        rules.push_str(&format!(
            "    - CRITICAL: Maximum length is {} characters\n",
            cfg.max_body_length
        ));
    } else {
        rules.push_str("DO NOT CREATE A BODY, LEAVE IT BLANK");
    }

    if cfg.verbose_descriptions {
        rules.push_str(RULE_MESSAGE_VERBOSE);
    } else {
        rules.push_str(RULE_MESSAGE_CONCISE);
    }

    rules.push('\n');
    rules
}
