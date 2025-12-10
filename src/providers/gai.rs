use llmao::{Provider, extract::Extract};
use schemars::generate::SchemaSettings;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use super::{provider::ProviderError, schema::ResponseSchema};
use crate::commands::auth::get_token;

#[derive(Debug)]
pub struct GaiProvider {
    agent: ureq::Agent,

    #[allow(dead_code)]
    config: GaiConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaiConfig {
    // todo more so for the worker
    // allow for different models
    pub model: String,
}

impl Default for GaiConfig {
    fn default() -> Self {
        Self {
            model: "gemini-flash-2.5".to_owned(),
        }
    }
}

// create this as we create our request
impl GaiProvider {
    pub fn new() -> Self {
        Self {
            agent: Agent::new_with_defaults(),
            config: GaiConfig::default(),
        }
    }
}

impl Default for GaiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GaiProvider {
    type Error = ProviderError;
}

impl Extract<ResponseSchema> for GaiProvider {
    type Prompt = String;
    type Content = String;

    fn extract(
        &mut self,
        prompt: String,
        diffs: String,
    ) -> Result<ResponseSchema, ProviderError> {
        let generator = SchemaSettings::draft2020_12()
            .with(|s| {
                s.meta_schema = None;
                s.inline_subschemas = true;
            })
            .into_generator();

        let schema = serde_json::to_value(
            generator.into_root_schema_for::<ResponseSchema>(),
        )?;

        #[derive(Serialize, Debug)]
        struct FromUser {
            schema: serde_json::Value,
            prompt: String,
            diffs: String,
        }

        let request_body = FromUser {
            schema,
            prompt,
            diffs,
        };

        let endpoint = "https://cli.gai.fyi/generate";
        let auth_token = get_token()
            .map_err(|_| ProviderError::NotAuthenticated)?;

        let resp = self
            .agent
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", auth_token))
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
            .ok_or_else(|| ProviderError::NoContent)?;

        let result: ResponseSchema =
            serde_json::from_str(generated_text)
                .map_err(|_| ProviderError::InvalidSchema)?;

        Ok(result)
    }
}
