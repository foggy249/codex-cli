//! LLM client abstraction
//!
//! Provides a simplified interface to interact with Large Language Models

use crate::error::{AgentError, Result};
use crate::tools::ToolSpec;
use async_channel::{Receiver, Sender};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Response event from streaming LLM
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Text delta (partial response)
    TextDelta(String),

    /// Tool call started
    ToolCall(ToolCall),

    /// Response completed
    Complete,

    /// Error occurred
    Error(String),
}

/// Simple LLM client for OpenAI-compatible APIs
pub struct LlmClient {
    http_client: reqwest::Client,
    api_key: String,
    api_base_url: String,
    model: String,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        api_base_url: Option<String>,
    ) -> Self {
        let api_base_url = api_base_url.unwrap_or_else(|| {
            "https://api.openai.com/v1".to_string()
        });

        Self {
            http_client: reqwest::Client::new(),
            api_key: api_key.into(),
            api_base_url,
            model: model.into(),
        }
    }

    /// Layer 1: Simple text completion (no history, no tools)
    ///
    /// This is the simplest way to use the LLM client. Just provide a prompt
    /// and get back a response.
    ///
    /// # Example
    /// ```no_run
    /// # use codex_agent_core::client::LlmClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = LlmClient::new("api-key", "gpt-4", None);
    /// let response = client.complete_simple("What is 2+2?").await?;
    /// println!("{}", response); // "4"
    /// # Ok(())
    /// # }
    /// ```
    pub async fn complete_simple(&self, prompt: impl Into<String>) -> Result<String> {
        let messages = vec![Message::user(prompt)];
        let response = self.complete(messages, None).await?;
        Ok(response.content)
    }

    /// Layer 2: Chat completion (with history, no tools)
    ///
    /// Use this when you want to maintain conversation history but don't need
    /// tool calling capabilities.
    ///
    /// # Example
    /// ```no_run
    /// # use codex_agent_core::client::{LlmClient, Message};
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = LlmClient::new("api-key", "gpt-4", None);
    /// let messages = vec![
    ///     Message::user("My name is Alice"),
    ///     Message::assistant("Nice to meet you, Alice!"),
    ///     Message::user("What's my name?"),
    /// ];
    /// let response = client.complete_chat(messages).await?;
    /// println!("{}", response.content); // "Your name is Alice"
    /// # Ok(())
    /// # }
    /// ```
    pub async fn complete_chat(&self, messages: Vec<Message>) -> Result<Message> {
        self.complete(messages, None).await
    }

    /// Layer 3: Complete with tools (full features, manual loop)
    ///
    /// Use this when you want tool calling but want to control the tool
    /// execution loop yourself.
    ///
    /// # Example
    /// ```no_run
    /// # use codex_agent_core::client::{LlmClient, Message};
    /// # use codex_agent_core::tools::ToolSpec;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = LlmClient::new("api-key", "gpt-4", None);
    /// let tools = vec![/* your tool specs */];
    /// 
    /// let mut messages = vec![Message::user("What's the weather?")];
    /// 
    /// loop {
    ///     let response = client.complete_with_tools(messages.clone(), tools.clone()).await?;
    ///     
    ///     if let Some(tool_calls) = response.tool_calls {
    ///         // Handle tool calls manually
    ///         for call in tool_calls {
    ///             // Execute tool and add result to messages
    ///         }
    ///         messages.push(response);
    ///     } else {
    ///         // No more tool calls, we're done
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn complete_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolSpec>,
    ) -> Result<Message> {
        self.complete(messages, Some(tools)).await
    }

    /// Layer 4: Full completion (internal use by Agent)
    ///
    /// This is the complete API that the Agent uses. Most users should use
    /// one of the higher-level methods above.
    ///
    /// Send a completion request and get a complete response
    pub async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message> {
        let stream = self.stream(messages, tools).await?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        while let Some(event) = stream.recv().await.ok() {
            match event {
                StreamEvent::TextDelta(delta) => content.push_str(&delta),
                StreamEvent::ToolCall(call) => tool_calls.push(call),
                StreamEvent::Complete => break,
                StreamEvent::Error(err) => return Err(AgentError::ClientError(err)),
            }
        }

        Ok(Message {
            role: "assistant".to_string(),
            content,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            tool_call_id: None,
        })
    }

    /// Stream completion responses
    pub async fn stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>> {
        let (tx, rx) = async_channel::unbounded();

        let url = format!("{}/chat/completions", self.api_base_url);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| AgentError::ClientError(e.to_string()))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools
                .into_iter()
                .map(|t| json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                }))
                .collect::<Vec<_>>());
        }

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AgentError::ClientError(format!(
                "API request failed with status {status}: {text}"
            )));
        }

        // Spawn a task to handle the streaming response
        tokio::spawn(async move {
            if let Err(e) = Self::handle_stream(response, tx.clone()).await {
                let _ = tx.send(StreamEvent::Error(e.to_string())).await;
            }
        });

        Ok(rx)
    }

    async fn handle_stream(
        response: reqwest::Response,
        tx: Sender<StreamEvent>,
    ) -> Result<()> {
        let mut stream = response.bytes_stream().eventsource();

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        let _ = tx.send(StreamEvent::Complete).await;
                        break;
                    }

                    let chunk: Value = serde_json::from_str(&event.data)
                        .map_err(|e| AgentError::ClientError(e.to_string()))?;

                    if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta") {
                                // Handle content delta
                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                    if !content.is_empty() {
                                        let _ = tx.send(StreamEvent::TextDelta(content.to_string())).await;
                                    }
                                }

                                // Handle tool calls
                                if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                    for tool_call in tool_calls {
                                        if let Ok(call) = serde_json::from_value::<ToolCall>(tool_call.clone()) {
                                            let _ = tx.send(StreamEvent::ToolCall(call)).await;
                                        }
                                    }
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

        Ok(())
    }
}
