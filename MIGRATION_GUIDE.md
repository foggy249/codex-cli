# Migration Guide: From Full Codex CLI to Agent Framework

This guide helps you understand how to migrate from using the full Codex CLI to building your own applications with the Codex Agent Framework.

## Overview

The Codex Agent Framework extracts the core agent capabilities from the Codex CLI, providing a lightweight, reusable foundation for building LLM-powered applications.

## What's Different

### Full Codex CLI

```bash
# CLI-based interaction
codex "analyze this codebase"

# With full TUI
codex

# Non-interactive
codex exec "fix these tests"
```

### Agent Framework

```rust
// Programmatic API
let agent = Agent::new(config).await?;
let response = agent.run("analyze this codebase").await?;
println!("{}", response.text);
```

## Feature Comparison

| Feature | Full CLI | Framework | Notes |
|---------|----------|-----------|-------|
| Core Agent Logic | ✅ | ✅ | Same core |
| LLM Integration | ✅ | ✅ | OpenAI compatible |
| Tool System | ✅ | ✅ | Pluggable |
| Multi-turn Conversations | ✅ | ✅ | Full context |
| Streaming Responses | ✅ | ✅ | Real-time |
| Terminal UI (TUI) | ✅ | ❌ | Build your own UI |
| CLI Arguments | ✅ | ❌ | Use clap/your choice |
| ChatGPT OAuth | ✅ | ❌ | API keys only |
| Session Persistence | ✅ | ❌ | In-memory only |
| Platform Sandboxing | ✅ | ⚠️ | Basic only |
| MCP Server | ✅ | ⚠️ | Client only |
| Telemetry | ✅ | ❌ | Add your own |
| Auto-updates | ✅ | ❌ | N/A |

## Migration Examples

### Example 1: Simple Query

**Before (CLI):**
```bash
codex "What is 2+2?"
```

**After (Framework):**
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

### Example 2: Code Analysis with Tools

**Before (CLI):**
```bash
codex "analyze this codebase and suggest improvements"
```

**After (Framework):**
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
        "analyze this codebase and suggest improvements"
    ).await?;
    println!("{}", response.text);
    Ok(())
}
```

### Example 3: Multi-turn Conversation

**Before (CLI):**
```bash
# Interactive mode maintains context
codex
> What files are in src/?
> Read the first one
> Summarize it
```

**After (Framework):**
```rust
let agent = Agent::new(config).await?;

// First turn
let response1 = agent.run("What files are in src/?").await?;
println!("{}", response1.text);

// Second turn (context maintained)
let response2 = agent.run("Read the first one").await?;
println!("{}", response2.text);

// Third turn
let response3 = agent.run("Summarize it").await?;
println!("{}", response3.text);
```

### Example 4: Streaming Responses

**Before (CLI):**
```bash
# Streaming is automatic in TUI
codex "explain quantum computing"
```

**After (Framework):**
```rust
let mut stream = agent.run_streaming("explain quantum computing").await?;

while let Some(event) = stream.next().await {
    match event {
        AgentEvent::TextDelta(delta) => print!("{}", delta),
        AgentEvent::ToolExecution(name) => {
            println!("\n[Executing: {}]", name);
        }
        AgentEvent::Complete(response) => {
            println!("\n\nComplete!");
            break;
        }
    }
}
```

## Key Concepts Mapping

### Configuration

**CLI:**
```toml
# ~/.codex/config.toml
model = "gpt-4"
approval_policy = "untrusted"
sandbox_policy = "workspace-write"
```

**Framework:**
```rust
let config = AgentConfig::builder()
    .model("gpt-4")
    .approval_policy(AskForApproval::UnlessTrusted)
    .sandbox_policy(SandboxPolicy::WorkspaceWrite {
        writable_roots: vec![],
        network_access: false,
        exclude_tmpdir_env_var: false,
        exclude_slash_tmp: false,
    })
    .build()?;
```

### Authentication

**CLI:**
```bash
# OAuth flow
codex login

# Or API key
export OPENAI_API_KEY=sk-...
```

**Framework:**
```rust
// From environment
let config = AgentConfig::builder()
    .api_key_from_env()
    .build()?;

// Or direct
let config = AgentConfig::builder()
    .api_key("sk-...")
    .build()?;
```

### Tools

**CLI:**
Built-in tools available automatically.

**Framework:**
```rust
// Standard tools
let config = AgentConfig::builder()
    .tools(StandardTools::all())
    .build()?;

// Or specific tools
let config = AgentConfig::builder()
    .tools(StandardTools::code_tools())
    .build()?;

// Or custom tools
let config = AgentConfig::builder()
    .tool(CustomTool)
    .tool(AnotherTool)
    .build()?;
```

## Common Patterns

### Pattern 1: Command Line Tool

Create a CLI wrapper around the framework:

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    prompt: String,
    
    #[arg(short, long, default_value = "gpt-4")]
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let agent = Agent::new(
        AgentConfig::builder()
            .model(&cli.model)
            .api_key_from_env()
            .build()?
    ).await?;
    
    let response = agent.run(&cli.prompt).await?;
    println!("{}", response.text);
    Ok(())
}
```

### Pattern 2: Web Service

Build an HTTP API:

```rust
use axum::{Router, routing::post, Json};

async fn chat_endpoint(Json(req): Json<ChatRequest>) -> Json<ChatResponse> {
    let agent = Agent::new(config).await.unwrap();
    let response = agent.run(req.message).await.unwrap();
    Json(ChatResponse { text: response.text })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/chat", post(chat_endpoint));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Pattern 3: Background Worker

Process tasks asynchronously:

```rust
async fn process_review_task(file_path: PathBuf) -> anyhow::Result<String> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .tools(StandardTools::code_tools())
            .build()?
    ).await?;
    
    let response = agent.run(format!(
        "Review the file {} and provide feedback",
        file_path.display()
    )).await?;
    
    Ok(response.text)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Process multiple files concurrently
    let files = vec!["src/main.rs", "src/lib.rs", "src/utils.rs"];
    
    let tasks: Vec<_> = files.into_iter()
        .map(|f| tokio::spawn(process_review_task(PathBuf::from(f))))
        .collect();
    
    for task in tasks {
        match task.await? {
            Ok(review) => println!("Review: {}", review),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

## Advanced Topics

### Custom Tools

Implement your own tools:

```rust
use async_trait::async_trait;
use codex_agent_core::{Tool, ToolContext, ToolResult};

struct DatabaseQueryTool;

#[async_trait]
impl Tool for DatabaseQueryTool {
    fn name(&self) -> &str {
        "query_database"
    }
    
    fn description(&self) -> &str {
        "Execute a SQL query against the database"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "SQL query to execute"
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        // Your database logic here
        let query = args["query"].as_str().unwrap();
        // Execute query...
        Ok(ToolResult::text("Query results..."))
    }
}

// Use it
let agent = Agent::new(
    AgentConfig::builder()
        .tool(DatabaseQueryTool)
        .build()?
).await?;
```

### Session Management

Manage multiple concurrent sessions:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Agent>>>,
}

impl SessionManager {
    async fn get_or_create(&self, user_id: &str) -> Agent {
        let mut sessions = self.sessions.write().await;
        
        sessions.entry(user_id.to_string())
            .or_insert_with(|| {
                Agent::new(AgentConfig::builder()
                    .model("gpt-4")
                    .api_key_from_env()
                    .build()
                    .unwrap()
                ).await.unwrap()
            })
            .clone()
    }
}
```

### Error Handling

Framework errors are structured:

```rust
use codex_agent_core::AgentError;

match agent.run(prompt).await {
    Ok(response) => println!("{}", response.text),
    Err(AgentError::ClientError(e)) => {
        eprintln!("LLM API error: {}", e);
    }
    Err(AgentError::ToolError(e)) => {
        eprintln!("Tool execution failed: {}", e);
    }
    Err(AgentError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Limitations & Workarounds

### Limitation 1: No Session Persistence

**CLI:** Sessions saved to disk, can resume later

**Framework:** In-memory only

**Workaround:** Implement your own persistence:

```rust
// Serialize session state
let messages = agent.messages().await;
let json = serde_json::to_string(&messages)?;
std::fs::write("session.json", json)?;

// Later, reconstruct (simplified)
// Framework doesn't directly support this yet,
// but you can replay messages
```

### Limitation 2: No Platform Sandboxing

**CLI:** Full Seatbelt/Landlock support

**Framework:** Basic sandbox policies only

**Workaround:** Run in containers or use OS-level restrictions

### Limitation 3: No Built-in TUI

**CLI:** Beautiful terminal UI

**Framework:** Programmatic API only

**Workaround:** Use libraries like `ratatui` or `cursive`:

```rust
// Example with ratatui
use ratatui::Terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(/* backend */)?;
    
    loop {
        terminal.draw(|f| {
            // Render your UI
        })?;
        
        // Handle input, call agent, update UI
    }
    
    Ok(())
}
```

## Best Practices

1. **Error Handling**: Always handle errors properly
2. **Resource Management**: Clean up agents when done
3. **API Key Security**: Never hardcode API keys
4. **Rate Limiting**: Be aware of API rate limits
5. **Testing**: Write tests with mock LLM responses
6. **Logging**: Use `tracing` for observability
7. **Performance**: Cache agent instances when possible

## Next Steps

1. Read the [Framework README](../agent-framework/README.md)
2. Try the [Examples](../agent-framework/examples/)
3. Build your own application
4. Contribute improvements

## Getting Help

- **Issues**: Check GitHub issues
- **Examples**: See `agent-framework/examples/`
- **Source**: Read the framework source code
- **Original CLI**: Reference the full Codex CLI for patterns

## Contributing

Contributions to the framework are welcome! Please:

1. Keep changes focused and minimal
2. Add tests for new features
3. Update documentation
4. Follow existing code style

## License

Apache-2.0 (same as Codex CLI)
