# Codex Agent Framework - Complete Usage Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Your First Agent](#your-first-agent)
4. [Core Concepts](#core-concepts)
5. [Configuration](#configuration)
6. [Working with Tools](#working-with-tools)
7. [Advanced Usage](#advanced-usage)
8. [Best Practices](#best-practices)
9. [Examples](#examples)
10. [API Reference](#api-reference)

---

## Getting Started

### Prerequisites

- **Rust**: Edition 2024 or later
- **OpenAI API Key**: Get one from [OpenAI Platform](https://platform.openai.com/)
- **Tokio**: Async runtime (included as dependency)

### Quick Start (5 Minutes)

```bash
# 1. Set your API key
export OPENAI_API_KEY=sk-...

# 2. Add the framework to your project
cd your-project
# (See installation section below for Cargo.toml setup)

# 3. Create a simple agent
cargo new --bin my-agent
cd my-agent
# Add dependencies (see below)

# 4. Run your first agent
cargo run
```

---

## Installation

### Option 1: Local Path (Development)

Add to your `Cargo.toml`:

```toml
[dependencies]
codex-agent-core = { path = "path/to/agent-framework/crates/codex-agent-core" }
codex-agent-tools = { path = "path/to/agent-framework/crates/codex-agent-tools" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Option 2: Git Dependency

```toml
[dependencies]
codex-agent-core = { git = "https://github.com/your-org/codex-cli", path = "agent-framework/crates/codex-agent-core" }
codex-agent-tools = { git = "https://github.com/your-org/codex-cli", path = "agent-framework/crates/codex-agent-tools" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

---

## Your First Agent

### Minimal Example (15 lines)

```rust
use codex_agent_core::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the agent
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env() // Reads OPENAI_API_KEY
            .build()?
    ).await?;
    
    // Ask a question
    let response = agent.run("What is 2+2?").await?;
    println!("Answer: {}", response.text);
    
    Ok(())
}
```

**Output:**
```
Answer: 2+2 equals 4.
```

### With Tools (File Access)

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
        "Read the README.md file and tell me what this project does"
    ).await?;
    
    println!("{}", response.text);
    Ok(())
}
```

---

## Core Concepts

### The Agent

The `Agent` is your main interface to the framework. It:
- Manages conversation state
- Handles LLM communication
- Orchestrates tool execution
- Maintains message history

```rust
let agent = Agent::new(config).await?;
```

### Configuration

Use the builder pattern to configure your agent:

```rust
let config = AgentConfig::builder()
    // Required
    .model("gpt-4")              // LLM model to use
    .api_key("sk-...")           // Or .api_key_from_env()
    
    // Optional
    .working_directory(".")      // Where to run file operations
    .approval_policy(policy)     // Tool approval settings
    .sandbox_policy(policy)      // Security boundaries
    .tool(CustomTool)            // Add custom tools
    .tools(StandardTools::all()) // Add standard tools
    .build()?;
```

### Tools

Tools are functions the LLM can call to interact with the world:

**Built-in Tools:**
- `shell` - Execute shell commands
- `read_file` - Read file contents
- `write_file` - Write to files

**Tool Collections:**
```rust
StandardTools::all()         // All standard tools
StandardTools::code_tools()  // Code-related tools
StandardTools::read_only()   // Read-only tools
```

### Sessions

Each `Agent` maintains a conversation session:
- Tracks message history
- Preserves context across turns
- Maintains tool state

```rust
// Multiple turns in same session
let response1 = agent.run("What files are in src/?").await?;
let response2 = agent.run("Read the first one").await?; // Knows context
```

---

## Configuration

### Model Selection

```rust
// OpenAI models
.model("gpt-4")
.model("gpt-4-turbo")
.model("gpt-3.5-turbo")

// Custom provider
.model("custom-model")
.api_base_url("https://api.custom-provider.com/v1")
```

### Authentication

```rust
// From environment variable (recommended)
.api_key_from_env()  // Reads OPENAI_API_KEY

// Direct (not recommended for production)
.api_key("sk-...")

// Try multiple env vars
let key = std::env::var("OPENAI_API_KEY")
    .or_else(|_| std::env::var("API_KEY"))?;
config.api_key(key)
```

### Approval Policies

Control when user approval is required:

```rust
use codex_agent_core::AskForApproval;

// Always ask for approval (safest)
.approval_policy(AskForApproval::UnlessTrusted)

// Ask only if command fails
.approval_policy(AskForApproval::OnFailure)

// Ask only if agent explicitly requests
.approval_policy(AskForApproval::OnRequest)

// Never ask (⚠️ dangerous!)
.approval_policy(AskForApproval::Never)
```

### Sandbox Policies

Define file system access boundaries:

```rust
use codex_agent_core::SandboxPolicy;

// Read-only access
.sandbox_policy(SandboxPolicy::ReadOnly)

// Write access in workspace only
.sandbox_policy(SandboxPolicy::WorkspaceWrite {
    writable_roots: vec![],
    network_access: false,
    exclude_tmpdir_env_var: false,
    exclude_slash_tmp: false,
})

// Full access (⚠️ use with caution!)
.sandbox_policy(SandboxPolicy::DangerFullAccess)
```

---

## Working with Tools

### Using Standard Tools

```rust
use codex_agent_tools::StandardTools;

let agent = Agent::new(
    AgentConfig::builder()
        .model("gpt-4")
        .api_key_from_env()
        .tools(StandardTools::code_tools())
        .build()?
).await?;

// The agent can now use shell and file tools
let response = agent.run(
    "List all Rust files and show me the contents of the largest one"
).await?;
```

### Creating Custom Tools

Implement the `Tool` trait:

```rust
use async_trait::async_trait;
use codex_agent_core::{Tool, ToolContext, ToolResult};
use serde_json::Value;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }
    
    fn description(&self) -> &str {
        "Get current weather for a location"
    }
    
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or zip code"
                }
            },
            "required": ["location"]
        })
    }
    
    async fn execute(
        &self,
        args: Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let location = args["location"].as_str().unwrap();
        
        // Your custom logic here
        let weather_data = fetch_weather(location).await?;
        
        Ok(ToolResult::text(format!(
            "Weather in {}: {}°F, {}",
            location, weather_data.temp, weather_data.conditions
        )))
    }
}

// Use it
let agent = Agent::new(
    AgentConfig::builder()
        .model("gpt-4")
        .api_key_from_env()
        .tool(WeatherTool)
        .build()?
).await?;
```

### Tool Context

Tools receive context about their execution environment:

```rust
async fn execute(&self, args: Value, ctx: &ToolContext) 
    -> anyhow::Result<ToolResult> 
{
    // Access working directory
    println!("Working in: {}", ctx.cwd.display());
    
    // Check approval policy
    if matches!(ctx.approval_policy, AskForApproval::UnlessTrusted) {
        // Handle approval...
    }
    
    // Check sandbox policy
    match &ctx.sandbox_policy {
        SandboxPolicy::ReadOnly => {
            // Read-only operations only
        }
        _ => {}
    }
    
    // Access call ID for logging
    println!("Call ID: {}", ctx.call_id);
    
    Ok(ToolResult::text("Done"))
}
```

---

## Advanced Usage

### Streaming Responses

Get real-time updates as the agent generates responses:

```rust
let mut stream = agent.run_streaming("Explain quantum computing").await?;

while let Some(event) = stream.next().await {
    match event {
        AgentEvent::TextDelta(delta) => {
            print!("{}", delta); // Print as it arrives
            std::io::stdout().flush()?;
        }
        AgentEvent::ToolExecution(name) => {
            println!("\n[Executing tool: {}]", name);
        }
        AgentEvent::Complete(response) => {
            println!("\n\nComplete!");
            break;
        }
    }
}
```

### Multi-Turn Conversations

The agent maintains context automatically:

```rust
let agent = Agent::new(config).await?;

// Turn 1
let r1 = agent.run("Create a file called data.txt with 'Hello, World!'").await?;
println!("Response 1: {}", r1.text);

// Turn 2 - references previous turn
let r2 = agent.run("Now read that file back to me").await?;
println!("Response 2: {}", r2.text);

// Turn 3 - continues conversation
let r3 = agent.run("What was in the file?").await?;
println!("Response 3: {}", r3.text);
```

### Error Handling

Handle errors gracefully:

```rust
use codex_agent_core::AgentError;

match agent.run(prompt).await {
    Ok(response) => {
        println!("Success: {}", response.text);
    }
    Err(AgentError::ClientError(e)) => {
        eprintln!("LLM API error: {}", e);
        // Maybe retry with exponential backoff
    }
    Err(AgentError::ToolError(e)) => {
        eprintln!("Tool execution failed: {}", e);
        // Maybe fall back to a different approach
    }
    Err(AgentError::ConfigError(e)) => {
        eprintln!("Configuration error: {}", e);
        // Fix configuration and retry
    }
    Err(AgentError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
        // Check connectivity, maybe retry
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

### Session Management

```rust
// Get session ID
let session_id = agent.session_id();
println!("Session: {}", session_id);

// Get message history
let messages = agent.messages().await;
for msg in messages {
    println!("{}: {}", msg.role, msg.content);
}

// Reset conversation
agent.reset().await;
```

---

## Best Practices

### 1. Security

```rust
// ✅ DO: Use environment variables for API keys
.api_key_from_env()

// ❌ DON'T: Hardcode API keys
.api_key("sk-...")  // Visible in source control!

// ✅ DO: Use appropriate sandbox policies
.sandbox_policy(SandboxPolicy::WorkspaceWrite { ... })

// ❌ DON'T: Use full access unless necessary
.sandbox_policy(SandboxPolicy::DangerFullAccess)
```

### 2. Error Handling

```rust
// ✅ DO: Handle specific error types
match agent.run(prompt).await {
    Ok(r) => handle_success(r),
    Err(AgentError::ClientError(e)) => handle_api_error(e),
    Err(e) => handle_other_error(e),
}

// ❌ DON'T: Swallow errors
let _ = agent.run(prompt).await;  // Silent failure!
```

### 3. Resource Management

```rust
// ✅ DO: Reuse agent instances
let agent = Agent::new(config).await?;
for prompt in prompts {
    let response = agent.run(prompt).await?;
    process(response);
}

// ❌ DON'T: Create new agents for each request
for prompt in prompts {
    let agent = Agent::new(config).await?;  // Expensive!
    let response = agent.run(prompt).await?;
}
```

### 4. Testing

```rust
// ✅ DO: Test with mock tools
#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockTool;
    
    #[async_trait]
    impl Tool for MockTool {
        // Mock implementation for testing
    }
    
    #[tokio::test]
    async fn test_agent_with_mock() {
        let agent = Agent::new(
            AgentConfig::builder()
                .model("gpt-4")
                .api_key("test-key")
                .tool(MockTool)
                .build()
                .unwrap()
        ).await.unwrap();
        
        // Test with mock...
    }
}
```

### 5. Logging

```rust
use tracing::{info, debug, error};

// ✅ DO: Use structured logging
info!("Starting agent session");
debug!(session_id = %agent.session_id(), "Session created");

match agent.run(prompt).await {
    Ok(r) => info!("Agent responded successfully"),
    Err(e) => error!(error = %e, "Agent failed"),
}
```

---

## Examples

### Example 1: CLI Tool

```rust
use clap::Parser;
use codex_agent_core::{Agent, AgentConfig};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()?
    ).await?;
    
    let response = agent.run(&cli.prompt).await?;
    println!("{}", response.text);
    
    Ok(())
}
```

### Example 2: Web Service

```rust
use axum::{Router, routing::post, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    prompt: String,
}

#[derive(Serialize)]
struct Response {
    text: String,
}

async fn chat(Json(req): Json<Request>) -> Json<Response> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()
            .unwrap()
    ).await.unwrap();
    
    let response = agent.run(req.prompt).await.unwrap();
    Json(Response { text: response.text })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/chat", post(chat));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Example 3: Background Worker

```rust
use tokio::task::JoinSet;

async fn process_task(prompt: String) -> anyhow::Result<String> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()?
    ).await?;
    
    let response = agent.run(prompt).await?;
    Ok(response.text)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tasks = vec![
        "Summarize document A",
        "Analyze data B",
        "Generate report C",
    ];
    
    let mut set = JoinSet::new();
    
    for task in tasks {
        set.spawn(process_task(task.to_string()));
    }
    
    while let Some(result) = set.join_next().await {
        match result? {
            Ok(output) => println!("Result: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

---

## API Reference

### Agent

```rust
impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self>;
    
    /// Run a single turn with the agent
    pub async fn run(&self, message: impl Into<String>) 
        -> Result<AgentResponse>;
    
    /// Run a turn with streaming responses
    pub async fn run_streaming(&self, message: impl Into<String>) 
        -> Result<AgentStream>;
    
    /// Get the session ID
    pub fn session_id(&self) -> &str;
    
    /// Get the current message history
    pub async fn messages(&self) -> Vec<Message>;
    
    /// Reset the conversation
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

### Tool

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    
    async fn execute(
        &self,
        args: Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult>;
    
    fn requires_approval(&self, ctx: &ToolContext) -> bool {
        false
    }
}
```

---

## Next Steps

- **Explore Examples**: Check out `examples/` directory for complete applications
- **Read Migration Guide**: If coming from full Codex CLI, see `MIGRATION_GUIDE.md`
- **Join Community**: Star the repo and contribute improvements
- **Build Something**: Start with a simple agent and extend it

## Getting Help

- **Documentation**: Read the analysis and implementation docs
- **Examples**: Study the provided examples
- **Issues**: Check GitHub issues for common problems
- **Source Code**: The framework is well-commented

## Contributing

Contributions welcome! Please:
1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Keep changes focused

---

**Happy Building! 🚀**
