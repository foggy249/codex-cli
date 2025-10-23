//! Minimal agent example
//!
//! This is the simplest possible agent - just a few lines to get started.
//!
//! Set your API key:
//! ```
//! export OPENAI_API_KEY=sk-...
//! ```
//!
//! Then run:
//! ```
//! cargo run --example minimal
//! ```

use codex_agent_core::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create agent with minimal configuration
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()?
    ).await?;

    // Run a simple query
    let response = agent.run("What is 2+2? Just give me the answer.").await?;
    
    println!("\nResponse: {}", response.text);

    Ok(())
}
