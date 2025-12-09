use llmao::extract::{Error, ErrorKind, Extract};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display, EnumIter};

use crate::ai::{
    gai::{GaiConfig, GaiProvider},
    gemini::{GeminiConfig, GeminiProvider},
    openai::{OpenAIConfig, OpenAIProvider},
    schema::ResponseSchema,
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
pub enum ProviderKind {
    OpenAI,
    Gemini,
    Claude,
    Gai,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProviderConfigs {
    pub gai: GaiConfig,
    pub openai: OpenAIConfig,
    pub gemini: GeminiConfig,
}

#[derive(Debug)]
pub enum ProviderError {
    HttpError(ureq::Error),
    ParseError(serde_json::Error),
    NoContent,
    InvalidSchema,
    NotAuthenticated,
}

impl Display for ProviderError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ProviderError::HttpError(e) => {
                write!(f, "HTTP error: {}", e)
            }
            ProviderError::ParseError(e) => {
                write!(f, "Parse error: {}", e)
            }
            ProviderError::NoContent => {
                write!(f, "No content in response")
            }
            ProviderError::InvalidSchema => {
                write!(f, "Invalid schema")
            }
            ProviderError::NotAuthenticated => {
                write!(f, "Not authenticated")
            }
        }
    }
}

impl Error for ProviderError {
    fn kind(&self) -> ErrorKind {
        match self {
            ProviderError::NoContent => ErrorKind::NoData,
            ProviderError::ParseError(_) => {
                ErrorKind::DeserializationFailed
            }
            ProviderError::InvalidSchema => ErrorKind::BadSchema,
            _ => ErrorKind::NoData,
        }
    }
}

impl From<ureq::Error> for ProviderError {
    fn from(e: ureq::Error) -> Self {
        ProviderError::HttpError(e)
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(e: serde_json::Error) -> Self {
        ProviderError::ParseError(e)
    }
}

pub fn extract_from_provider(
    provider: &ProviderKind,
    prompt: &str,
    diffs: &str,
) -> Result<ResponseSchema, ProviderError> {
    match provider {
        ProviderKind::Gai => {
            let mut gai = GaiProvider::new();
            gai.extract(prompt.to_owned(), diffs.to_owned())
        }
        ProviderKind::OpenAI => {
            let mut openai = OpenAIProvider::new();
            openai.extract(prompt.to_owned(), diffs.to_owned())
        }
        ProviderKind::Gemini => {
            let mut gemini = GeminiProvider::new();
            gemini.extract(prompt.to_owned(), diffs.to_owned())
        }
        _ => todo!(),
    }
}
