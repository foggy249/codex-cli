# Codex Agent Framework - Implementation Plan

## Overview

This document outlines the concrete steps to extract a reusable agent framework from the Codex CLI codebase.

## Framework Structure

```
agent-framework/
├── Cargo.toml                  # Workspace root
├── README.md                   # Framework documentation
├── examples/                   # Example applications
│   ├── minimal/               # Bare minimum agent
│   ├── code_assistant/        # Code review agent
│   └── chat_bot/              # Simple chat agent
└── crates/
    ├── codex-agent-core/      # Core agent framework
    │   ├── src/
    │   │   ├── lib.rs
    │   │   ├── agent.rs       # Main Agent API
    │   │   ├── session.rs     # Session management
    │   │   ├── protocol.rs    # Re-export from codex-protocol
    │   │   ├── client.rs      # LLM client facade
    │   │   ├── tools.rs       # Tool system facade
    │   │   └── config.rs      # Framework configuration
    │   └── Cargo.toml
    ├── codex-agent-tools/     # Standard tool implementations
    │   ├── src/
    │   │   ├── lib.rs
    │   │   ├── shell.rs       # Shell execution
    │   │   ├── file.rs        # File operations
    │   │   └── registry.rs    # Standard tool registry
    │   └── Cargo.toml
    └── codex-agent-mcp/       # MCP integration (optional)
        ├── src/
        │   ├── lib.rs
        │   └── client.rs
        └── Cargo.toml
```

## Implementation Phases

### Phase 1: Foundation Setup ✅

**Goal:** Create framework structure and extract protocol types

**Tasks:**
1. Create `agent-framework/` directory
2. Set up Cargo workspace
3. Create `codex-agent-core` crate
4. Re-export protocol types from existing `codex-protocol`
5. Add minimal dependencies
6. Write basic README

**Deliverables:**
- Compiling workspace
- Protocol types accessible
- Basic project structure

### Phase 2: Core Agent API 🔄

**Goal:** Create simple, high-level Agent API

**Tasks:**
1. Design Agent struct and builder pattern
2. Extract session management (simplified from codex.rs)
3. Implement Op/Event queue system
4. Create configuration types (minimal subset)
5. Add error types
6. Write unit tests

**API Shape:**
```rust
pub struct Agent {
    // Private fields
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Result<Self>;
    pub async fn run(&self, prompt: impl Into<String>) -> Result<AgentResponse>;
    pub async fn run_streaming(&self, prompt: impl Into<String>) -> Result<AgentStream>;
    pub async fn submit(&self, op: Op) -> Result<()>;
    pub async fn next_event(&self) -> Result<Option<Event>>;
}

pub struct AgentConfig {
    pub model: String,
    pub api_key: String,
    pub tools: Vec<Box<dyn Tool>>,
    // ... more fields
}

impl AgentConfig {
    pub fn builder() -> AgentConfigBuilder;
}
```

**Deliverables:**
- Working Agent API
- Basic configuration
- Event system functioning

### Phase 3: LLM Client Integration 🔄

**Goal:** Integrate LLM client with streaming support

**Tasks:**
1. Extract ModelClient (adapt from core/src/client.rs)
2. Simplify authentication (API key only initially)
3. Implement streaming response handling
4. Add provider configuration (OpenAI, Anthropic)
5. Test with real API calls

**Key Adaptations:**
- Remove ChatGPT OAuth (framework uses API keys)
- Simplify auth manager
- Keep streaming SSE logic
- Support multiple providers

**Deliverables:**
- Working LLM client
- Streaming responses
- Provider abstraction

### Phase 4: Tool System 🔄

**Goal:** Implement pluggable tool system

**Tasks:**
1. Define Tool trait
2. Create ToolRegistry
3. Extract tool orchestrator (simplified)
4. Implement basic approval system
5. Add standard tools (shell, file ops)
6. Write tool examples

**Tool Trait:**
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult>;
}

pub struct ToolContext {
    pub cwd: PathBuf,
    pub approval_policy: ApprovalPolicy,
    pub sandbox_policy: SandboxPolicy,
}
```

**Deliverables:**
- Tool trait and registry
- Standard tool implementations
- Working tool execution

### Phase 5: Examples & Documentation 📝

**Goal:** Create working examples and comprehensive docs

**Tasks:**
1. Minimal agent example (10 lines)
2. Code assistant example
3. Chat bot with history
4. Write API documentation
5. Write architecture guide
6. Create migration guide

**Examples:**

**Minimal:**
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

**Code Assistant:**
```rust
use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .tools(StandardTools::code_tools())
            .working_directory(".")
            .build()?
    ).await?;
    
    let response = agent.run(
        "Review the code in src/ and suggest improvements"
    ).await?;
    
    println!("{}", response.text);
    Ok(())
}
```

**Deliverables:**
- Working examples
- Complete documentation
- Tutorial guides

### Phase 6: PoC Application 🚀

**Goal:** Build production-ready demo application

**Tasks:**
1. Design PoC application (e.g., code review bot)
2. Implement with framework
3. Add CLI interface
4. Write application docs
5. Performance testing
6. Polish UX

**PoC Ideas:**
- **Code Review Bot:** Analyze PRs and suggest improvements
- **Documentation Generator:** Generate docs from code
- **Test Generator:** Create tests for untested code
- **Refactoring Assistant:** Suggest and apply refactorings

**Deliverables:**
- Complete PoC application
- Application documentation
- Demo video/screenshots

## Technical Decisions

### What to Include

✅ **Core Event System:** Op/Event queue pattern
✅ **LLM Client:** Streaming, multiple providers
✅ **Tool System:** Registry, orchestration, basic tools
✅ **Protocol Types:** All from codex-protocol
✅ **Basic Config:** Model, API key, tools, policies
✅ **Error Handling:** Comprehensive error types

### What to Simplify

⚠️ **Authentication:** API key only (no ChatGPT OAuth)
⚠️ **Configuration:** Subset of full config system
⚠️ **Approval System:** Basic implementation (async callbacks)
⚠️ **History Management:** Simplified conversation tracking

### What to Make Optional

❌ **Sandboxing:** Platform-specific, advanced feature
❌ **TUI:** Not needed for framework
❌ **Telemetry:** Optional, plugin-based
❌ **MCP Server:** Separate optional crate

## Dependencies

### Core Framework
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
async-channel = "2"
async-trait = "0.1"
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "stream"] }
eventsource-stream = "0.2"
tracing = "0.1"
futures = "0.3"

# Re-use existing protocol crate
codex-protocol = { path = "../../codex-rs/protocol" }
```

### Tools Crate
```toml
[dependencies]
codex-agent-core = { path = "../codex-agent-core" }
tokio = { version = "1", features = ["process", "io-std"] }
tempfile = "3"
which = "6"
walkdir = "2"
```

## Testing Strategy

### Unit Tests
- Protocol serialization
- Event queue behavior
- Configuration parsing
- Tool execution (mocked LLM)

### Integration Tests
- Real API calls (with test API key)
- Multi-turn conversations
- Tool orchestration
- Error handling

### Example Tests
- All examples must compile and run
- Automated testing of examples
- Documentation code snippets tested

## Success Criteria

1. ✅ Framework compiles independently
2. ✅ All examples work with real APIs
3. ✅ Documentation is comprehensive
4. ✅ PoC application is production-ready
5. ✅ No regressions in codex-rs codebase
6. ✅ Performance is acceptable (<2x overhead)
7. ✅ Error messages are clear and actionable

## Risk Mitigation

### Risk: Over-complexity
**Mitigation:** Start with MVP, add features only when needed

### Risk: Incomplete extraction
**Mitigation:** Build real examples to validate completeness

### Risk: Breaking existing code
**Mitigation:** Keep framework separate, no modifications to codex-rs

### Risk: Poor documentation
**Mitigation:** Write docs alongside code, not after

### Risk: Performance issues
**Mitigation:** Profile early, optimize hot paths

## Timeline

- **Day 1:** Phase 1 + Phase 2 (Foundation + Agent API)
- **Day 2:** Phase 3 + Phase 4 (LLM Client + Tools)
- **Day 3:** Phase 5 (Examples + Docs)
- **Day 4:** Phase 6 (PoC Application)

## Deliverables Summary

1. **FRAMEWORK_ANALYSIS.md** ✅ - Comprehensive architecture analysis
2. **agent-framework/** - Complete framework crates
3. **examples/** - Working example applications
4. **docs/** - API and usage documentation
5. **PoC Application** - Production-ready demo
6. **MIGRATION_GUIDE.md** - Guide for adopting framework

## Next Steps

1. Review and approve this plan
2. Begin Phase 1 implementation
3. Iterate on feedback
4. Complete all phases
5. Final review and polish
