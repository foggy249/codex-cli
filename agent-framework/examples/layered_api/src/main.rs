//! Layered API Example
//!
//! This example demonstrates the different layers of abstraction available
//! in the framework, from simple text completion to full agent applications.

use codex_agent_core::client::{LlmClient, Message};
use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎯 Layered API Examples");
    println!("========================\n");

    // Create a client for lower-level operations
    let client = LlmClient::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
        "gpt-4",
        None,
    );

    // ========================================================================
    // Layer 1: Simple Text Completion (No History, No Tools)
    // ========================================================================
    println!("📝 Layer 1: Simple Text Completion");
    println!("   Use case: One-off completions, no context needed\n");

    let response = client.complete_simple("What is 2+2? Answer in one word.").await?;
    println!("   Q: What is 2+2?");
    println!("   A: {}\n", response);
    println!("   ✓ Simplest API - just prompt in, text out\n");
    println!("---\n");

    // ========================================================================
    // Layer 2: Chat Completion (With History, No Tools)
    // ========================================================================
    println!("📝 Layer 2: Chat Completion (with history)");
    println!("   Use case: Conversational AI, chatbots\n");

    let mut messages = vec![
        Message::user("My favorite color is blue."),
    ];
    
    let response = client.complete_chat(messages.clone()).await?;
    println!("   User: My favorite color is blue.");
    println!("   Assistant: {}", response.content);
    messages.push(response);

    messages.push(Message::user("What's my favorite color?"));
    let response = client.complete_chat(messages.clone()).await?;
    println!("   User: What's my favorite color?");
    println!("   Assistant: {}\n", response.content);
    
    println!("   ✓ Maintains conversation history");
    println!("   ✓ No tool calling\n");
    println!("---\n");

    // ========================================================================
    // Layer 3: Manual Tool Loop (With Tools, Manual Control)
    // ========================================================================
    println!("📝 Layer 3: Manual Tool Loop");
    println!("   Use case: Custom tool orchestration, fine-grained control\n");

    let tools = StandardTools::code_tools();
    let tool_specs: Vec<_> = tools.iter().map(|t| {
        use codex_agent_core::tools::{Tool, ToolSpec};
        ToolSpec {
            name: t.name().to_string(),
            description: t.description().to_string(),
            parameters: t.parameters_schema(),
        }
    }).collect();

    let mut messages = vec![
        Message::user("List files in the current directory using the shell tool."),
    ];

    println!("   User: List files in the current directory using the shell tool.");
    
    // Note: This is a demonstration of the API, not a complete implementation
    println!("   (In a real implementation, you would:");
    println!("    1. Call client.complete_with_tools()");
    println!("    2. Check for tool_calls in response");
    println!("    3. Execute tools manually");
    println!("    4. Add results back to messages");
    println!("    5. Loop until no more tool calls)\n");
    
    println!("   ✓ Full control over tool execution");
    println!("   ✓ Can add custom logic between steps");
    println!("   ✓ Inspect and modify tool calls\n");
    println!("---\n");

    // ========================================================================
    // Layer 4: Full Agent (Automatic Tool Loop)
    // ========================================================================
    println!("📝 Layer 4: Full Agent (automatic tool loop)");
    println!("   Use case: Autonomous agents, production applications\n");

    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .working_directory(".")
            .tools(StandardTools::read_only())
            .build()?
    ).await?;

    println!("   User: Count how many .md files are in the current directory.");
    let response = agent.run("Count how many .md files are in the current directory.").await?;
    println!("   Agent: {}\n", response.text);

    println!("   ✓ Automatic tool loop");
    println!("   ✓ Session management");
    println!("   ✓ Multi-turn conversations");
    println!("   ✓ Production ready\n");
    println!("---\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("📊 Summary of Layers:\n");
    println!("   Layer 1 (Simple):     client.complete_simple(prompt)");
    println!("   Layer 2 (Chat):       client.complete_chat(messages)");
    println!("   Layer 3 (Manual):     client.complete_with_tools(messages, tools)");
    println!("   Layer 4 (Agent):      agent.run(message)\n");

    println!("💡 Choose the layer that matches your needs:");
    println!("   - Simple tasks? Use Layer 1");
    println!("   - Chat without tools? Use Layer 2");
    println!("   - Custom orchestration? Use Layer 3");
    println!("   - Full agent? Use Layer 4\n");

    Ok(())
}
