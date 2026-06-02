use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config::LlmConfig, models::Example};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnrichedWord {
    pub word: String,
    pub translation: String,
    pub examples: Vec<Example>,
}

#[async_trait]
pub trait WordEnricher: Send + Sync {
    async fn enrich_word(&self, word: &str, language: &str) -> Result<EnrichedWord>;
}

#[derive(Clone)]
pub struct OpenAiCompatibleEnricher {
    client: reqwest::Client,
    config: LlmConfig,
}

impl OpenAiCompatibleEnricher {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }
}

#[async_trait]
impl WordEnricher for OpenAiCompatibleEnricher {
    async fn enrich_word(&self, word: &str, language: &str) -> Result<EnrichedWord> {
        let prompt = format!(
            r#"You are a vocabulary assistant. Analyze the {language} word "{word}".

Rules for word forms:
- If the input is a conjugated verb or plural noun, set "word" to the base form (lemma).
- For IRREGULAR forms only, append the conjugation pattern after translation.
- For REGULAR forms, do not mention any rule.

Return a valid JSON object:
{{
  "word": "base form",
  "translation": "/IPA/ Chinese translation",
  "examples": [
    {{"sentence": "Example in {language}", "translation": "Chinese translation"}},
    {{"sentence": "Example in {language}", "translation": "Chinese translation"}}
  ]
}}

Requirements:
- Include IPA phonetic transcription at the start of translation.
- Provide at least 2 examples, preferably related to daily life or programming/software engineering.
- Keep translation concise."#
        );

        let base_url = self.config.base_url.trim_end_matches('/');
        let response: serde_json::Value = self
            .client
            .post(format!("{base_url}/chat/completions"))
            .bearer_auth(&self.config.api_key)
            .json(&json!({
                "model": self.config.model,
                "messages": [{"role": "user", "content": prompt}],
                "response_format": {"type": "json_object"}
            }))
            .send()
            .await
            .context("failed to call LLM endpoint")?
            .error_for_status()
            .context("LLM endpoint returned an error")?
            .json()
            .await
            .context("failed to decode LLM response")?;

        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .context("LLM response did not include choices[0].message.content")?;
        serde_json::from_str(content).context("LLM content was not valid enriched-word JSON")
    }
}
