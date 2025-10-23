//! Agent configuration

use std::env;
use std::path::PathBuf;
use crate::error::{AgentError, Result};
use crate::tools::Tool;
use codex_protocol::config_types::{ReasoningEffort, ReasoningSummary};
use codex_protocol::protocol::{AskForApproval, SandboxPolicy};

/// Configuration for creating an agent
pub struct AgentConfig {
    /// LLM model to use (e.g., "gpt-4", "claude-3-sonnet")
    pub model: String,

    /// API key for authentication
    pub api_key: String,

    /// Optional API base URL for custom providers
    pub api_base_url: Option<String>,

    /// Model provider (auto-detected from model name if not specified)
    pub provider: Option<String>,

    /// Working directory for file operations
    pub working_directory: PathBuf,

    /// Approval policy for tool execution
    pub approval_policy: AskForApproval,

    /// Sandbox policy for tool execution
    pub sandbox_policy: SandboxPolicy,

    /// Reasoning effort (for reasoning-capable models)
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Reasoning summary preference
    pub reasoning_summary: ReasoningSummary,

    /// Maximum tokens in context (optional, auto-detected from model)
    pub max_context_tokens: Option<i64>,

    /// Tools available to the agent
    pub(crate) tools: Vec<Box<dyn Tool>>,
}

impl AgentConfig {
    /// Create a new builder for agent configuration
    pub fn builder() -> AgentConfigBuilder {
        AgentConfigBuilder::new()
    }
}

/// Builder for agent configuration
pub struct AgentConfigBuilder {
    model: Option<String>,
    api_key: Option<String>,
    api_base_url: Option<String>,
    provider: Option<String>,
    working_directory: Option<PathBuf>,
    approval_policy: AskForApproval,
    sandbox_policy: SandboxPolicy,
    reasoning_effort: Option<ReasoningEffort>,
    reasoning_summary: ReasoningSummary,
    max_context_tokens: Option<i64>,
    tools: Vec<Box<dyn Tool>>,
}

impl AgentConfigBuilder {
    fn new() -> Self {
        Self {
            model: None,
            api_key: None,
            api_base_url: None,
            provider: None,
            working_directory: None,
            approval_policy: AskForApproval::UnlessTrusted,
            sandbox_policy: SandboxPolicy::WorkspaceWrite {
                writable_roots: vec![],
                network_access: false,
                exclude_tmpdir_env_var: false,
                exclude_slash_tmp: false,
            },
            reasoning_effort: None,
            reasoning_summary: ReasoningSummary::Auto,
            max_context_tokens: None,
            tools: Vec::new(),
        }
    }

    /// Set the model name (e.g., "gpt-4", "claude-3-sonnet")
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the API key directly
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Load API key from environment variable (OPENAI_API_KEY or ANTHROPIC_API_KEY)
    pub fn api_key_from_env(mut self) -> Self {
        // Try common environment variables
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            self.api_key = Some(key);
        } else if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            self.api_key = Some(key);
        } else if let Ok(key) = env::var("API_KEY") {
            self.api_key = Some(key);
        }
        self
    }

    /// Set custom API base URL
    pub fn api_base_url(mut self, url: impl Into<String>) -> Self {
        self.api_base_url = Some(url.into());
        self
    }

    /// Set the provider explicitly (otherwise auto-detected from model name)
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Set the working directory
    pub fn working_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set the approval policy
    pub fn approval_policy(mut self, policy: AskForApproval) -> Self {
        self.approval_policy = policy;
        self
    }

    /// Set the sandbox policy
    pub fn sandbox_policy(mut self, policy: SandboxPolicy) -> Self {
        self.sandbox_policy = policy;
        self
    }

    /// Set reasoning effort (for reasoning-capable models)
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Set reasoning summary preference
    pub fn reasoning_summary(mut self, summary: ReasoningSummary) -> Self {
        self.reasoning_summary = summary;
        self
    }

    /// Set maximum context tokens
    pub fn max_context_tokens(mut self, tokens: i64) -> Self {
        self.max_context_tokens = Some(tokens);
        self
    }

    /// Add a tool to the agent
    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    /// Add multiple tools to the agent
    pub fn tools(mut self, tools: impl IntoIterator<Item = Box<dyn Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Build the agent configuration
    pub fn build(self) -> Result<AgentConfig> {
        let model = self.model.ok_or_else(|| {
            AgentError::ConfigError("Model must be specified".to_string())
        })?;

        let api_key = self.api_key.ok_or_else(|| {
            AgentError::ConfigError(
                "API key must be specified (use api_key() or api_key_from_env())".to_string()
            )
        })?;

        let working_directory = self.working_directory.unwrap_or_else(|| {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        Ok(AgentConfig {
            model,
            api_key,
            api_base_url: self.api_base_url,
            provider: self.provider,
            working_directory,
            approval_policy: self.approval_policy,
            sandbox_policy: self.sandbox_policy,
            reasoning_effort: self.reasoning_effort,
            reasoning_summary: self.reasoning_summary,
            max_context_tokens: self.max_context_tokens,
            tools: self.tools,
        })
    }
}
