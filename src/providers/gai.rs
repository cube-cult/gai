use llmao::{Provider, extract::Extract};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use ureq::Agent;

use crate::cmd::auth::get_token;

use super::provider::ProviderError;

#[derive(Debug)]
pub struct GaiProvider {
    agent: ureq::Agent,

    #[allow(dead_code)]
    config: GaiConfig,
    schema: Option<Value>,
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
            schema: None,
        }
    }

    /// insert schema
    pub fn schema(
        mut self,
        schema: Value,
    ) -> Self {
        self.schema = Some(schema);
        self
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

impl<T> Extract<T> for GaiProvider
where
    T: DeserializeOwned,
{
    type Prompt = String;
    type Content = String;

    fn extract(
        &mut self,
        prompt: String,
        content: String,
    ) -> Result<T, ProviderError> {
        /// json struct, used when deserializing
        /// on server
        #[derive(Serialize, Debug)]
        struct FromUser {
            schema: serde_json::Value,
            prompt: String,
            content: String,
        }

        let schema = match &self.schema {
            Some(s) => s.to_owned(),
            None => return Err(ProviderError::InvalidSchema),
        };

        let request_body = FromUser {
            schema,
            prompt,
            content,
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

        let result: T = serde_json::from_str(generated_text)
            .map_err(|_| ProviderError::InvalidSchema)?;

        Ok(result)
    }
}
