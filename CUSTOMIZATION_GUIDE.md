# Framework Customization Guide

## Introduction

The Codex Agent Framework is designed with customization at every layer. This guide shows you how to customize the framework to meet your specific needs, from simple text completion to complex multi-agent systems.

## The Four Layers

The framework provides four layers of abstraction, each building on the previous:

```
Layer 4: Agent Application   ← agent.run()           [Highest Level]
Layer 3: Tool Orchestration   ← Session management
Layer 2: LLM Client          ← client.complete()
Layer 1: Provider            ← LlmProvider trait     [Lowest Level]
```

## Layer 1: Simple Text Completion

**Use Case:** One-off text generation, no conversation history needed

**Example:**
```rust
use codex_agent_core::client::LlmClient;

let client = LlmClient::new("api-key", "gpt-4", None);
let response = client.complete_simple("Translate 'hello' to Spanish").await?;
println!("{}", response); // "Hola"
```

**When to use:**
- Quick text transformations
- Simple Q&A
- No context needed
- Fastest and simplest API

## Layer 2: Chat Completion

**Use Case:** Conversational AI with history, but no tools

**Example:**
```rust
use codex_agent_core::client::{LlmClient, Message};

let client = LlmClient::new("api-key", "gpt-4", None);

let mut messages = vec![
    Message::user("My name is Alice"),
];

let response1 = client.complete_chat(messages.clone()).await?;
messages.push(response1);

messages.push(Message::user("What's my name?"));
let response2 = client.complete_chat(messages).await?;
// Response: "Your name is Alice"
```

**When to use:**
- Chatbots
- Customer service
- Conversational interfaces
- Context matters, but no tool calling needed

## Layer 3: Manual Tool Loop

**Use Case:** Tool calling with custom orchestration

**Example:**
```rust
use codex_agent_core::client::{LlmClient, Message};
use codex_agent_core::tools::{Tool, ToolSpec};

let client = LlmClient::new("api-key", "gpt-4", None);
let tools = get_my_tools();
let tool_specs: Vec<ToolSpec> = /* convert to specs */;

let mut messages = vec![Message::user("What's the weather?")];

loop {
    let response = client.complete_with_tools(
        messages.clone(), 
        tool_specs.clone()
    ).await?;
    
    if let Some(tool_calls) = response.tool_calls {
        // YOU control tool execution
        messages.push(response);
        
        for call in tool_calls {
            let result = execute_tool(&call).await?;
            messages.push(Message::tool_result(call.id, result));
        }
    } else {
        // No more tool calls
        break;
    }
}
```

**When to use:**
- Custom tool execution logic
- Need to intercept tool calls
- Want to add logging/monitoring
- Complex orchestration patterns

## Layer 4: Full Agent

**Use Case:** Autonomous agents with automatic tool loop

**Example:**
```rust
use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

let agent = Agent::new(
    AgentConfig::builder()
        .model("gpt-4")
        .api_key_from_env()
        .tools(StandardTools::all())
        .build()?
).await?;

let response = agent.run("Analyze this codebase").await?;
// Agent automatically executes tools as needed
```

**When to use:**
- Production agents
- Autonomous task execution
- Don't need custom orchestration
- Want the full agent experience

## Provider Customization

### Using Different Providers

The framework supports pluggable LLM providers through the `LlmProvider` trait.

**OpenAI (default):**
```rust
use codex_agent_core::providers::OpenAiProvider;

let provider = OpenAiProvider::new("api-key", "gpt-4", None);
```

**Simple Completion Provider (no tools):**
```rust
use codex_agent_core::providers::SimpleCompletionProvider;

let provider = SimpleCompletionProvider::new("api-key", "gpt-4");
// This provider doesn't support tool calling
```

### Creating Custom Providers

Implement the `LlmProvider` trait:

```rust
use async_trait::async_trait;
use codex_agent_core::providers::LlmProvider;
use codex_agent_core::client::{Message, StreamEvent};
use codex_agent_core::tools::ToolSpec;

struct LocalLlamaProvider {
    base_url: String,
    model: String,
}

#[async_trait]
impl LlmProvider for LocalLlamaProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message> {
        // Your custom implementation
        let response = reqwest::Client::new()
            .post(&format!("{}/v1/completions", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
            }))
            .send()
            .await?;
        
        // Parse and return
        todo!()
    }
    
    async fn stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>> {
        // Streaming implementation
        todo!()
    }
    
    fn supports_tools(&self) -> bool {
        false  // Llama might not support tools
    }
    
    fn model(&self) -> &str {
        &self.model
    }
}

// Use it
let provider = LocalLlamaProvider {
    base_url: "http://localhost:8080".to_string(),
    model: "llama-2-13b".to_string(),
};
```

## Advanced Customization

### Custom Tool Execution

Create your own tool orchestrator:

```rust
struct CachedToolOrchestrator {
    cache: Arc<Mutex<HashMap<String, ToolResult>>>,
}

impl CachedToolOrchestrator {
    async fn execute_tool(
        &self,
        tool_call: &ToolCall,
        registry: &ToolRegistry,
        context: &ToolContext,
    ) -> Result<ToolResult> {
        let cache_key = format!(
            "{}-{}",
            tool_call.function.name,
            tool_call.function.arguments
        );
        
        // Check cache
        if let Some(cached) = self.cache.lock().await.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Execute and cache
        let result = registry.execute(tool_call, context).await?;
        self.cache.lock().await.insert(cache_key, result.clone());
        Ok(result)
    }
}
```

### Custom System Prompts

Override system prompt generation:

```rust
trait SystemPromptBuilder {
    fn build_prompt(&self, config: &AgentConfig) -> String;
}

struct CustomPromptBuilder;

impl SystemPromptBuilder for CustomPromptBuilder {
    fn build_prompt(&self, config: &AgentConfig) -> String {
        format!(
            "You are an expert {}. {}",
            "developer",
            "Always write secure code."
        )
    }
}
```

## Best Practices

### 1. Choose the Right Layer

- **Layer 1** for simple, stateless operations
- **Layer 2** for conversational apps without tools
- **Layer 3** for custom tool orchestration
- **Layer 4** for full autonomous agents

### 2. Start Simple

Begin with the highest layer that meets your needs, then drop down if you need more control.

```rust
// Start here
let agent = Agent::new(config).await?;

// Only drop down if needed
let client = LlmClient::new(...);
let response = client.complete_chat(messages).await?;
```

### 3. Reuse Components

Mix and match layers:

```rust
// Use client for simple completion
let simple = client.complete_simple("Quick question").await?;

// Use agent for complex tasks
let complex = agent.run("Analyze codebase").await?;
```

### 4. Test at Each Layer

```rust
#[tokio::test]
async fn test_layer_1() {
    let client = LlmClient::new("test-key", "gpt-4", None);
    let response = client.complete_simple("test").await.unwrap();
    assert!(!response.is_empty());
}

#[tokio::test]
async fn test_layer_4() {
    let agent = Agent::new(config).await.unwrap();
    let response = agent.run("test").await.unwrap();
    assert!(!response.text.is_empty());
}
```

## Examples

See the `examples/` directory for complete working examples:

- **minimal** - Simplest agent (Layer 4)
- **layered_api** - Demonstrates all 4 layers
- **custom_tool** - Custom tool implementation
- **code_assistant** - Production-ready application

## Migration from Full Codex CLI

If you're migrating from the full Codex CLI:

```rust
// Old CLI-based approach
// codex "analyze this code"

// New framework approach (Layer 4)
let agent = Agent::new(config).await?;
let response = agent.run("analyze this code").await?;

// Or Layer 1 for simple cases
let client = LlmClient::new(api_key, "gpt-4", None);
let response = client.complete_simple("analyze this code").await?;
```

## Troubleshooting

**Q: Which layer should I use?**
A: Start with Layer 4 (Agent). Only drop to lower layers if you need more control.

**Q: How do I customize tool execution?**
A: Use Layer 3 (manual tool loop) to control tool execution yourself.

**Q: Can I mix layers?**
A: Yes! Use different layers for different tasks in the same application.

**Q: How do I add a new LLM provider?**
A: Implement the `LlmProvider` trait with your custom logic.

## Next Steps

1. Read the [USAGE_GUIDE.md](USAGE_GUIDE.md) for detailed API documentation
2. Try the [examples](agent-framework/examples/)
3. Check [FAQ.md](FAQ.md) for common questions
4. See [CUSTOMIZATION_ANALYSIS.md](CUSTOMIZATION_ANALYSIS.md) for detailed architecture

## Summary

The framework provides flexibility at every level:

- **Layer 1**: Simple text → Simple API
- **Layer 2**: Chat history → Chat API
- **Layer 3**: Manual control → Tool Loop API
- **Layer 4**: Autonomous → Agent API

Choose the layer that matches your needs, and enjoy the flexibility!
