//! Tool system for agent extensibility

use std::path::PathBuf;
use async_trait::async_trait;
use serde_json::Value;
use codex_protocol::protocol::{AskForApproval, SandboxPolicy};

/// Context provided to tools during execution
pub struct ToolContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Approval policy for this tool execution
    pub approval_policy: AskForApproval,

    /// Sandbox policy for this tool execution
    pub sandbox_policy: SandboxPolicy,

    /// Call ID for this specific invocation
    pub call_id: String,
}

/// Result of tool execution
#[derive(Debug, Clone)]
pub enum ToolResult {
    /// Simple text result
    Text(String),

    /// Structured JSON result
    Json(Value),

    /// Result with additional metadata
    Complex {
        text: String,
        metadata: Value,
    },

    /// Error result
    Error(String),
}

impl ToolResult {
    /// Create a text result
    pub fn text(content: impl Into<String>) -> Self {
        ToolResult::Text(content.into())
    }

    /// Create a JSON result
    pub fn json(value: Value) -> Self {
        ToolResult::Json(value)
    }

    /// Create an error result
    pub fn error(msg: impl Into<String>) -> Self {
        ToolResult::Error(msg.into())
    }

    /// Get the text representation
    pub fn as_text(&self) -> String {
        match self {
            ToolResult::Text(s) => s.clone(),
            ToolResult::Json(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            ToolResult::Complex { text, .. } => text.clone(),
            ToolResult::Error(e) => format!("Error: {e}"),
        }
    }

    /// Check if this is an error result
    pub fn is_error(&self) -> bool {
        matches!(self, ToolResult::Error(_))
    }
}

/// Trait for implementing agent tools
///
/// Tools are functions that the LLM can call to perform actions or retrieve information.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name for this tool (used by LLM to invoke it)
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does
    fn description(&self) -> &str;

    /// JSON schema for the tool's parameters
    ///
    /// Should follow the JSON Schema specification.
    /// Return `serde_json::json!({})` for tools with no parameters.
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with the given arguments
    ///
    /// # Arguments
    /// * `args` - JSON object containing the tool parameters
    /// * `ctx` - Context about the current execution environment
    ///
    /// # Returns
    /// A `ToolResult` containing the result of the execution
    async fn execute(
        &self,
        args: Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult>;

    /// Whether this tool requires user approval before execution
    ///
    /// Can be overridden based on tool-specific logic.
    /// Default implementation respects the approval policy in the context.
    fn requires_approval(&self, _ctx: &ToolContext) -> bool {
        false
    }
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Create a registry with the given tools
    pub fn with_tools(tools: Vec<Box<dyn Tool>>) -> Self {
        Self { tools }
    }

    /// Add a tool to the registry
    pub fn add(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|b| b.as_ref())
    }

    /// Get all tools
    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    /// Get tool specifications for LLM
    pub fn to_tool_specs(&self) -> Vec<ToolSpec> {
        self.tools
            .iter()
            .map(|tool| ToolSpec {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters_schema(),
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool specification for LLM API
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
