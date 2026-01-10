use serde::Serialize;
use std::fmt;

/// request struct
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// system prompt
    /// or the main prompt
    /// this should have specific instructions
    /// for this specific request
    pub system: String,

    /// ContentPart array
    content: Vec<ContentPart>,
}

/// content arrays for the request body
/// in openai's case, this would be part of
/// input: { "role": "user", "content": this here }
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentPart {
    /// at the moment, since we're handling
    /// diffs, this should just be text ala string
    Text { text: String },
}

impl fmt::Display for Request {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        writeln!(f, "System: {}", self.system)?;
        writeln!(f, "Content:")?;

        for (i, part) in self
            .content
            .iter()
            .enumerate()
        {
            match part {
                ContentPart::Text { text } => {
                    writeln!(f, "[{}] {}", i, text)?;
                }
            }
        }

        Ok(())
    }
}

impl Request {
    /// create a new request
    /// with an existing system prompt
    pub fn new(system: &str) -> Self {
        Self {
            system: system.to_owned(),
            content: Vec::new(),
        }
    }

    /// insert a single content text entry
    pub fn insert_content(
        mut self,
        text: &str,
    ) -> Self {
        self.content
            .push(ContentPart::Text {
                text: text.to_owned(),
            });
        self
    }

    /// inserts an entire vector of new text entries
    /// to contents array
    pub fn insert_contents(
        mut self,
        texts: &[String],
    ) -> Request {
        self.content.extend(
            texts
                .iter()
                .map(|t| ContentPart::Text { text: t.to_owned() }),
        );

        self
    }

    /// convert content part array to json
    /// will fail if invalid (shouldnt fail)
    pub fn get_content_as_json(
        &self
    ) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.content)?)
    }

    /// convert cntent part array to a string
    pub fn get_content_as_str(&self) -> String {
        let mut s = String::new();

        for part in &self.content {
            match part {
                ContentPart::Text { text } => {
                    if !text.is_empty() {
                        let text = format!("{text}\n");
                        s.push_str(&text);
                    }
                }
            }
        }

        s
    }
}
