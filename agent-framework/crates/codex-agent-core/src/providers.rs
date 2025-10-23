//! LLM provider abstraction
//!
//! This module defines the trait for LLM providers and common implementations

use crate::client::{Message, StreamEvent, ToolCall};
use crate::error::Result;
use crate::tools::ToolSpec;
use async_channel::Receiver;
use async_trait::async_trait;

/// Trait for LLM providers
///
/// Implement this trait to add support for custom LLM providers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Complete a conversation and return the assistant's response
    ///
    /// # Arguments
    /// * `messages` - Conversation history
    /// * `tools` - Optional tool specifications for function calling
    ///
    /// # Returns
    /// The assistant's response message, potentially with tool calls
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message>;

    /// Stream a conversation and return events as they arrive
    ///
    /// # Arguments
    /// * `messages` - Conversation history
    /// * `tools` - Optional tool specifications for function calling
    ///
    /// # Returns
    /// A receiver for streaming events
    async fn stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>>;

    /// Whether this provider supports tool/function calling
    fn supports_tools(&self) -> bool {
        true
    }

    /// Whether this provider supports streaming responses
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Get the model name/identifier for this provider
    fn model(&self) -> &str;
}

/// OpenAI-compatible provider implementation
pub struct OpenAiProvider {
    http_client: reqwest::Client,
    api_key: String,
    api_base_url: String,
    model: String,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        api_base_url: Option<String>,
    ) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_key: api_key.into(),
            api_base_url: api_base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use serde_json::json;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| crate::error::AgentError::ConfigError(e.to_string()))?,
        );

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.api_base_url))
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::AgentError::ClientError(format!(
                "API request failed with status {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::error::AgentError::ClientError(e.to_string()))?;

        let choice = &json["choices"][0];
        let message = &choice["message"];

        Ok(Message {
            role: message["role"]
                .as_str()
                .unwrap_or("assistant")
                .to_string(),
            content: message["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            tool_calls: message["tool_calls"].as_array().map(|calls| {
                calls
                    .iter()
                    .filter_map(|call| {
                        Some(ToolCall {
                            id: call["id"].as_str()?.to_string(),
                            r#type: call["type"].as_str()?.to_string(),
                            function: crate::client::FunctionCall {
                                name: call["function"]["name"].as_str()?.to_string(),
                                arguments: call["function"]["arguments"].as_str()?.to_string(),
                            },
                        })
                    })
                    .collect()
            }),
            tool_call_id: None,
        })
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>> {
        use async_channel::bounded;
        use eventsource_stream::Eventsource;
        use futures::StreamExt;
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use serde_json::json;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| crate::error::AgentError::ConfigError(e.to_string()))?,
        );

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.api_base_url))
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::AgentError::ClientError(format!(
                "API request failed with status {}: {}",
                status, text
            )));
        }

        let (tx, rx) = bounded(100);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream().eventsource();

            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            let _ = tx.send(StreamEvent::Complete).await;
                            break;
                        }

                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) {
                            let choice = &json["choices"][0];
                            let delta = &choice["delta"];

                            if let Some(content) = delta["content"].as_str() {
                                let _ = tx.send(StreamEvent::TextDelta(content.to_string())).await;
                            }

                            if let Some(tool_calls) = delta["tool_calls"].as_array() {
                                for call in tool_calls {
                                    if let (Some(id), Some(name)) = (
                                        call["id"].as_str(),
                                        call["function"]["name"].as_str(),
                                    ) {
                                        let _ = tx
                                            .send(StreamEvent::ToolCall(ToolCall {
                                                id: id.to_string(),
                                                r#type: "function".to_string(),
                                                function: crate::client::FunctionCall {
                                                    name: name.to_string(),
                                                    arguments: call["function"]["arguments"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string(),
                                                },
                                            }))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    fn model(&self) -> &str {
        &self.model
    }
}

/// Simple completion provider (no tools, just text)
pub struct SimpleCompletionProvider {
    inner: OpenAiProvider,
}

impl SimpleCompletionProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(api_key, model, None),
        }
    }
}

#[async_trait]
impl LlmProvider for SimpleCompletionProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message> {
        // Always pass None for tools
        self.inner.complete(messages, None).await
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>> {
        // Always pass None for tools
        self.inner.stream(messages, None).await
    }

    fn supports_tools(&self) -> bool {
        false
    }

    fn model(&self) -> &str {
        self.inner.model()
    }
}
