use llmao::{Provider, extract::Extract};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use ureq::Agent;

use super::provider::ProviderError;

#[derive(Debug)]
pub struct GeminiProvider {
    agent: ureq::Agent,

    config: GeminiConfig,
    api_key: String,

    schema: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub model: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: "gemini-2.5-flash".to_owned(),
        }
    }
}

// create this as we create our request
impl GeminiProvider {
    pub fn new() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap();

        Self {
            agent: Agent::new_with_defaults(),
            config: GeminiConfig::default(),
            api_key,
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

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GeminiProvider {
    type Error = ProviderError;
}

impl<T> Extract<T> for GeminiProvider
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
        let schema = match &self.schema {
            Some(s) => s.to_owned(),
            None => return Err(ProviderError::InvalidSchema),
        };

        let text = format!("{}\n\n{}", prompt, content);

        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": text
                }]
            }],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": schema,
            }
        });

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.config.model
        );

        let response = self
            .agent
            .post(endpoint)
            .header(
                "x-goog-api-key",
                self.api_key
                    .to_owned(),
            )
            .header("Content-Type", "application/json")
            .send_json(&request_body)?
            .body_mut()
            .read_json()?;

        // converting the response into a valid serde_json Value
        let response_json: serde_json::Value = response;

        let generated_text = response_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| ProviderError::NoContent)?;

        let extracted: T = serde_json::from_str(generated_text)?;

        Ok(extracted)
    }
}
