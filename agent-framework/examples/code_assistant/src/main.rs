//! Code assistant example
//!
//! This example shows how to create an agent with tools that can read and analyze code.
//!
//! Set your API key:
//! ```
//! export OPENAI_API_KEY=sk-...
//! ```
//!
//! Then run:
//! ```
//! cargo run --example code_assistant
//! ```

use codex_agent_core::{Agent, AgentConfig};
use codex_agent_tools::StandardTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 Code Assistant Agent");
    println!("=======================\n");

    // Create agent with code tools
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .working_directory(".")
            .tools(StandardTools::code_tools())
            .build()?
    ).await?;

    println!("Agent initialized with session ID: {}\n", agent.session_id());

    // Example 1: Analyze a file
    println!("📝 Example 1: Analyzing this file...\n");
    
    let response = agent.run(
        "Read the file examples/code_assistant/src/main.rs and provide a brief summary of what it does."
    ).await?;
    
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 2: Check for Rust files
    println!("📝 Example 2: Listing Rust files...\n");
    
    let response = agent.run(
        "Use the shell tool to find all .rs files in the current directory and subdirectories. Show me the first 10."
    ).await?;
    
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 3: Multi-turn conversation
    println!("📝 Example 3: Multi-turn conversation...\n");
    
    let response = agent.run(
        "How many lines of code are in this example file?"
    ).await?;
    
    println!("Response: {}\n", response.text);

    Ok(())
}
