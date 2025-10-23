# Codex Agent Framework - Comprehensive Architecture Analysis

**Date:** 2025-10-23
**Purpose:** Extract a reusable agent framework from the Codex CLI codebase

## Executive Summary

This document provides a detailed analysis of the Codex CLI codebase to guide the extraction of a reusable agent framework. The framework will enable developers to build LLM-powered applications and agents without the full complexity of the complete Codex CLI system.

### Key Findings

1. **Architecture Pattern**: The codebase follows a clean event-driven architecture with clear separation of concerns
2. **Core Components**: Protocol system, tool orchestration, LLM client abstraction, and sandbox management
3. **Extraction Strategy**: Surgical extraction of core abstractions while maintaining stability
4. **Target Framework**: Lightweight, composable agent framework with minimal dependencies

---

## 1. Codebase Architecture Overview

### 1.1 Repository Structure

```
codex-cli/
├── codex-rs/           # Rust implementation (main focus)
│   ├── core/           # Business logic and orchestration
│   ├── protocol/       # Event system and type definitions
│   ├── cli/            # CLI entry point
│   ├── tui/            # Terminal UI
│   ├── exec/           # Non-interactive execution
│   ├── mcp-server/     # Model Context Protocol server
│   ├── backend-client/ # API client for OpenAI backend
│   ├── rmcp-client/    # MCP client implementation
│   └── [utilities]/    # Various utility crates
├── sdk/                # TypeScript SDK wrapper
└── docs/               # Documentation
```

### 1.2 Architecture Layers

The system is organized in clear architectural layers:

```
┌─────────────────────────────────────────┐
│  User Interfaces (CLI, TUI, SDK)        │
├─────────────────────────────────────────┤
│  Core Orchestration (codex-core)        │
│  - Session management                   │
│  - Event processing                     │
│  - Tool orchestration                   │
├─────────────────────────────────────────┤
│  Protocol Layer (codex-protocol)        │
│  - Event types                          │
│  - Message formats                      │
│  - Op/Event definitions                 │
├─────────────────────────────────────────┤
│  Client Layer                           │
│  - Model client abstraction             │
│  - Provider implementations             │
│  - Authentication                       │
├─────────────────────────────────────────┤
│  Tool System                            │
│  - Tool registry                        │
│  - Tool orchestrator                    │
│  - Sandbox manager                      │
│  - Approval system                      │
├─────────────────────────────────────────┤
│  Infrastructure                         │
│  - MCP client/server                    │
│  - File operations                      │
│  - Process execution                    │
│  - Platform sandboxing                  │
└─────────────────────────────────────────┘
```

---

## 2. Major Components

### 2.1 Core Orchestrator (`codex-core`)

**Purpose:** Central business logic for the agent system

**Key Files:**
- `codex-rs/core/src/codex.rs` - Main `Codex` struct and session management
- `codex-rs/core/src/state.rs` - Session state management
- `codex-rs/core/src/conversation_history.rs` - Message history tracking
- `codex-rs/core/src/conversation_manager.rs` - Multi-conversation handling

**Core Concepts:**

1. **Submission Queue / Event Queue Pattern:**
```rust
pub struct Codex {
    next_id: AtomicU64,
    tx_sub: Sender<Submission>,   // Input queue
    rx_event: Receiver<Event>,     // Output queue
}
```

2. **Session Lifecycle:**
   - `spawn()` - Initialize a new Codex session
   - `submit()` - Send operations to the agent
   - `next_event()` - Receive events from the agent
   - Async event loop processes submissions

3. **Key Operations:**
   - `UserTurn` - Process user input and generate response
   - `Interrupt` - Cancel ongoing operations
   - `ExecApproval/PatchApproval` - Handle approval requests
   - `Compact` - Compress conversation history

**Framework Extraction Value:** ⭐⭐⭐⭐⭐
- High-level orchestration patterns are essential
- Event-driven architecture is perfect for framework use
- Session management can be simplified for general use

### 2.2 Protocol System (`codex-protocol`)

**Purpose:** Type-safe event and message definitions

**Key Files:**
- `codex-rs/protocol/src/protocol.rs` - Core Op and Event types
- `codex-rs/protocol/src/models.rs` - LLM message types
- `codex-rs/protocol/src/items.rs` - Turn item types

**Core Concepts:**

1. **Operation Types (Inputs):**
```rust
pub enum Op {
    UserInput { items: Vec<UserInput> },
    UserTurn { items, cwd, approval_policy, sandbox_policy, model, ... },
    ExecApproval { id, decision },
    PatchApproval { id, decision },
    Interrupt,
    Shutdown,
    // ... more ops
}
```

2. **Event Types (Outputs):**
```rust
pub enum EventMsg {
    SessionConfigured(SessionConfiguredEvent),
    AgentMessageDelta(AgentMessageDeltaEvent),
    ItemStarted(ItemStartedEvent),
    ItemCompleted(ItemCompletedEvent),
    ExecApprovalRequest(ExecApprovalRequestEvent),
    TurnAborted(TurnAbortedEvent),
    // ... more events
}
```

3. **Turn Items:**
   - Text messages
   - Code changes (patches)
   - File operations
   - Shell commands
   - Web search results

**Framework Extraction Value:** ⭐⭐⭐⭐⭐
- Protocol types are the foundation of the framework
- Clean separation allows easy reuse
- Minimal dependencies on other components

### 2.3 Tool Orchestration System

**Purpose:** Execute and manage tool calls from the LLM

**Key Files:**
- `codex-rs/core/src/tools/orchestrator.rs` - Central tool execution
- `codex-rs/core/src/tools/registry.rs` - Tool registration and dispatch
- `codex-rs/core/src/tools/router.rs` - Tool routing logic
- `codex-rs/core/src/tools/handlers/` - Individual tool implementations

**Core Concepts:**

1. **Tool Lifecycle:**
```
Tool Call → Approval Check → Sandbox Selection → Execution → Retry Logic → Result
```

2. **Tool Registry:**
```rust
pub struct ToolRegistry {
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle(&self, invocation: ToolInvocation) 
        -> Result<ToolOutput, FunctionCallError>;
}
```

3. **Built-in Tools:**
   - `shell` / `bash` - Execute shell commands
   - `read_file` / `view` - File system operations
   - `str_replace` / `create` - File editing
   - `apply_patch` - Apply code patches
   - `mcp_tool` - Call MCP server tools
   - `plan` / `report_progress` - Task management

4. **Tool Orchestrator Pattern:**
   - Approval management (user consent)
   - Sandbox selection (security boundaries)
   - Retry logic (escalation on failure)
   - Error handling and telemetry

**Framework Extraction Value:** ⭐⭐⭐⭐⭐
- Tool system is the core value proposition
- Highly reusable across different agent types
- Clean abstraction with pluggable handlers

### 2.4 LLM Client Abstraction

**Purpose:** Unified interface to different LLM providers

**Key Files:**
- `codex-rs/core/src/client.rs` - Main `ModelClient` struct
- `codex-rs/core/src/client_common.rs` - Shared types and utilities
- `codex-rs/core/src/chat_completions.rs` - Chat completions API
- `codex-rs/core/src/model_provider_info.rs` - Provider configurations

**Core Concepts:**

1. **Client Interface:**
```rust
pub struct ModelClient {
    config: Arc<Config>,
    auth_manager: Option<Arc<AuthManager>>,
    client: reqwest::Client,
    provider: ModelProviderInfo,
    // ...
}

impl ModelClient {
    pub async fn stream(&self, prompt: &Prompt) -> Result<ResponseStream>;
}
```

2. **Provider Support:**
   - OpenAI (Responses API and Chat Completions)
   - Anthropic
   - Custom providers (via base URL override)
   - Local models (Ollama)

3. **Streaming Architecture:**
   - Server-Sent Events (SSE) for real-time responses
   - Aggregation of token deltas
   - Tool call extraction from stream
   - Rate limit tracking

4. **Context Management:**
   - Token counting and tracking
   - Context window management
   - Auto-compaction when approaching limits

**Framework Extraction Value:** ⭐⭐⭐⭐
- Essential for any LLM application
- Already well abstracted
- Supports multiple providers

### 2.5 Sandbox & Approval System

**Purpose:** Security and user control over agent actions

**Key Files:**
- `codex-rs/core/src/tools/sandboxing.rs` - Approval abstractions
- `codex-rs/core/src/sandboxing/` - Platform-specific sandboxes
- `codex-rs/core/src/seatbelt.rs` - macOS Seatbelt
- `codex-rs/core/src/landlock.rs` - Linux Landlock
- `codex-rs/linux-sandbox/` - Standalone Linux sandbox

**Core Concepts:**

1. **Approval Policies:**
```rust
pub enum AskForApproval {
    Untrusted,     // Always ask
    OnFailure,     // Ask only if command exits non-zero
    OnRequest,     // Ask if agent explicitly requests
    Never,         // Never ask (dangerous!)
}
```

2. **Sandbox Policies:**
```rust
pub enum SandboxPolicy {
    ReadOnly,              // Read-only file system access
    WorkspaceWrite,        // Write within workspace only
    DangerFullAccess,      // Full system access
}
```

3. **Platform Sandboxing:**
   - **macOS:** Seatbelt profiles for file/network restrictions
   - **Linux:** Landlock LSM for path-based access control
   - **Generic:** Process isolation and capability restrictions

4. **Approval Flow:**
   - Tool requests approval before execution
   - User reviews command/action
   - Decision: Approve, Deny, or Abort entire turn
   - Session-level approval caching

**Framework Extraction Value:** ⭐⭐⭐
- Critical for production use
- May be optional for some framework users
- Platform-specific code can be challenging

### 2.6 Configuration System

**Purpose:** Flexible configuration management

**Key Files:**
- `codex-rs/core/src/config.rs` - Main `Config` struct
- `codex-rs/core/src/config_types.rs` - Type definitions
- `codex-rs/core/src/config_loader/` - Configuration loading

**Core Concepts:**

1. **Configuration Structure:**
```rust
pub struct Config {
    pub model: String,
    pub model_provider: String,
    pub approval_policy: AskForApproval,
    pub sandbox_policy: SandboxPolicy,
    pub cwd: PathBuf,
    pub mcp_servers: HashMap<String, McpServerConfig>,
    // ... many more fields
}
```

2. **Configuration Sources:**
   - TOML file (`~/.codex/config.toml`)
   - Environment variables
   - CLI arguments
   - Profile-based overrides

3. **MCP Server Configuration:**
   - Server launchers (command + args)
   - Authentication settings
   - Environment variables

**Framework Extraction Value:** ⭐⭐⭐⭐
- Essential for framework flexibility
- Can be simplified for framework use
- Profile system is useful pattern

### 2.7 MCP (Model Context Protocol) Integration

**Purpose:** Extensibility through standardized protocol

**Key Files:**
- `codex-rs/rmcp-client/` - MCP client implementation
- `codex-rs/mcp-server/` - MCP server implementation
- `codex-rs/mcp-types/` - MCP type definitions

**Core Concepts:**

1. **MCP Client:**
   - Connects to external MCP servers
   - Exposes their tools to the agent
   - Handles resources and prompts

2. **MCP Server:**
   - Exposes Codex as a tool to other systems
   - Enables multi-agent workflows
   - Provides `codex` and `codex-reply` tools

3. **Integration Points:**
   - Tools from MCP servers appear alongside built-in tools
   - Resources can be referenced in prompts
   - Prompts can be loaded from MCP servers

**Framework Extraction Value:** ⭐⭐⭐⭐
- MCP is a standard protocol for agent extensibility
- Valuable for framework users
- Already well-separated

---

## 3. Agent Execution Flow

### 3.1 Interactive Session Flow

```
1. User starts session
   └─> Codex::spawn() creates session

2. User submits prompt
   └─> Op::UserTurn sent to submission queue

3. Session processes turn
   ├─> Load conversation history
   ├─> Build system prompt with instructions
   ├─> Add user message to context
   └─> Call ModelClient::stream()

4. Model responds with tool calls
   ├─> Parse tool calls from SSE stream
   ├─> For each tool call:
   │   ├─> Check if approval needed
   │   ├─> If yes: emit ExecApprovalRequest event
   │   ├─> Wait for ExecApproval submission
   │   ├─> Select sandbox based on policy
   │   ├─> Execute tool in sandbox
   │   ├─> If sandbox denies, potentially retry without sandbox
   │   └─> Emit ItemCompleted event with result
   └─> Send tool results back to model

5. Model generates final response
   ├─> Stream tokens to user (AgentMessageDelta events)
   └─> Emit TurnCompleted event

6. Save conversation state
   └─> Update session rollout on disk

7. Loop back to step 2 for next turn
```

### 3.2 Non-Interactive (Exec) Flow

```
1. User runs: codex exec "prompt"
   └─> ExecCli parses arguments

2. Create Codex session
   ├─> Set approval_policy = Never (no user interaction)
   ├─> Set appropriate sandbox_policy
   └─> Spawn session

3. Submit single UserTurn
   └─> Include initial prompt

4. Process events until TurnCompleted
   ├─> Stream agent responses to stdout
   ├─> Show tool executions
   └─> Handle errors

5. Exit with status code
   └─> 0 = success, non-zero = error
```

### 3.3 Tool Execution Flow (Detailed)

```
Tool Call Received
    │
    ├─> 1. Approval Check
    │   ├─> Needs approval? (based on policy + tool preferences)
    │   ├─> If yes: emit approval request → wait for user decision
    │   └─> If approved or no approval needed: continue
    │
    ├─> 2. Sandbox Selection
    │   ├─> Tool declares preference (None, ReadOnly, Write)
    │   ├─> Policy overrides (ReadOnly, WorkspaceWrite, FullAccess)
    │   └─> Select effective sandbox level
    │
    ├─> 3. First Attempt
    │   ├─> Execute tool in selected sandbox
    │   └─> Result?
    │       ├─> Success: return result
    │       └─> Sandbox Denied: proceed to retry logic
    │
    ├─> 4. Retry Logic (on sandbox denial)
    │   ├─> Check if tool supports escalation
    │   ├─> Check if policy allows retry (not Never/OnRequest)
    │   ├─> If both yes: ask for approval to retry without sandbox
    │   └─> If approved: execute without sandbox
    │
    └─> 5. Return Result
        ├─> Format result for model
        ├─> Log telemetry
        └─> Emit ItemCompleted event
```

---

## 4. Key Technologies & Dependencies

### 4.1 Core Rust Dependencies

| Crate | Purpose | Framework Critical? |
|-------|---------|---------------------|
| `tokio` | Async runtime | Yes - Essential |
| `async-channel` | Event queues | Yes - Core pattern |
| `reqwest` | HTTP client | Yes - LLM API calls |
| `serde` / `serde_json` | Serialization | Yes - Protocol |
| `anyhow` / `thiserror` | Error handling | Yes - Error types |
| `tracing` | Logging | Yes - Observability |
| `eventsource-stream` | SSE parsing | Yes - Streaming |
| `rmcp` | MCP protocol | Optional - Extensibility |
| `landlock` | Linux sandbox | Optional - Security |
| `tempfile` | Temp files | Yes - File ops |
| `regex-lite` | Pattern matching | Moderate - Utils |

### 4.2 Platform-Specific Dependencies

- **macOS:** Core Foundation (Seatbelt profiles)
- **Linux:** Landlock, Seccomp (sandboxing)
- **Cross-platform:** `which`, `dirs`, `dunce`

### 4.3 Optional Components

- **TUI:** ratatui, crossterm (not needed for framework)
- **CLI:** clap (user can choose their own CLI framework)
- **Backend services:** OpenTelemetry, Sentry (optional telemetry)

---

## 5. Standalone CLI Environment

### 5.1 Entry Points

1. **Main Binary** (`codex-rs/cli/src/main.rs`):
   - Parses CLI arguments with `clap`
   - Routes to appropriate subcommand
   - Handles auth and configuration

2. **Interactive TUI** (`codex-rs/tui/`):
   - Full-screen terminal interface
   - Event loop for user interaction
   - Not needed for framework

3. **Exec Mode** (`codex-rs/exec/`):
   - Non-interactive automation
   - Simple stdio interaction
   - Good pattern for framework use

4. **MCP Server** (`codex-rs/mcp-server/`):
   - Exposes Codex over MCP
   - Useful reference for framework

### 5.2 Data Storage

- **Config:** `~/.codex/config.toml`
- **Sessions:** `~/.codex/sessions/{id}/`
- **Logs:** `~/.codex/log/`
- **Auth tokens:** System keychain (via `keyring` crate)

### 5.3 Authentication

- **ChatGPT Login:** OAuth device flow
- **API Key:** Direct OpenAI API key
- **Auth Manager:** Handles token refresh and storage

---

## 6. Framework Extraction Strategy

### 6.1 Design Principles

1. **Minimal Complexity:** Extract only essential components
2. **Surgical Changes:** Preserve existing codebase stability
3. **Composability:** Enable users to pick and choose components
4. **Compatibility:** Framework should be usable alongside full Codex CLI
5. **Documentation:** Clear examples and migration paths

### 6.2 Core Framework Components (MVP)

#### Phase 1: Foundation
- ✅ Protocol types (Op, Event, models)
- ✅ Event system (async channels)
- ✅ Basic configuration
- ✅ Error types

#### Phase 2: LLM Integration
- ✅ ModelClient abstraction
- ✅ Streaming response handling
- ✅ Provider configuration
- ✅ Context management

#### Phase 3: Agent Core
- ✅ Agent session management
- ✅ Turn processing
- ✅ Message history
- ✅ Tool call orchestration

#### Phase 4: Tool System
- ✅ Tool registry and handler trait
- ✅ Basic tool implementations (shell, file ops)
- ✅ Tool orchestrator (approval + sandbox)
- ✅ MCP client integration

#### Phase 5: Optional Features
- ⚠️ Advanced sandboxing (platform-specific)
- ⚠️ TUI components (if needed)
- ⚠️ Telemetry integration
- ⚠️ Full configuration system

### 6.3 Framework Architecture

```
codex-agent-framework/
├── codex-agent-core/           # Main framework crate
│   ├── protocol/               # Event types and protocol
│   ├── client/                 # LLM client abstraction
│   ├── agent/                  # Core agent logic
│   ├── tools/                  # Tool system
│   └── config/                 # Basic configuration
├── codex-agent-tools/          # Standard tool implementations
│   ├── shell/                  # Shell execution
│   ├── file/                   # File operations
│   └── web/                    # Web search (optional)
├── codex-agent-mcp/            # MCP integration (optional)
├── codex-agent-sandbox/        # Sandboxing utilities (optional)
└── examples/
    ├── minimal/                # Bare minimum agent
    ├── code-assistant/         # Code review agent
    └── chat-bot/               # Simple chat agent
```

### 6.4 API Design

**Simple agent example:**

```rust
use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create agent with minimal config
    let config = AgentConfig::builder()
        .model("gpt-4")
        .api_key_from_env()
        .tools(StandardTools::default())
        .build()?;
    
    let agent = Agent::new(config).await?;
    
    // 2. Run a turn
    let response = agent
        .run("Analyze this codebase and suggest improvements")
        .await?;
    
    println!("Response: {}", response.text);
    
    Ok(())
}
```

**Streaming example:**

```rust
let mut stream = agent
    .run_streaming("Fix all linting errors")
    .await?;

while let Some(event) = stream.next().await {
    match event {
        AgentEvent::Message(delta) => print!("{}", delta),
        AgentEvent::ToolCall(call) => println!("\nRunning: {}", call.name),
        AgentEvent::Complete => break,
        _ => {}
    }
}
```

**Custom tool example:**

```rust
use codex_agent_core::{Tool, ToolContext, ToolResult};

struct CustomTool;

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> &str { "custom_tool" }
    
    fn description(&self) -> &str {
        "A custom tool that does something"
    }
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        // Custom tool logic
        Ok(ToolResult::text("Custom tool executed"))
    }
}

// Register with agent
let agent = Agent::builder()
    .tool(CustomTool)
    .build()?;
```

### 6.5 Extraction Methodology

1. **Copy Core Types:** Start with protocol types (low risk)
2. **Extract Abstractions:** Create new traits for core interfaces
3. **Implement Adapters:** Bridge between framework and existing code
4. **Test Incrementally:** Ensure each layer works before moving up
5. **Document Extensively:** Clear examples for each component
6. **Version Carefully:** Semantic versioning with clear stability guarantees

---

## 7. Component Dependency Analysis

### 7.1 Core Dependencies (Must Extract)

```
codex-protocol (standalone crate)
    ↓
codex-agent-core
    ├─> LLM Client (requires: reqwest, eventsource-stream, serde)
    ├─> Agent Logic (requires: tokio, async-channel)
    └─> Tool System (requires: async-trait)
        └─> codex-agent-tools
            └─> Standard Tools (requires: tempfile, which, etc.)
```

### 7.2 Optional Dependencies

```
codex-agent-mcp (optional)
    └─> requires: rmcp

codex-agent-sandbox (optional)
    └─> requires: landlock (Linux), core-foundation (macOS)

codex-agent-telemetry (optional)
    └─> requires: opentelemetry, tracing
```

### 7.3 Avoided Dependencies

- `codex-tui` - UI layer not needed
- `codex-cli` - CLI specifics not needed
- `codex-backend-client` - Internal OpenAI services
- Platform-specific sandboxing (initially optional)

---

## 8. Testing Strategy

### 8.1 Unit Tests

- Protocol type serialization/deserialization
- Event queue behavior
- Tool handler execution
- Configuration parsing

### 8.2 Integration Tests

- End-to-end agent sessions
- Tool orchestration flows
- Approval and sandbox logic
- MCP integration

### 8.3 Example Applications

- Minimal agent (5-10 lines)
- Code review agent
- File processor agent
- Chat agent with memory

---

## 9. Documentation Plan

### 9.1 Framework Documentation

1. **Getting Started Guide**
   - Installation
   - First agent in 5 minutes
   - Core concepts

2. **Architecture Guide**
   - Component overview
   - Event system
   - Tool system
   - Configuration

3. **API Reference**
   - Agent API
   - Tool API
   - Configuration API
   - Protocol types

4. **Advanced Topics**
   - Custom tools
   - MCP integration
   - Sandboxing
   - Telemetry

### 9.2 Migration Guide

- Moving from full Codex CLI to framework
- Compatibility considerations
- Feature parity matrix

---

## 10. Risks and Mitigation

### 10.1 Risks

1. **Over-extraction:** Taking too much complexity
   - **Mitigation:** Start with MVP, add features incrementally

2. **Under-extraction:** Missing essential features
   - **Mitigation:** Build real examples to validate completeness

3. **Breaking Changes:** Disrupting existing codebase
   - **Mitigation:** Copy, don't move; maintain separation

4. **Maintenance Burden:** Divergence between framework and CLI
   - **Mitigation:** Share core protocol types, clear ownership

5. **Platform Differences:** Sandbox and tool variations
   - **Mitigation:** Graceful degradation, feature detection

### 10.2 Success Criteria

- ✅ Framework compiles without CLI dependencies
- ✅ All examples run successfully
- ✅ Documentation is clear and complete
- ✅ PoC application demonstrates value
- ✅ No regressions in existing codebase

---

## 11. Implementation Timeline

### Phase 1: Foundation (Day 1)
- Create framework crate structure
- Extract protocol types
- Set up basic CI/CD
- Write initial documentation

### Phase 2: Core Agent (Day 2)
- Extract LLM client
- Implement agent session management
- Add basic tool system
- Create minimal example

### Phase 3: Tools & Features (Day 3)
- Implement standard tools
- Add approval system
- Optional: MCP integration
- Create code assistant example

### Phase 4: Polish & Demo (Day 4)
- Write comprehensive docs
- Build PoC application
- Performance testing
- Final review and cleanup

---

## 12. Conclusion

The Codex CLI codebase is well-architected with clear separation of concerns, making it an excellent candidate for framework extraction. The event-driven design, tool orchestration system, and LLM client abstraction are particularly valuable components.

By following a surgical extraction approach focused on core abstractions, we can create a lightweight, composable agent framework that enables developers to build sophisticated LLM applications without the complexity of the full CLI system.

The key to success is maintaining minimal complexity while preserving the essential features that make Codex powerful: tool orchestration, streaming responses, approval workflows, and extensibility through MCP.

---

## Appendix A: Key Files Reference

### Core Files to Extract From

1. **Protocol & Types:**
   - `codex-rs/protocol/src/protocol.rs`
   - `codex-rs/protocol/src/models.rs`
   - `codex-rs/protocol/src/items.rs`

2. **Agent Core:**
   - `codex-rs/core/src/codex.rs`
   - `codex-rs/core/src/state.rs`
   - `codex-rs/core/src/conversation_history.rs`

3. **LLM Client:**
   - `codex-rs/core/src/client.rs`
   - `codex-rs/core/src/client_common.rs`
   - `codex-rs/core/src/chat_completions.rs`

4. **Tool System:**
   - `codex-rs/core/src/tools/orchestrator.rs`
   - `codex-rs/core/src/tools/registry.rs`
   - `codex-rs/core/src/tools/handlers/`

5. **Configuration:**
   - `codex-rs/core/src/config.rs` (simplified)
   - `codex-rs/core/src/config_types.rs`

### Files to Avoid

- `codex-rs/tui/` - UI layer
- `codex-rs/cli/` - CLI specifics
- `codex-rs/backend-client/` - Internal services
- `codex-rs/chatgpt/` - Auth flows (can abstract)

---

## Appendix B: Glossary

- **Op:** Operation/submission from user to agent
- **Event:** Output event from agent to user
- **Turn:** One complete interaction cycle (user input → agent response)
- **Tool:** Function the agent can call (shell, file ops, etc.)
- **MCP:** Model Context Protocol - standard for agent extensibility
- **Sandbox:** Security boundary for tool execution
- **Approval:** User consent required for certain actions
- **Stream:** Real-time SSE response from LLM
- **Session:** Persistent conversation context
- **Rollout:** Saved session state on disk
