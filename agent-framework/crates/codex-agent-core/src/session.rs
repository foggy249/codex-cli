//! Session management for agent conversations

use crate::client::{LlmClient, Message};
use crate::config::AgentConfig;
use crate::error::{AgentError, Result};
use crate::tools::{ToolContext, ToolRegistry};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Agent session managing a single conversation
pub struct Session {
    /// Session ID
    pub id: String,

    /// LLM client
    client: Arc<LlmClient>,

    /// Tool registry
    tools: Arc<ToolRegistry>,

    /// Configuration
    config: Arc<AgentConfig>,

    /// Message history
    messages: Arc<Mutex<Vec<Message>>>,

    /// System instructions
    system_prompt: String,
}

impl Session {
    /// Create a new session
    pub fn new(mut config: AgentConfig) -> Self {
        let client = Arc::new(LlmClient::new(
            config.api_key.clone(),
            config.model.clone(),
            config.api_base_url.clone(),
        ));

        // Take ownership of tools to avoid clone
        let tools_vec = std::mem::take(&mut config.tools);
        let tools = Arc::new(ToolRegistry::with_tools(tools_vec));

        let system_prompt = Self::build_system_prompt(&config);

        let messages = Arc::new(Mutex::new(vec![Message::system(system_prompt.clone())]));

        Self {
            id: Uuid::new_v4().to_string(),
            client,
            tools,
            config: Arc::new(config),
            messages,
            system_prompt,
        }
    }

    /// Get the session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the current message history
    pub async fn messages(&self) -> Vec<Message> {
        self.messages.lock().await.clone()
    }

    /// Add a user message and get the assistant's response
    pub async fn run(&self, user_message: impl Into<String>) -> Result<String> {
        let user_msg = Message::user(user_message);

        // Add user message to history
        {
            let mut msgs = self.messages.lock().await;
            msgs.push(user_msg.clone());
        }

        // Get response from LLM
        loop {
            let messages = self.messages.lock().await.clone();
            let tool_specs = if !self.tools.all().is_empty() {
                Some(self.tools.to_tool_specs())
            } else {
                None
            };

            let response = self.client.complete(messages, tool_specs).await?;

            // Check if there are tool calls
            if let Some(tool_calls) = &response.tool_calls {
                // Add assistant message with tool calls
                {
                    let mut msgs = self.messages.lock().await;
                    msgs.push(response.clone());
                }

                // Execute tools
                for tool_call in tool_calls {
                    let tool_name = &tool_call.function.name;
                    let tool_args: serde_json::Value =
                        serde_json::from_str(&tool_call.function.arguments)
                            .map_err(|e| AgentError::ToolError(format!("Invalid tool arguments: {e}")))?;

                    let result = if let Some(tool) = self.tools.get(tool_name) {
                        let ctx = ToolContext {
                            cwd: self.config.working_directory.clone(),
                            approval_policy: self.config.approval_policy,
                            sandbox_policy: self.config.sandbox_policy.clone(),
                            call_id: tool_call.id.clone(),
                        };

                        match tool.execute(tool_args, &ctx).await {
                            Ok(result) => result.as_text(),
                            Err(e) => format!("Error executing tool: {e}"),
                        }
                    } else {
                        format!("Tool '{tool_name}' not found")
                    };

                    // Add tool result to history
                    {
                        let mut msgs = self.messages.lock().await;
                        msgs.push(Message::tool_result(&tool_call.id, result));
                    }
                }

                // Continue loop to get next response
                continue;
            }

            // No tool calls, return the response
            {
                let mut msgs = self.messages.lock().await;
                msgs.push(response.clone());
            }

            return Ok(response.content);
        }
    }

    /// Build the system prompt
    fn build_system_prompt(_config: &AgentConfig) -> String {
        // Build a simple system prompt
        // Note: tools are automatically added by the LLM client, not in the system prompt
        let mut prompt = String::from(
            "You are a helpful AI assistant. You can use tools to help accomplish tasks.\n\n"
        );

        // Add working directory context
        prompt.push_str(&format!(
            "Working directory: {}\n",
            _config.working_directory.display()
        ));

        prompt
    }

    /// Reset the conversation
    pub async fn reset(&self) {
        let mut msgs = self.messages.lock().await;
        msgs.clear();
        msgs.push(Message::system(self.system_prompt.clone()));
    }
}
