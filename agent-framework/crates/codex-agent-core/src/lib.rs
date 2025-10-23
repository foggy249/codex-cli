//! # Codex Agent Framework
//!
//! A lightweight, composable framework for building LLM-powered agents.
//!
//! This crate provides the core abstractions and implementation for creating
//! agents that can interact with Large Language Models, execute tools, and
//! manage conversations.
//!
//! ## Features
//!
//! - **Simple API**: Create agents with just a few lines of code
//! - **Streaming Support**: Real-time responses from LLMs
//! - **Tool System**: Pluggable tools for extending agent capabilities
//! - **Multiple Providers**: Support for OpenAI, Anthropic, and custom providers
//! - **Event-Driven**: Subscribe to agent events for fine-grained control
//!
//! ## Quick Start
//!
//! ```no_run
//! use codex_agent_core::{Agent, AgentConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let agent = Agent::new(
//!         AgentConfig::builder()
//!             .model("gpt-4")
//!             .api_key_from_env()
//!             .build()?
//!     ).await?;
//!     
//!     let response = agent.run("What is 2+2?").await?;
//!     println!("{}", response.text);
//!     Ok(())
//! }
//! ```

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod agent;
pub mod config;
pub mod error;
pub mod client;
pub mod protocol;
pub mod session;
pub mod tools;

// Re-export main types for convenience
pub use agent::{Agent, AgentResponse, AgentStream};
pub use config::{AgentConfig, AgentConfigBuilder};
pub use error::{AgentError, Result};
pub use protocol::{Event, EventMsg, Op};
pub use tools::{Tool, ToolContext, ToolResult};

// Re-export commonly used protocol types
pub use codex_protocol::protocol::{
    AskForApproval, ReviewDecision, SandboxPolicy,
};
