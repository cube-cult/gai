use llmao::{Provider, extract::Extract};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use ureq::Agent;

use super::provider::ProviderError;

#[derive(Debug)]
pub struct OpenAIProvider {
    agent: ureq::Agent,

    config: OpenAIConfig,
    api_key: String,

    schema: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub model: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            model: "gpt-5-nano".to_owned(),
        }
    }
}

// create this as we create our request
impl OpenAIProvider {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();

        Self {
            agent: Agent::new_with_defaults(),
            config: OpenAIConfig::default(),
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

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for OpenAIProvider {
    type Error = ProviderError;
}

impl<T> Extract<T> for OpenAIProvider
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

        /* println!(
            "{}",
            serde_json::to_string_pretty(&schema).unwrap()
        ); */

        let request_body = serde_json::json!({
            "model": self.config.model,
            "input": [
                {
                    "role": "system",
                    "content": prompt
                },
                {
                    "role": "user",
                    "content": content
                }
            ],
            "text": {
                "format": {
                    "type": "json_schema",
                    "name": "response_schema",
                    "schema": schema,
                    "strict": true
                }
            }
        });

        /* println!(
            "{}",
            serde_json::to_string_pretty(&request_body).unwrap()
        ); */

        let response = self
            .agent
            .post("https://api.openai.com/v1/responses")
            .header(
                "Authorization",
                &format!("Bearer {}", self.api_key),
            )
            .header("Content-Type", "application/json")
            .send_json(&request_body)?
            .body_mut()
            .read_json()?;

        // converting the response into a valid serde_json Value
        let response_json: serde_json::Value = response;

        //println!("{:#}", response_json);

        // extract the content from the OpenAI api response format
        // https://platform.openai.com/docs/guides/structured-outputs
        let content =
            response_json["output"][1]["content"][0]["text"]
                .as_str()
                .ok_or(ProviderError::NoContent)?;

        //println!("content:\n{:#?}", content);

        let extracted: T = serde_json::from_str(content)?;

        Ok(extracted)
    }
}
