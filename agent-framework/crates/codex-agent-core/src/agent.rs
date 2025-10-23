//! Main Agent API

use crate::config::AgentConfig;
use crate::error::Result;
use crate::session::Session;
use async_channel::Receiver;
use std::sync::Arc;

/// High-level agent interface
pub struct Agent {
    session: Arc<Session>,
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let session = Arc::new(Session::new(config));
        Ok(Self { session })
    }

    /// Run a single turn with the agent
    ///
    /// This sends a message to the agent and waits for the complete response,
    /// including any tool executions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use codex_agent_core::{Agent, AgentConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// let agent = Agent::new(
    ///     AgentConfig::builder()
    ///         .model("gpt-4")
    ///         .api_key("sk-...")
    ///         .build()?
    /// ).await?;
    ///
    /// let response = agent.run("What is 2+2?").await?;
    /// println!("Response: {}", response.text);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self, message: impl Into<String>) -> Result<AgentResponse> {
        let text = self.session.run(message).await?;
        Ok(AgentResponse { text })
    }

    /// Run a turn with streaming responses
    ///
    /// This returns a stream of events as the agent generates its response.
    /// Useful for showing progress to users or handling tool calls incrementally.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use codex_agent_core::{Agent, AgentConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let agent = Agent::new(AgentConfig::builder().model("gpt-4").api_key("sk-...").build()?).await?;
    /// let mut stream = agent.run_streaming("Explain quantum computing").await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         AgentEvent::TextDelta(delta) => print!("{}", delta),
    ///         AgentEvent::ToolExecution(name) => println!("\nExecuting tool: {}", name),
    ///         AgentEvent::Complete(response) => {
    ///             println!("\nFinal: {}", response.text);
    ///             break;
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_streaming(&self, message: impl Into<String>) -> Result<AgentStream> {
        // For now, this is a simplified implementation
        // A full implementation would stream through the entire turn including tool calls
        let response = self.run(message).await?;
        
        let (tx, rx) = async_channel::unbounded();
        
        // Send the complete response as an event
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::Complete(response)).await;
        });
        
        Ok(AgentStream { rx })
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        self.session.id()
    }

    /// Get the current message history
    pub async fn messages(&self) -> Vec<crate::client::Message> {
        self.session.messages().await
    }

    /// Reset the conversation
    pub async fn reset(&self) {
        self.session.reset().await;
    }
}

/// Response from the agent
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// The text response from the agent
    pub text: String,
}

/// Streaming events from the agent
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Partial text response
    TextDelta(String),

    /// Tool is being executed
    ToolExecution(String),

    /// Complete response
    Complete(AgentResponse),
}

/// Stream of agent events
pub struct AgentStream {
    rx: Receiver<AgentEvent>,
}

impl AgentStream {
    /// Get the next event from the stream
    pub async fn next(&mut self) -> Option<AgentEvent> {
        self.rx.recv().await.ok()
    }

    /// Collect all events until completion
    pub async fn collect(mut self) -> Result<AgentResponse> {
        while let Some(event) = self.next().await {
            if let AgentEvent::Complete(response) = event {
                return Ok(response);
            }
        }
        Err(crate::error::AgentError::SessionError(
            "Stream ended without completion".to_string()
        ))
    }
}
