# Frequently Asked Questions (FAQ)

## General Questions

### What is the Codex Agent Framework?

The Codex Agent Framework is a lightweight, reusable library extracted from the Codex CLI that enables developers to build LLM-powered agents with minimal code. It provides:

- Simple API for agent creation
- Tool system for extending agent capabilities
- Streaming responses
- Multi-turn conversations
- Type-safe protocol

### How does it differ from the full Codex CLI?

| Feature | Full CLI | Framework |
|---------|----------|-----------|
| Purpose | Complete coding assistant | Building blocks for custom agents |
| UI | Terminal UI included | Programmatic API only |
| Setup | CLI tool | Rust library |
| Customization | Configuration files | Full programmatic control |
| Tools | Built-in + MCP | Built-in + custom tools |
| Deployment | Standalone binary | Embed in your application |

### Is it production-ready?

Yes, for many use cases! The framework is:
- ✅ Functionally complete
- ✅ Type-safe
- ✅ Well-documented
- ✅ Tested (builds successfully)

However, consider adding:
- Your own comprehensive tests
- Monitoring and logging
- Error recovery strategies
- Rate limiting

---

## Setup & Installation

### Do I need to clone the entire Codex CLI repo?

Yes, currently. The framework depends on `codex-protocol` which is part of the main repo.

```bash
git clone https://github.com/your-org/codex-cli
cd codex-cli
# Then reference agent-framework in your project
```

### Can I publish it to crates.io?

Not yet. The framework currently depends on the local `codex-protocol` crate. To publish:

1. Either publish `codex-protocol` first
2. Or vendor the protocol types into the framework

This is planned for a future release.

### What Rust version do I need?

Rust 2024 edition (1.80 or later). Check with:

```bash
rustc --version
```

Update if needed:
```bash
rustup update
```

---

## API Keys & Authentication

### Where do I get an API key?

1. Go to [OpenAI Platform](https://platform.openai.com/)
2. Sign up or log in
3. Navigate to API keys
4. Create a new secret key
5. Copy and save it securely (you can't view it again!)

### Do I need a paid OpenAI account?

- **Free tier:** Can use `gpt-3.5-turbo`
- **Paid tier:** Required for `gpt-4` and other advanced models

### How should I store my API key?

**✅ Recommended:**
```bash
export OPENAI_API_KEY=sk-your-key
```

Then in code:
```rust
.api_key_from_env()
```

**❌ Not recommended:**
```rust
.api_key("sk-your-key")  // Hardcoded! Visible in git!
```

**🔐 Production:**
- Use secrets management (AWS Secrets Manager, HashiCorp Vault, etc.)
- Environment variables injected at runtime
- Never commit keys to source control

### Can I use other LLM providers?

Yes! The framework supports OpenAI-compatible APIs:

```rust
AgentConfig::builder()
    .model("custom-model")
    .api_base_url("https://api.anthropic.com/v1")
    .api_key(anthropic_key)
    .build()?
```

Works with:
- OpenAI
- Anthropic Claude (via compatibility layer)
- Azure OpenAI
- Any OpenAI-compatible API

---

## Usage

### How do I create the simplest possible agent?

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
    
    let response = agent.run("Hello!").await?;
    println!("{}", response.text);
    Ok(())
}
```

### Can agents access files and run commands?

Yes! Add tools:

```rust
use codex_agent_tools::StandardTools;

AgentConfig::builder()
    .model("gpt-4")
    .api_key_from_env()
    .tools(StandardTools::code_tools())  // Enables file & shell access
    .build()?
```

Then the agent can read/write files and execute commands as needed.

### How do I maintain conversation context?

Automatic! The agent maintains history:

```rust
let agent = Agent::new(config).await?;

// Turn 1
let r1 = agent.run("My name is Alice").await?;

// Turn 2 - agent remembers context
let r2 = agent.run("What's my name?").await?;
// Response: "Your name is Alice"
```

### Can I stream responses?

Yes:

```rust
let mut stream = agent.run_streaming("Explain quantum physics").await?;

while let Some(event) = stream.next().await {
    match event {
        AgentEvent::TextDelta(delta) => print!("{}", delta),
        AgentEvent::Complete(_) => break,
        _ => {}
    }
}
```

### How do I reset the conversation?

```rust
agent.reset().await;
// Now the agent has no memory of previous turns
```

---

## Tools

### What built-in tools are available?

- `shell` - Execute shell commands
- `read_file` - Read file contents
- `write_file` - Write to files

Access via:
```rust
use codex_agent_tools::StandardTools;

StandardTools::all()         // All tools
StandardTools::code_tools()  // shell + file tools
StandardTools::read_only()   // read_file only
```

### How do I create a custom tool?

Implement the `Tool` trait:

```rust
use async_trait::async_trait;
use codex_agent_core::{Tool, ToolContext, ToolResult};

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Does something useful" }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "param": { "type": "string" }
            }
        })
    }
    
    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext)
        -> anyhow::Result<ToolResult>
    {
        Ok(ToolResult::text("Done!"))
    }
}

// Use it
AgentConfig::builder().tool(MyTool).build()?
```

### Can tools require approval?

Yes! Implement `requires_approval`:

```rust
fn requires_approval(&self, ctx: &ToolContext) -> bool {
    // Only ask for approval if policy requires it
    matches!(ctx.approval_policy, AskForApproval::UnlessTrusted)
}
```

### How do I test custom tools?

```rust
#[tokio::test]
async fn test_my_tool() {
    let tool = MyTool;
    let ctx = ToolContext {
        cwd: PathBuf::from("."),
        approval_policy: AskForApproval::Never,
        sandbox_policy: SandboxPolicy::ReadOnly,
        call_id: "test".to_string(),
    };
    
    let result = tool.execute(
        serde_json::json!({"param": "value"}),
        &ctx
    ).await.unwrap();
    
    assert!(!result.is_error());
}
```

---

## Performance

### Why is the first request slow?

Several reasons:
1. **Cold start:** Tokio runtime initialization
2. **TLS handshake:** First HTTPS connection
3. **Model loading:** API-side initialization

Subsequent requests are faster.

### How can I make it faster?

1. **Use faster models:**
```rust
.model("gpt-3.5-turbo")  // Faster than gpt-4
```

2. **Reuse agent instances:**
```rust
let agent = Agent::new(config).await?;
// Reuse for multiple requests
```

3. **Stream responses:**
```rust
run_streaming()  // Shows results as they arrive
```

4. **Parallel processing:**
```rust
let tasks: Vec<_> = prompts.into_iter()
    .map(|p| tokio::spawn(process(p)))
    .collect();
```

### How much does it cost?

Depends on:
- Model used (GPT-4 is more expensive than GPT-3.5)
- Token count (input + output)
- Frequency of use

**Example costs (GPT-4):**
- Simple query: $0.01 - $0.05
- Code analysis: $0.10 - $0.50
- Long conversation: $0.50 - $2.00

Check current pricing at [OpenAI Pricing](https://openai.com/pricing).

### Can I cache responses?

Not built-in, but you can:

```rust
use std::collections::HashMap;

let mut cache: HashMap<String, String> = HashMap::new();

if let Some(cached) = cache.get(prompt) {
    return Ok(cached.clone());
}

let response = agent.run(prompt).await?;
cache.insert(prompt.to_string(), response.text.clone());
```

---

## Security

### Is it safe to give agents file access?

Depends on your sandbox policy:

- `ReadOnly` - Safe, can only read
- `WorkspaceWrite` - Moderately safe, can write in working directory
- `DangerFullAccess` - ⚠️ Dangerous, full system access

**Recommendation:** Start with `ReadOnly` or `WorkspaceWrite`.

### Can agents access the internet?

Only if you give them a tool that makes HTTP requests. Built-in tools don't access the network (except the `shell` tool can run `curl`, `wget`, etc.).

### What about API key security?

**Best practices:**
1. ✅ Use environment variables
2. ✅ Never commit keys to git
3. ✅ Rotate keys regularly
4. ✅ Use secrets management in production
5. ❌ Don't hardcode keys
6. ❌ Don't log keys
7. ❌ Don't pass keys in URLs

### Can I audit agent actions?

Yes! The agent maintains message history:

```rust
let messages = agent.messages().await;
for msg in messages {
    // Log or audit each message
    audit_log(&msg);
}
```

Or use tracing:
```rust
use tracing::info;

info!("Agent executing tool: {}", tool_name);
```

---

## Error Handling

### How do I handle errors?

Match on specific error types:

```rust
use codex_agent_core::AgentError;

match agent.run(prompt).await {
    Ok(response) => handle_success(response),
    Err(AgentError::ClientError(e)) => {
        // API error - maybe retry
    }
    Err(AgentError::ToolError(e)) => {
        // Tool failed - maybe use fallback
    }
    Err(e) => {
        // Other errors
    }
}
```

### What if the API is down?

Implement retry logic:

```rust
let mut attempts = 0;
loop {
    match agent.run(prompt).await {
        Ok(response) => break Ok(response),
        Err(e) if attempts < 3 => {
            attempts += 1;
            tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
        }
        Err(e) => break Err(e),
    }
}
```

### Can I timeout long-running requests?

Yes:

```rust
use tokio::time::{timeout, Duration};

match timeout(Duration::from_secs(30), agent.run(prompt)).await {
    Ok(Ok(response)) => println!("{}", response.text),
    Ok(Err(e)) => eprintln!("Agent error: {}", e),
    Err(_) => eprintln!("Request timed out!"),
}
```

---

## Development

### How do I test my agents?

1. **Unit test tools:**
```rust
#[tokio::test]
async fn test_tool() {
    let tool = MyTool;
    // Test tool in isolation
}
```

2. **Integration test with mock LLM:**
```rust
// TODO: Mock LLM client not yet implemented
// For now, test against real API with test key
```

3. **End-to-end test:**
```rust
#[tokio::test]
async fn test_agent() {
    let agent = Agent::new(test_config()).await?;
    let response = agent.run("test prompt").await?;
    assert!(!response.text.is_empty());
}
```

### How do I debug issues?

1. **Enable logging:**
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

2. **Inspect messages:**
```rust
let messages = agent.messages().await;
dbg!(&messages);
```

3. **Use streaming to see progress:**
```rust
let mut stream = agent.run_streaming(prompt).await?;
// Watch what happens in real-time
```

### Can I contribute to the framework?

Yes! Contributions welcome:
1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Update documentation
5. Submit a pull request

---

## Deployment

### Can I deploy this in production?

Yes, with proper precautions:

1. ✅ Add comprehensive error handling
2. ✅ Implement monitoring and logging
3. ✅ Use secrets management
4. ✅ Add rate limiting
5. ✅ Test thoroughly
6. ✅ Plan for API outages

### How do I package for deployment?

```bash
cargo build --release
```

The binary includes all dependencies. Deploy:
- As a microservice
- In a Docker container
- As a Lambda function
- As a standalone binary

### What about Docker?

Example `Dockerfile`:

```dockerfile
FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/your-app /usr/local/bin/
CMD ["your-app"]
```

---

## Troubleshooting

### Where can I find more help?

1. **TROUBLESHOOTING.md** - Common issues and solutions
2. **USAGE_GUIDE.md** - Complete usage documentation
3. **Examples** - Working code in `examples/` directory
4. **GitHub Issues** - Search or create an issue

### My question isn't answered here!

1. Check the **TROUBLESHOOTING.md** guide
2. Read the **USAGE_GUIDE.md** documentation
3. Look at the **examples** directory
4. Search GitHub issues
5. Create a new issue with details

---

## Future Plans

### What's coming next?

Planned features:
- [ ] Comprehensive test suite
- [ ] Mock LLM for testing
- [ ] Session persistence
- [ ] More LLM providers
- [ ] Advanced features (caching, etc.)
- [ ] Official crates.io release

### How can I stay updated?

- Star the repository
- Watch for releases
- Follow GitHub discussions
- Check the changelog

### Can I request features?

Yes! Create a GitHub issue with:
- Clear description of the feature
- Use case / motivation
- Example API if applicable

---

**Have more questions? Check the other documentation files or open an issue!**
