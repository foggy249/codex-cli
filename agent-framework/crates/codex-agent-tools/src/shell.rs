//! Shell execution tool

use async_trait::async_trait;
use codex_agent_core::{Tool, ToolContext, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Tool for executing shell commands
pub struct ShellTool;

#[derive(Serialize, Deserialize)]
struct ShellArgs {
    command: String,
    #[serde(default)]
    description: String,
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return its output. Use this to run commands, scripts, or CLI tools."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "description": {
                    "type": "string",
                    "description": "A brief description of what the command does"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> anyhow::Result<ToolResult> {
        let args: ShellArgs = serde_json::from_value(args)?;

        // Determine shell
        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let shell_flag = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        // Execute command
        let mut child = Command::new(shell)
            .arg(shell_flag)
            .arg(&args.command)
            .current_dir(&ctx.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Read output
        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(mut out) = child.stdout.take() {
            out.read_to_string(&mut stdout).await?;
        }

        if let Some(mut err) = child.stderr.take() {
            err.read_to_string(&mut stderr).await?;
        }

        let status = child.wait().await?;

        // Format output
        let mut output = String::new();
        
        if !stdout.is_empty() {
            output.push_str(&stdout);
        }
        
        if !stderr.is_empty() {
            if !output.is_empty() {
                output.push_str("\n--- stderr ---\n");
            }
            output.push_str(&stderr);
        }

        if !status.success() {
            output.push_str(&format!("\nExit code: {}", status.code().unwrap_or(-1)));
        }

        Ok(ToolResult::text(output))
    }

    fn requires_approval(&self, ctx: &ToolContext) -> bool {
        matches!(
            ctx.approval_policy,
            codex_agent_core::AskForApproval::UnlessTrusted
        )
    }
}
