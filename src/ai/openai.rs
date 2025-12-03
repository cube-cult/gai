use llmao::{Provider, extract::Extract};
use schemars::{
    Schema, generate::SchemaSettings, transform::RecursiveTransform,
};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::ai::{provider::ProviderError, response::ResponseSchema};

#[derive(Debug)]
pub struct OpenAIProvider {
    agent: ureq::Agent,

    #[allow(dead_code)]
    config: OpenAIConfig,
    api_key: String,
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
        }
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

impl Extract<ResponseSchema> for OpenAIProvider {
    type Prompt = String;
    type Content = String;

    fn extract(
        &mut self,
        prompt: String,
        diffs: String,
    ) -> Result<ResponseSchema, ProviderError> {
        let generator = SchemaSettings::default()
            .for_serialize()
            .with(|s| {
                s.meta_schema = None;
                s.inline_subschemas = true;
            })
            .with_transform(RecursiveTransform(
                |schema: &mut Schema| {
                    if schema.get("properties").is_some() {
                        schema.insert(
                            "additionalProperties".to_owned(),
                            false.into(),
                        );
                    }
                },
            ))
            .into_generator();

        let schema = serde_json::to_value(
            generator.into_root_schema_for::<ResponseSchema>(),
        )?;

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
                    "content": diffs
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

        let extracted: ResponseSchema =
            serde_json::from_str(content)?;

        Ok(extracted)
    }
}
