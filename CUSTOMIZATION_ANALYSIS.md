# Framework Customization Layers - Analysis & Design

## Overview

This document analyzes the current framework architecture and proposes enhancements to enable better customization at every layer, from low-level LLM calls to high-level agent applications.

## Current Architecture Layers

```
┌─────────────────────────────────────────────┐
│  Layer 4: Agent Application (High-Level)    │  ← Agent.run()
├─────────────────────────────────────────────┤
│  Layer 3: Session Management (Mid-Level)    │  ← Session, tool orchestration
├─────────────────────────────────────────────┤
│  Layer 2: LLM Client (Low-Level)            │  ← LlmClient.complete()
├─────────────────────────────────────────────┤
│  Layer 1: HTTP/Network (Bottom)             │  ← reqwest
└─────────────────────────────────────────────┘
```

## Identified Customization Gaps

### Layer 1: LLM Client Level (Bottom)
**Current State:** Hardcoded to specific implementation
**Gaps:**
- ❌ Cannot swap LLM providers easily
- ❌ Cannot customize request/response format
- ❌ Cannot intercept or modify requests
- ❌ Cannot add custom headers or auth
- ❌ No retry logic or error handling hooks

### Layer 2: Non-Agentic LLM Application
**Current State:** Coupled with agent loop
**Gaps:**
- ❌ Cannot use LLM client without agent
- ❌ No simple completion API
- ❌ Cannot disable tool calling
- ❌ No direct message manipulation

### Layer 3: Session/Tool Orchestration
**Current State:** Fixed orchestration logic
**Gaps:**
- ❌ Cannot customize tool execution flow
- ❌ Cannot inject middleware
- ❌ Cannot override system prompt generation
- ❌ No hooks for before/after tool execution

### Layer 4: Agent Application (Top)
**Current State:** Single Agent abstraction
**Gaps:**
- ❌ Cannot customize agentic loop
- ❌ No multi-agent support
- ❌ Cannot add custom behaviors
- ❌ Limited event system

## Proposed Solution: Trait-Based Customization

### Design Principles
1. **Trait-based abstraction** at each layer
2. **Sensible defaults** for common cases
3. **Escape hatches** for advanced use
4. **Backward compatible** with existing code

## New Architecture with Customization Points

```
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Agent Application                             │
│  ┌─────────────────────────────────────────────────┐   │
│  │ trait AgentBehavior {                           │   │
│  │   fn before_turn() -> ...                       │   │
│  │   fn after_turn() -> ...                        │   │
│  │   fn on_tool_call() -> ...                      │   │
│  │ }                                                │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Session & Tool Orchestration                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ trait ToolOrchestrator {                        │   │
│  │   fn execute_tool() -> ...                      │   │
│  │   fn handle_tool_result() -> ...                │   │
│  │ }                                                │   │
│  │ trait SystemPromptBuilder {                     │   │
│  │   fn build_prompt() -> ...                      │   │
│  │ }                                                │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  Layer 2: LLM Client Abstraction                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │ trait LlmProvider {                             │   │
│  │   fn complete() -> ...                          │   │
│  │   fn stream() -> ...                            │   │
│  │   fn supports_tools() -> bool                   │   │
│  │ }                                                │   │
│  │ Implementations:                                │   │
│  │   - OpenAiProvider                              │   │
│  │   - AnthropicProvider                           │   │
│  │   - CustomProvider                              │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  Layer 1: HTTP/Network (Pluggable)                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │ trait HttpClient {                              │   │
│  │   fn request() -> ...                           │   │
│  │   fn with_retry() -> ...                        │   │
│  │ }                                                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: LLM Provider Abstraction (Layer 2)

**Goal:** Enable swappable LLM providers

**New Traits:**
```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Message>;
    
    async fn stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
    ) -> Result<Receiver<StreamEvent>>;
    
    fn supports_tools(&self) -> bool { true }
    fn supports_streaming(&self) -> bool { true }
}
```

**Implementations:**
- `OpenAiProvider` (existing client)
- `SimpleCompletionProvider` (no tools, just text)
- Template for custom providers

**Benefits:**
- ✅ Easy to add new LLM providers
- ✅ Can use different providers for different agents
- ✅ Simple text completion without agent loop

### Phase 2: Non-Agentic LLM API (Layer 2.5)

**Goal:** Simple LLM completion without agents

**New API:**
```rust
pub struct LlmClient {
    provider: Box<dyn LlmProvider>,
}

impl LlmClient {
    // Simple completion (no tools, no loop)
    pub async fn complete_simple(&self, prompt: &str) -> Result<String>;
    
    // Structured completion (with history, no tools)
    pub async fn complete_chat(
        &self,
        messages: Vec<Message>
    ) -> Result<Message>;
    
    // Full completion (with tools, but caller handles loop)
    pub async fn complete_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolSpec>,
    ) -> Result<Message>;
}
```

**Use Cases:**
```rust
// Use case 1: Simple text generation
let client = LlmClient::new(OpenAiProvider::new(api_key, "gpt-4"));
let response = client.complete_simple("Write a haiku").await?;

// Use case 2: Chat without tools
let messages = vec![
    Message::user("Hello"),
    // ... conversation
];
let response = client.complete_chat(messages).await?;

// Use case 3: Manual tool loop
loop {
    let response = client.complete_with_tools(messages, tools).await?;
    if let Some(tool_calls) = response.tool_calls {
        // Handle tools manually
    } else {
        break;
    }
}
```

### Phase 3: Session Customization (Layer 3)

**Goal:** Customize session behavior

**New Traits:**
```rust
#[async_trait]
pub trait SystemPromptBuilder: Send + Sync {
    fn build_prompt(&self, config: &AgentConfig) -> String;
}

#[async_trait]
pub trait ToolOrchestrator: Send + Sync {
    async fn execute_tool(
        &self,
        tool_call: &ToolCall,
        registry: &ToolRegistry,
        context: &ToolContext,
    ) -> Result<ToolResult>;
    
    fn before_tool_execution(&self, tool_name: &str) {}
    fn after_tool_execution(&self, tool_name: &str, result: &ToolResult) {}
}
```

**Configuration:**
```rust
AgentConfig::builder()
    .model("gpt-4")
    .api_key_from_env()
    .system_prompt_builder(CustomPromptBuilder)
    .tool_orchestrator(CustomOrchestrator)
    .build()?
```

### Phase 4: Agent Behavior Hooks (Layer 4)

**Goal:** Customize agent loop behavior

**New Traits:**
```rust
#[async_trait]
pub trait AgentBehavior: Send + Sync {
    async fn before_turn(&self, input: &str) -> Result<Option<String>>;
    async fn after_turn(&self, response: &str) -> Result<Option<String>>;
    async fn on_tool_call(&self, tool_name: &str, args: &Value) -> Result<bool>;
    async fn on_error(&self, error: &AgentError) -> Result<()>;
}
```

**Usage:**
```rust
struct LoggingBehavior;

#[async_trait]
impl AgentBehavior for LoggingBehavior {
    async fn before_turn(&self, input: &str) -> Result<Option<String>> {
        println!("User input: {}", input);
        Ok(None) // Don't modify input
    }
    
    async fn on_tool_call(&self, tool_name: &str, args: &Value) -> Result<bool> {
        println!("Calling tool: {} with {:?}", tool_name, args);
        Ok(true) // Allow execution
    }
}

let agent = Agent::builder()
    .config(config)
    .behavior(LoggingBehavior)
    .build()
    .await?;
```

## Backward Compatibility

All changes maintain backward compatibility:

```rust
// Old code still works
let agent = Agent::new(config).await?;
let response = agent.run("Hello").await?;

// New customization available but optional
let agent = Agent::builder()
    .config(config)
    .provider(CustomProvider)
    .behavior(CustomBehavior)
    .orchestrator(CustomOrchestrator)
    .build()
    .await?;
```

## Example Use Cases

### Use Case 1: Simple Text Completion (No Agent)

```rust
use codex_agent_core::LlmClient;
use codex_agent_core::providers::OpenAiProvider;

let client = LlmClient::new(
    OpenAiProvider::new(api_key, "gpt-4")
);

let response = client.complete_simple(
    "Translate 'Hello' to Spanish"
).await?;

println!("{}", response); // "Hola"
```

### Use Case 2: Custom LLM Provider

```rust
struct LocalLlamaProvider {
    base_url: String,
}

#[async_trait]
impl LlmProvider for LocalLlamaProvider {
    async fn complete(&self, messages: Vec<Message>, _tools: Option<Vec<ToolSpec>>) 
        -> Result<Message> 
    {
        // Custom implementation for local Llama
        let response = reqwest::Client::new()
            .post(&format!("{}/v1/completions", self.base_url))
            .json(&json!({ "messages": messages }))
            .send()
            .await?;
        // Parse and return
    }
    
    fn supports_tools(&self) -> bool { false }
}

// Use it
let agent = Agent::builder()
    .provider(LocalLlamaProvider { base_url: "http://localhost:8080" })
    .build()
    .await?;
```

### Use Case 3: Custom Tool Orchestration

```rust
struct CachedToolOrchestrator {
    cache: Arc<Mutex<HashMap<String, ToolResult>>>,
}

#[async_trait]
impl ToolOrchestrator for CachedToolOrchestrator {
    async fn execute_tool(
        &self,
        tool_call: &ToolCall,
        registry: &ToolRegistry,
        context: &ToolContext,
    ) -> Result<ToolResult> {
        let cache_key = format!("{}-{}", tool_call.function.name, tool_call.function.arguments);
        
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

### Use Case 4: Multi-Agent System

```rust
struct CoordinatorBehavior {
    sub_agents: Vec<Agent>,
}

#[async_trait]
impl AgentBehavior for CoordinatorBehavior {
    async fn before_turn(&self, input: &str) -> Result<Option<String>> {
        // Delegate to specialized sub-agents
        if input.contains("code") {
            let response = self.sub_agents[0].run(input).await?;
            return Ok(Some(response.text));
        }
        Ok(None) // Let main agent handle
    }
}
```

## Implementation Priority

### Critical (Week 1)
1. ✅ Create `LlmProvider` trait
2. ✅ Implement `OpenAiProvider` 
3. ✅ Add `SimpleCompletionProvider`
4. ✅ Create `LlmClient` with layered API

### High Priority (Week 2)
5. ✅ Create `SystemPromptBuilder` trait
6. ✅ Create `ToolOrchestrator` trait
7. ✅ Update `Session` to use traits
8. ✅ Add examples for each layer

### Medium Priority (Week 3)
9. ⚠️ Create `AgentBehavior` trait
10. ⚠️ Add `Agent::builder()` pattern
11. ⚠️ Add hooks and middleware
12. ⚠️ Update all documentation

### Future
- Provider implementations (Anthropic, etc.)
- Advanced orchestration patterns
- Multi-agent coordination
- Streaming middleware

## Documentation Updates Needed

1. **ARCHITECTURE.md** - Document new layer system
2. **CUSTOMIZATION_GUIDE.md** - How to customize each layer
3. **PROVIDER_GUIDE.md** - Creating custom providers
4. **EXAMPLES/** - One example per customization level
5. **API_REFERENCE.md** - Complete trait documentation

## Testing Strategy

1. **Unit Tests** - Each trait implementation
2. **Integration Tests** - Layer interactions
3. **Example Tests** - All use cases work
4. **Backward Compat Tests** - Old code still works

## Success Metrics

- ✅ Can use LLM client without agent
- ✅ Can swap LLM providers
- ✅ Can customize at every layer
- ✅ Backward compatible
- ✅ Clear documentation
- ✅ Working examples

## Summary

This refactoring provides:

1. **Layer 1 (Bottom)**: Custom HTTP clients, retry logic
2. **Layer 2 (LLM Client)**: Pluggable providers, simple completions
3. **Layer 3 (Session)**: Custom orchestration, prompt building
4. **Layer 4 (Agent)**: Behavior hooks, multi-agent support

All while maintaining backward compatibility and providing sensible defaults.
