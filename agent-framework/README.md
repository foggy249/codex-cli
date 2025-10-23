# Codex Agent Framework

A lightweight, composable framework for building LLM-powered agents extracted from the Codex CLI codebase.

## Features

- **🚀 Simple API**: Create agents with just a few lines of code
- **⚡ Streaming Support**: Real-time responses from LLMs
- **🔧 Pluggable Tools**: Extend agent capabilities with custom tools
- **🌐 Multiple Providers**: Support for OpenAI, Anthropic, and custom providers
- **📡 Event-Driven**: Subscribe to agent events for fine-grained control
- **🔒 Type-Safe**: Built with Rust for reliability and performance

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
codex-agent-core = { path = "path/to/agent-framework/crates/codex-agent-core" }
codex-agent-tools = { path = "path/to/agent-framework/crates/codex-agent-tools" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Minimal Example

```rust
use codex_agent_core::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()?
    ).await?;
    
    let response = agent.run("What is 2+2?").await?;
    println!("{}", response.text);
    Ok(())
}
```

### Agent with Tools

```rust
use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .working_directory(".")
            .tools(StandardTools::code_tools())
            .build()?
    ).await?;
    
    let response = agent.run(
        "Read the README.md file and summarize it"
    ).await?;
    println!("{}", response.text);
    Ok(())
}
```

### Multi-turn Conversation

```rust
let agent = Agent::new(config).await?;

// First turn
let response1 = agent.run("What files are in this directory?").await?;
println!("{}", response1.text);

// Second turn (maintains context)
let response2 = agent.run("Read the first file you found").await?;
println!("{}", response2.text);
```

## Architecture

The framework is organized into several components:

```
┌──────────────────────────────────────┐
│         Agent API                    │  ← High-level interface
├──────────────────────────────────────┤
│      Session Management              │  ← Conversation state
├──────────────────────────────────────┤
│       LLM Client                     │  ← Provider abstraction
├──────────────────────────────────────┤
│       Tool System                    │  ← Pluggable capabilities
├──────────────────────────────────────┤
│    Protocol & Events                 │  ← Type-safe messages
└──────────────────────────────────────┘
```

## Core Concepts

### Agent

The main entry point for interacting with the framework. An agent manages:
- LLM client connection
- Conversation history
- Tool registry
- Configuration

### Tools

Tools are functions that the LLM can call to perform actions:

```rust
use async_trait::async_trait;
use codex_agent_core::{Tool, ToolContext, ToolResult};

struct CustomTool;

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> &str {
        "custom_tool"
    }
    
    fn description(&self) -> &str {
        "A custom tool that does something useful"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "The input parameter"
                }
            },
            "required": ["input"]
        })
    }
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        // Your custom logic here
        Ok(ToolResult::text("Result"))
    }
}

// Register with agent
let agent = Agent::new(
    AgentConfig::builder()
        .model("gpt-4")
        .api_key_from_env()
        .tool(CustomTool)
        .build()?
).await?;
```

### Configuration

Agents are configured using a builder pattern:

```rust
let config = AgentConfig::builder()
    // Required
    .model("gpt-4")
    .api_key("sk-...")  // or .api_key_from_env()
    
    // Optional
    .working_directory(".")
    .approval_policy(AskForApproval::Never)
    .sandbox_policy(SandboxPolicy::WorkspaceWrite)
    .tool(CustomTool)
    .tools(StandardTools::all())
    .build()?;
```

## Standard Tools

The `codex-agent-tools` crate provides several built-in tools:

### Shell Tool
Execute shell commands:
```rust
agent.run("List all Python files in this directory").await?;
```

### File Operations
Read and write files:
```rust
agent.run("Read the config.toml file").await?;
agent.run("Create a new file called README.md with some content").await?;
```

### Tool Collections
Pre-configured tool sets:
```rust
StandardTools::all()         // All standard tools
StandardTools::code_tools()  // Code-related tools
StandardTools::read_only()   // Read-only tools
```

## Examples

The `examples/` directory contains complete working examples:

1. **minimal**: Bare minimum agent (10 lines)
2. **code_assistant**: Agent with file and shell tools
3. **chat_bot**: Interactive chat with history

Run examples with:
```bash
export OPENAI_API_KEY=sk-...
cargo run -p minimal-example
cargo run -p code-assistant-example
cargo run -p chat-bot-example
```

## Advanced Features

### Streaming Responses

```rust
let mut stream = agent.run_streaming("Explain quantum computing").await?;

while let Some(event) = stream.next().await {
    match event {
        AgentEvent::TextDelta(delta) => print!("{}", delta),
        AgentEvent::ToolExecution(name) => println!("\nExecuting: {}", name),
        AgentEvent::Complete(response) => break,
    }
}
```

### Approval Policies

Control when user approval is required for tool execution:

```rust
use codex_agent_core::AskForApproval;

AgentConfig::builder()
    .approval_policy(AskForApproval::Untrusted)  // Always ask
    .approval_policy(AskForApproval::OnFailure)  // Ask if command fails
    .approval_policy(AskForApproval::OnRequest)  // Ask if agent requests
    .approval_policy(AskForApproval::Never)      // Never ask (⚠️ dangerous)
```

### Sandbox Policies

Restrict what tools can access:

```rust
use codex_agent_core::SandboxPolicy;

AgentConfig::builder()
    .sandbox_policy(SandboxPolicy::ReadOnly)           // Read-only access
    .sandbox_policy(SandboxPolicy::WorkspaceWrite)     // Write in workspace
    .sandbox_policy(SandboxPolicy::DangerFullAccess)   // Full access (⚠️ dangerous)
```

### Session Management

```rust
// Get session ID
let session_id = agent.session_id();

// Get message history
let messages = agent.messages().await;

// Reset conversation
agent.reset().await;
```

## Comparison with Full Codex CLI

This framework extracts the core agent capabilities while removing:

- ❌ Terminal UI (TUI)
- ❌ CLI argument parsing
- ❌ ChatGPT OAuth authentication
- ❌ Platform-specific sandboxing (Seatbelt, Landlock)
- ❌ Telemetry and observability
- ❌ Session persistence to disk
- ❌ MCP server implementation

What's included:

- ✅ Core agent logic
- ✅ LLM client (OpenAI-compatible)
- ✅ Tool system
- ✅ Event protocol
- ✅ Basic configuration
- ✅ Standard tools (shell, file ops)
- ✅ API key authentication
- ✅ Multi-turn conversations

## API Documentation

### Agent

```rust
impl Agent {
    pub async fn new(config: AgentConfig) -> Result<Self>;
    pub async fn run(&self, message: impl Into<String>) -> Result<AgentResponse>;
    pub async fn run_streaming(&self, message: impl Into<String>) -> Result<AgentStream>;
    pub fn session_id(&self) -> &str;
    pub async fn messages(&self) -> Vec<Message>;
    pub async fn reset(&self);
}
```

### AgentConfig

```rust
impl AgentConfigBuilder {
    pub fn model(self, model: impl Into<String>) -> Self;
    pub fn api_key(self, key: impl Into<String>) -> Self;
    pub fn api_key_from_env(self) -> Self;
    pub fn api_base_url(self, url: impl Into<String>) -> Self;
    pub fn working_directory(self, dir: impl Into<PathBuf>) -> Self;
    pub fn approval_policy(self, policy: AskForApproval) -> Self;
    pub fn sandbox_policy(self, policy: SandboxPolicy) -> Self;
    pub fn tool(self, tool: impl Tool + 'static) -> Self;
    pub fn tools(self, tools: impl IntoIterator<Item = Box<dyn Tool>>) -> Self;
    pub fn build(self) -> Result<AgentConfig>;
}
```

### Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, args: Value, ctx: &ToolContext) -> anyhow::Result<ToolResult>;
    fn requires_approval(&self, ctx: &ToolContext) -> bool { false }
}
```

## Design Principles

1. **Simplicity**: Easy to get started with minimal boilerplate
2. **Composability**: Mix and match components as needed
3. **Type Safety**: Leverage Rust's type system for correctness
4. **Flexibility**: Support multiple LLM providers and custom tools
5. **Performance**: Async/await with efficient streaming

## Limitations

- **API Key Only**: No ChatGPT OAuth support (use API keys)
- **Basic Sandboxing**: No platform-specific sandboxing (yet)
- **OpenAI-Compatible**: Designed for OpenAI-compatible APIs
- **No Persistence**: Sessions are in-memory only
- **Simplified Approval**: Basic approval system (no UI integration)

## Development

### Build

```bash
cd agent-framework
cargo build
```

### Test

```bash
cargo test
```

### Run Examples

```bash
export OPENAI_API_KEY=sk-...
cargo run -p minimal-example
```

### Format

```bash
cargo fmt
```

## Contributing

Contributions are welcome! Please:

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Keep changes focused and surgical

## License

Apache-2.0 (same as Codex CLI)

## Acknowledgments

This framework is extracted from the [Codex CLI](https://github.com/openai/codex) codebase developed by OpenAI. It preserves the core agent architecture while providing a simplified, reusable interface.

## Related Projects

- [Codex CLI](https://github.com/openai/codex) - Full-featured coding agent
- [OpenAI Platform](https://platform.openai.com/) - LLM APIs
- [Model Context Protocol](https://modelcontextprotocol.io/) - Agent extensibility standard
