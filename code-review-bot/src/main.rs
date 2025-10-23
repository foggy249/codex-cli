//! Code Review Bot - A Proof-of-Concept Application
//!
//! This bot uses the Codex Agent Framework to perform automated code reviews.
//! It can analyze entire directories, individual files, or specific changes,
//! and provide actionable feedback on code quality, potential bugs, and best practices.

use clap::{Parser, Subcommand};
use codex_agent_core::{Agent, AgentConfig, AskForApproval};
use codex_agent_tools::StandardTools;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "code-review-bot")]
#[command(about = "AI-powered code review bot using the Codex Agent Framework")]
struct Cli {
    /// OpenAI API key (or set OPENAI_API_KEY env var)
    #[arg(long)]
    api_key: Option<String>,

    /// Model to use
    #[arg(long, default_value = "gpt-4")]
    model: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Review a specific file
    File {
        /// Path to the file to review
        path: PathBuf,
    },

    /// Review an entire directory
    Dir {
        /// Path to the directory to review
        path: PathBuf,

        /// File extensions to include (e.g., "rs,py,js")
        #[arg(long, default_value = "rs,py,js,ts,go,java")]
        extensions: String,
    },

    /// Interactive code review session
    Interactive {
        /// Working directory
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Build agent configuration
    let mut config_builder = AgentConfig::builder()
        .model(&cli.model)
        .approval_policy(AskForApproval::Never)
        .tools(StandardTools::code_tools());

    if let Some(api_key) = cli.api_key {
        config_builder = config_builder.api_key(api_key);
    } else {
        config_builder = config_builder.api_key_from_env();
    }

    match cli.command {
        Commands::File { path } => {
            review_file(config_builder, path).await?;
        }
        Commands::Dir { path, extensions } => {
            review_directory(config_builder, path, extensions).await?;
        }
        Commands::Interactive { dir } => {
            interactive_review(config_builder, dir).await?;
        }
    }

    Ok(())
}

async fn review_file(
    config_builder: codex_agent_core::AgentConfigBuilder,
    path: PathBuf,
) -> anyhow::Result<()> {
    println!("🔍 Code Review Bot - File Review");
    println!("=================================\n");
    println!("Reviewing: {}\n", path.display());

    let config = config_builder
        .working_directory(path.parent().unwrap_or(std::path::Path::new(".")))
        .build()?;

    let agent = Agent::new(config).await?;

    let prompt = format!(
        "Please review the file '{}' and provide:\n\
        1. Overall code quality assessment\n\
        2. Potential bugs or issues\n\
        3. Best practice violations\n\
        4. Suggestions for improvement\n\
        5. Security concerns (if any)\n\n\
        Be specific and provide code examples where relevant.",
        path.display()
    );

    let response = agent.run(prompt).await?;

    println!("📋 Review Results:");
    println!("{}\n", "=".repeat(80));
    println!("{}", response.text);
    println!("{}\n", "=".repeat(80));

    Ok(())
}

async fn review_directory(
    config_builder: codex_agent_core::AgentConfigBuilder,
    path: PathBuf,
    extensions: String,
) -> anyhow::Result<()> {
    println!("🔍 Code Review Bot - Directory Review");
    println!("======================================\n");
    println!("Reviewing directory: {}", path.display());
    println!("File extensions: {}\n", extensions);

    let config = config_builder.working_directory(&path).build()?;

    let agent = Agent::new(config).await?;

    // Step 1: Get list of files
    println!("📁 Step 1: Finding files to review...\n");

    let ext_list = extensions.replace(',', " -o -name '*.");
    let find_cmd = format!(
        "find . -type f \\( -name '*.{}' \\) | head -20",
        ext_list
    );

    let response = agent
        .run(format!("Use the shell tool to list files to review: {find_cmd}"))
        .await?;

    println!("Files found:\n{}\n", response.text);

    // Step 2: Review architecture
    println!("🏗️  Step 2: Analyzing project architecture...\n");

    let response = agent
        .run(
            "Based on the files you found, describe the project architecture \
            and identify the main components. What patterns or frameworks are being used?",
        )
        .await?;

    println!("Architecture Analysis:");
    println!("{}\n", "=".repeat(80));
    println!("{}\n", response.text);
    println!("{}\n", "=".repeat(80));

    // Step 3: Code quality review
    println!("✅ Step 3: Reviewing code quality...\n");

    let response = agent
        .run(
            "Review the code in this project for:\n\
            1. Common code smells\n\
            2. Potential bugs or security issues\n\
            3. Best practices compliance\n\
            4. Testing coverage (based on test files found)\n\
            5. Documentation quality\n\n\
            Provide a summary with specific examples.",
        )
        .await?;

    println!("Code Quality Review:");
    println!("{}\n", "=".repeat(80));
    println!("{}\n", response.text);
    println!("{}\n", "=".repeat(80));

    // Step 4: Recommendations
    println!("💡 Step 4: Generating recommendations...\n");

    let response = agent
        .run(
            "Based on your analysis, provide top 5 actionable recommendations \
            to improve this codebase. Prioritize by impact and effort.",
        )
        .await?;

    println!("Recommendations:");
    println!("{}\n", "=".repeat(80));
    println!("{}\n", response.text);
    println!("{}\n", "=".repeat(80));

    Ok(())
}

async fn interactive_review(
    config_builder: codex_agent_core::AgentConfigBuilder,
    dir: PathBuf,
) -> anyhow::Result<()> {
    use std::io::{self, Write};

    println!("🔍 Code Review Bot - Interactive Mode");
    println!("======================================\n");
    println!("Working directory: {}", dir.display());
    println!("Type your questions or 'quit' to exit.\n");

    let config = config_builder.working_directory(&dir).build()?;

    let agent = Agent::new(config).await?;

    println!("Session ID: {}\n", agent.session_id());
    println!("Available commands:");
    println!("  - Ask about specific files or code patterns");
    println!("  - Request reviews of particular aspects");
    println!("  - Get architectural insights");
    println!("  - 'quit' or 'exit' to end session\n");

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
            println!("\n👋 Code review session ended. Thank you!");
            break;
        }

        print!("\n🤖 Bot: ");
        io::stdout().flush()?;

        match agent.run(input).await {
            Ok(response) => {
                println!("{}\n", response.text);
            }
            Err(e) => {
                eprintln!("❌ Error: {}\n", e);
            }
        }
    }

    Ok(())
}
