use anyhow::{Result, anyhow};
use schemars::generate::SchemaSettings;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use strum::{Display, EnumIter, IntoEnumIterator};
use ureq::Agent;

use crate::{
    ai::response::ResponseSchema,
    auth::get_token,
    config::ProviderConfig,
    consts::{CHATGPT_DEFAULT, CLAUDE_DEFAULT, GEMINI_DEFAULT},
};

#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    Eq,
    PartialEq,
    EnumIter,
    Display,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
pub enum Provider {
    OpenAI,
    Gemini,
    Claude,
    Gai,
}

impl Provider {
    pub fn name(&self, model: &str) -> String {
        format!("{} ({})", self, model)
    }

    pub fn create_defaults() -> HashMap<Provider, ProviderConfig> {
        let mut providers = HashMap::new();
        for provider in Provider::iter() {
            match provider {
                Provider::OpenAI => providers.insert(
                    provider,
                    ProviderConfig::new(CHATGPT_DEFAULT),
                ),
                Provider::Gemini => providers.insert(
                    provider,
                    ProviderConfig::new(GEMINI_DEFAULT),
                ),
                Provider::Claude => providers.insert(
                    provider,
                    ProviderConfig::new(CLAUDE_DEFAULT),
                ),
                Provider::Gai => providers.insert(
                    provider,
                    ProviderConfig::new(GEMINI_DEFAULT),
                ),
            };
        }

        providers
    }

    pub fn extract(
        &self,
        prompt: &str,
        model: &str,
        max_tokens: u64,
        diffs: &str,
    ) -> Result<ResponseSchema> {
        match self {
            Provider::Gai => {
                // atm rig-core doesn't seem to let us build our own client
                // realistically, we don't need a lot of it
                // since we can just create our own schema per provider
                // for gemini for example, the structured output schema
                // doesn't like additionalfields so we had to get rid of
                // deny_unknown_fields.
                // but openai requires this
                //
                // also rig-core was using a tool call to have the LLM
                // create its own structured output based on their own (provider) specs
                // it would be relatively flimsly and fail for us since we're targeting
                // the cheaper models which may not generate a proper structure
                // ideally we can restrict this with our own schema
                // but whether or not we generate it with schemars
                // is going to be up to decide later

                let generator = SchemaSettings::draft2020_12()
                    .with(|s| {
                        s.meta_schema = None;
                        s.inline_subschemas = true;
                    })
                    .into_generator();

                let schema = generator
                    .into_root_schema_for::<ResponseSchema>();

                let schema_value = serde_json::to_value(&schema)?;

                #[derive(Serialize, Debug)]
                struct FromUser {
                    schema: serde_json::Value,
                    prompt: String,
                    diffs: String,
                }

                let request_body = FromUser {
                    schema: schema_value,
                    prompt: prompt.to_string(),
                    diffs: diffs.to_string(),
                };

                let auth_token = get_token()?;

                let endpoint = "https://cli.gai.fyi/generate";

                // todo move this out for reuse
                let config = Agent::config_builder()
                    .timeout_global(Some(Duration::from_secs(5)))
                    .build();

                let agent: Agent = config.into();
                let resp = agent
                    .post(endpoint)
                    .header(
                        "Authorization",
                        format!("Bearer {}", auth_token),
                    )
                    .header("Content-Type", "application/json")
                    .send_json(&request_body)?
                    .body_mut()
                    .read_json::<serde_json::Value>()?;

                let generated_text = resp
                    .get("candidates")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.get(0))
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "Invalid response format from Gemini API"
                        )
                    })?;

                let result: ResponseSchema = serde_json::from_str(
                    generated_text,
                )
                .map_err(|e| {
                    anyhow!(
                        "faield to parse JSON into vlaid schema: {}",
                        e
                    )
                })?;

                Ok(result)
            }
            Provider::OpenAI => {
                todo!()
            }
            Provider::Gemini => {
                todo!()
            }
            Provider::Claude => {
                todo!()
            }
        }
    }
}
