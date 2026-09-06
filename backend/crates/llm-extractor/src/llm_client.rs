use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: Option<String>,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
}

#[derive(Debug, Serialize)]
struct ChatTemplateKwargs {
    enable_thinking: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

impl LlmClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: None,
            // model: "Qwen3-1.7B".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.0,
            max_tokens: 500,
            chat_template_kwargs: Some(ChatTemplateKwargs {
                enable_thinking: false,
            }),
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await
            .context("failed to send request")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read llama-server response")?;

        debug!(%status, body = %body, "Received response from llama-server");

        if !status.is_success() {
            anyhow::bail!("llama-server returned {}: {}", status, body);
        }

        let response: ChatResponse =
            serde_json::from_str(&body).context("failed to parse llama-server response")?;

        let content = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("llama-server returned no choices"))?;

        if content.is_empty() {
            anyhow::bail!("llama-server returned an empty message content");
        }

        Ok(content)
    }
}
