# Troubleshooting Guide

## Common Issues and Solutions

### Installation Issues

#### Problem: Cargo cannot find codex-protocol

**Error:**
```
error: failed to load manifest for dependency `codex-protocol`
```

**Solution:**
The framework depends on the `codex-protocol` crate from the main Codex CLI. Ensure you have:

1. Cloned the full repository
2. Used the correct path in your `Cargo.toml`:

```toml
codex-agent-core = { path = "path/to/codex-cli/agent-framework/crates/codex-agent-core" }
```

The path should point to the `agent-framework` directory within the cloned repo.

---

#### Problem: Rust edition not supported

**Error:**
```
error: edition 2024 is unstable
```

**Solution:**
Ensure you're using a recent enough Rust version:

```bash
rustup update
rustc --version  # Should be 1.80 or later
```

---

### API Key Issues

#### Problem: API key not found

**Error:**
```
ConfigError: API key must be specified
```

**Solution:**

1. **Check environment variable:**
```bash
echo $OPENAI_API_KEY
```

2. **Set it if missing:**
```bash
export OPENAI_API_KEY=sk-your-key-here
```

3. **Or pass it directly:**
```rust
AgentConfig::builder()
    .api_key("sk-your-key-here")  // Not recommended for production
    .build()?
```

4. **Verify it's loaded:**
```rust
use std::env;
match env::var("OPENAI_API_KEY") {
    Ok(key) => println!("Key found: {}...", &key[..10]),
    Err(_) => eprintln!("Key not found!"),
}
```

---

#### Problem: Invalid API key

**Error:**
```
ClientError: API request failed with status 401: Unauthorized
```

**Solution:**

1. Verify your API key is valid at [OpenAI Platform](https://platform.openai.com/)
2. Check for extra spaces or characters:
```bash
# Remove any trailing whitespace
export OPENAI_API_KEY=$(echo $OPENAI_API_KEY | tr -d ' \n')
```

3. Ensure you're using the correct key format (starts with `sk-`)

---

### Model Access Issues

#### Problem: Model not found or accessible

**Error:**
```
ClientError: API request failed with status 404: model 'gpt-4' not found
```

**Solution:**

1. **Check your OpenAI account tier** - GPT-4 requires paid access
2. **Use a different model:**
```rust
.model("gpt-3.5-turbo")  // Available on free tier
```

3. **Verify model name spelling:**
```rust
// Correct
.model("gpt-4")
.model("gpt-4-turbo")
.model("gpt-3.5-turbo")

// Incorrect
.model("gpt4")  // Missing hyphen!
.model("GPT-4")  // Wrong case!
```

---

### Network Issues

#### Problem: Connection timeout

**Error:**
```
NetworkError: connection timeout
```

**Solution:**

1. **Check internet connectivity:**
```bash
curl -I https://api.openai.com/v1/models
```

2. **Check proxy settings:**
```bash
env | grep -i proxy
```

3. **If behind a corporate proxy:**
```rust
// Set proxy in your environment
// export HTTPS_PROXY=http://proxy.company.com:8080
```

4. **Increase timeout** (if using custom HTTP client)

---

#### Problem: SSL/TLS errors

**Error:**
```
NetworkError: SSL certificate verification failed
```

**Solution:**

1. **Update OpenSSL:**
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install openssl

# macOS
brew upgrade openssl
```

2. **Update system certificates:**
```bash
# Ubuntu/Debian
sudo apt install ca-certificates
sudo update-ca-certificates
```

---

### Tool Execution Issues

#### Problem: Tool not found

**Error:**
```
ToolError: Tool 'shell' not found
```

**Solution:**

1. **Ensure tools are added to config:**
```rust
use codex_agent_tools::StandardTools;

AgentConfig::builder()
    .tools(StandardTools::code_tools())  // Add this!
    .build()?
```

2. **Verify tool is included:**
```rust
// Check what tools are available
let tools = StandardTools::all();
for tool in &tools {
    println!("Tool: {}", tool.name());
}
```

---

#### Problem: Shell command fails

**Error:**
```
ToolError: Error executing tool: command not found
```

**Solution:**

1. **Check if command exists:**
```bash
which ls  # Should show /bin/ls or similar
```

2. **Use absolute paths:**
```rust
agent.run("Use /bin/ls instead of ls").await?
```

3. **Check working directory:**
```rust
let agent = Agent::new(
    AgentConfig::builder()
        .working_directory("/path/to/project")  // Explicitly set
        .build()?
).await?;
```

---

#### Problem: File not found

**Error:**
```
ToolError: Failed to read file: No such file or directory
```

**Solution:**

1. **Use absolute paths:**
```rust
agent.run("Read /full/path/to/file.txt").await?
```

2. **Check working directory:**
```rust
println!("Working in: {}", agent.config.working_directory.display());
```

3. **Verify file exists:**
```bash
ls -la /path/to/file.txt
```

---

### Runtime Issues

#### Problem: Agent hangs indefinitely

**Symptom:** `agent.run()` never returns

**Solutions:**

1. **Check if LLM is generating infinite tool calls:**
   - Add logging to see what's happening
   - Use `run_streaming()` to see progress

2. **Network issue:**
   - Check if API is reachable
   - Look for proxy blocking requests

3. **Add timeout:**
```rust
use tokio::time::{timeout, Duration};

match timeout(Duration::from_secs(60), agent.run(prompt)).await {
    Ok(Ok(response)) => println!("{}", response.text),
    Ok(Err(e)) => eprintln!("Agent error: {}", e),
    Err(_) => eprintln!("Timeout!"),
}
```

---

#### Problem: High memory usage

**Symptom:** Process uses excessive RAM

**Solutions:**

1. **Reset agent between long conversations:**
```rust
agent.reset().await;  // Clears message history
```

2. **Create new agent for each task:**
```rust
for task in tasks {
    let agent = Agent::new(config).await?;
    agent.run(task).await?;
    // Agent dropped here, freeing memory
}
```

3. **Limit conversation length:**
```rust
let messages = agent.messages().await;
if messages.len() > 100 {
    agent.reset().await;
}
```

---

### Build Issues

#### Problem: Compilation errors in framework code

**Error:**
```
error[E0599]: no method named `clone` found for struct `AgentConfig`
```

**Solution:**
This is expected! `AgentConfig` cannot be cloned because it contains trait objects. Create new configs instead of cloning:

```rust
// ❌ Don't clone
let config2 = config.clone();

// ✅ Build new config
let config2 = AgentConfig::builder()
    .model("gpt-4")
    .api_key_from_env()
    .build()?;
```

---

#### Problem: Async runtime not available

**Error:**
```
panicked at 'there is no reactor running'
```

**Solution:**

Ensure you're using `#[tokio::main]`:

```rust
// ✅ Correct
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new(config).await?;
    // ...
}

// ❌ Wrong
fn main() {
    let agent = Agent::new(config).await;  // Can't await in sync function!
}
```

---

### Performance Issues

#### Problem: Slow response times

**Symptom:** Agent takes too long to respond

**Solutions:**

1. **Use faster model:**
```rust
.model("gpt-3.5-turbo")  // Faster than gpt-4
```

2. **Reduce context:**
```rust
// Keep conversations shorter
agent.reset().await;
```

3. **Enable streaming:**
```rust
let mut stream = agent.run_streaming(prompt).await?;
// Process responses as they arrive
```

4. **Check network latency:**
```bash
ping api.openai.com
```

---

#### Problem: Rate limiting

**Error:**
```
ClientError: API request failed with status 429: Rate limit exceeded
```

**Solutions:**

1. **Add retry logic with backoff:**
```rust
use tokio::time::{sleep, Duration};

let mut retries = 0;
loop {
    match agent.run(prompt).await {
        Ok(response) => break Ok(response),
        Err(AgentError::ClientError(e)) if e.contains("429") => {
            if retries >= 3 {
                break Err(anyhow::anyhow!("Max retries exceeded"));
            }
            retries += 1;
            sleep(Duration::from_secs(2_u64.pow(retries))).await;
        }
        Err(e) => break Err(e.into()),
    }
}
```

2. **Upgrade your OpenAI tier**

3. **Add delays between requests:**
```rust
for prompt in prompts {
    let response = agent.run(prompt).await?;
    sleep(Duration::from_secs(1)).await;
}
```

---

## Debugging Tips

### Enable Verbose Logging

```rust
// In main()
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Inspect Messages

```rust
let messages = agent.messages().await;
for (i, msg) in messages.iter().enumerate() {
    println!("Message {}: {} - {}", i, msg.role, msg.content);
}
```

### Test Tools Individually

```rust
use codex_agent_core::{ToolContext, Tool};

let tool = ShellTool;
let ctx = ToolContext {
    cwd: PathBuf::from("."),
    approval_policy: AskForApproval::Never,
    sandbox_policy: SandboxPolicy::ReadOnly,
    call_id: "test".to_string(),
};

let result = tool.execute(
    serde_json::json!({"command": "echo test"}),
    &ctx
).await?;

println!("Result: {:?}", result);
```

### Check Framework Version

```bash
cd agent-framework
cargo tree | grep codex
```

---

## Getting More Help

### Before Asking for Help

1. **Check this troubleshooting guide**
2. **Review the usage guide** (`USAGE_GUIDE.md`)
3. **Look at examples** in `examples/` directory
4. **Search GitHub issues**

### When Asking for Help

Include:

1. **Error message** (full text)
2. **Code snippet** (minimal reproduction)
3. **Versions:**
```bash
rustc --version
cargo --version
# Your Cargo.toml dependencies
```

4. **What you tried** (steps to reproduce)

### Where to Get Help

- **GitHub Issues**: For bugs and feature requests
- **Documentation**: Check all `.md` files
- **Examples**: Working code in `examples/`
- **Source Code**: Framework code is well-commented

---

## Prevention Tips

### Good Practices

1. **Always handle errors:**
```rust
match agent.run(prompt).await {
    Ok(r) => {},
    Err(e) => eprintln!("Error: {}", e),
}
```

2. **Use environment variables for secrets:**
```rust
.api_key_from_env()  // Don't hardcode!
```

3. **Test with simple prompts first:**
```rust
// Start simple
agent.run("What is 2+2?").await?;

// Then add complexity
agent.run("Read file and analyze").await?;
```

4. **Add timeouts for production:**
```rust
timeout(Duration::from_secs(30), agent.run(prompt)).await??;
```

5. **Monitor resource usage:**
```rust
let messages = agent.messages().await;
if messages.len() > 50 {
    println!("Warning: Long conversation history");
}
```

---

## Still Stuck?

If you've tried everything above and still having issues:

1. **Simplify:** Create the smallest possible reproduction
2. **Isolate:** Test each component separately
3. **Update:** Make sure you're on the latest version
4. **Report:** Open a GitHub issue with details

Remember: Most issues are configuration or environment-related, not framework bugs!
