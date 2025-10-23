//! Chat bot example
//!
//! This example shows how to create a simple chat bot that maintains conversation history.
//!
//! Set your API key:
//! ```
//! export OPENAI_API_KEY=sk-...
//! ```
//!
//! Then run:
//! ```
//! cargo run --example chat_bot
//! ```

use codex_agent_core::{Agent, AgentConfig};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("💬 Chat Bot");
    println!("===========\n");
    println!("Type your messages and press Enter. Type 'quit' to exit.\n");

    // Create agent
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .build()?
    ).await?;

    println!("Session ID: {}\n", agent.session_id());

    // Chat loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("\nGoodbye! 👋");
            break;
        }

        print!("Bot: ");
        io::stdout().flush()?;

        match agent.run(input).await {
            Ok(response) => {
                println!("{}\n", response.text);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
            }
        }
    }

    Ok(())
}
