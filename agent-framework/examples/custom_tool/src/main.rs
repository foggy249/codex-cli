//! Custom Tool Tutorial
//!
//! This example shows how to create and use custom tools with the agent framework.
//! We'll create several example tools to demonstrate different patterns.

use async_trait::async_trait;
use codex_agent_core::{Agent, AgentConfig, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};

// ============================================================================
// Example 1: Simple Tool (no external dependencies)
// ============================================================================

/// A simple calculator tool that performs basic arithmetic
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic arithmetic operations (add, subtract, multiply, divide)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number",
                    "description": "Second number"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let operation = args["operation"].as_str().unwrap();
        let a = args["a"].as_f64().unwrap();
        let b = args["b"].as_f64().unwrap();

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult::error("Cannot divide by zero"));
                }
                a / b
            }
            _ => return Ok(ToolResult::error("Unknown operation")),
        };

        Ok(ToolResult::text(format!("{} {} {} = {}", a, operation, b, result)))
    }
}

// ============================================================================
// Example 2: Tool with External API Call
// ============================================================================

/// A tool that fetches random facts from an API
struct RandomFactTool;

#[async_trait]
impl Tool for RandomFactTool {
    fn name(&self) -> &str {
        "random_fact"
    }

    fn description(&self) -> &str {
        "Get a random interesting fact"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "enum": ["science", "history", "nature"],
                    "description": "Category of fact to retrieve"
                }
            }
        })
    }

    async fn execute(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let category = args.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("science");

        // Simulate API call (replace with real API in production)
        let fact = match category {
            "science" => "The speed of light is approximately 299,792,458 meters per second.",
            "history" => "The first computer programmer was Ada Lovelace in 1843.",
            "nature" => "Honey never spoils and can last for thousands of years.",
            _ => "Did you know? The framework you're using is pretty cool!",
        };

        Ok(ToolResult::text(format!(
            "Random {} fact: {}",
            category, fact
        )))
    }
}

// ============================================================================
// Example 3: Tool with Context Usage
// ============================================================================

/// A tool that demonstrates using the ToolContext
struct FileStatsTool;

#[async_trait]
impl Tool for FileStatsTool {
    fn name(&self) -> &str {
        "file_stats"
    }

    fn description(&self) -> &str {
        "Get statistics about files in the working directory"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(
        &self,
        _args: Value,
        ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let working_dir = &ctx.cwd;

        // Count files in directory
        let entries = std::fs::read_dir(working_dir)?;
        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    file_count += 1;
                } else if entry.path().is_dir() {
                    dir_count += 1;
                }
            }
        }

        Ok(ToolResult::text(format!(
            "Working directory: {}\nFiles: {}\nDirectories: {}",
            working_dir.display(),
            file_count,
            dir_count
        )))
    }
}

// ============================================================================
// Example 4: Tool with Structured Output
// ============================================================================

/// A tool that returns structured JSON data
struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get weather information for a location (simulated)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or location"
                }
            },
            "required": ["location"]
        })
    }

    async fn execute(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let location = args["location"].as_str().unwrap();

        // Simulate weather data (replace with real API)
        let weather_data = json!({
            "location": location,
            "temperature": 72,
            "conditions": "Partly cloudy",
            "humidity": 65,
            "wind_speed": 10
        });

        Ok(ToolResult::json(weather_data))
    }
}

// ============================================================================
// Example 5: Tool with Approval Requirements
// ============================================================================

/// A tool that requires approval for sensitive operations
struct SensitiveTool;

#[async_trait]
impl Tool for SensitiveTool {
    fn name(&self) -> &str {
        "sensitive_operation"
    }

    fn description(&self) -> &str {
        "Perform a sensitive operation that requires approval"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "The sensitive action to perform"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> anyhow::Result<ToolResult> {
        let action = args["action"].as_str().unwrap();

        Ok(ToolResult::text(format!(
            "✓ Sensitive operation completed: {}",
            action
        )))
    }

    fn requires_approval(&self, ctx: &ToolContext) -> bool {
        // Always require approval for this tool
        matches!(
            ctx.approval_policy,
            codex_agent_core::AskForApproval::UnlessTrusted
        )
    }
}

// ============================================================================
// Main Example
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 Custom Tool Tutorial");
    println!("=======================\n");

    // Create an agent with all custom tools
    let agent = Agent::new(
        AgentConfig::builder()
            .model("gpt-4")
            .api_key_from_env()
            .tool(CalculatorTool)
            .tool(RandomFactTool)
            .tool(FileStatsTool)
            .tool(WeatherTool)
            .tool(SensitiveTool)
            .build()?
    ).await?;

    println!("Agent created with 5 custom tools\n");

    // Example 1: Simple calculation
    println!("📝 Example 1: Calculator Tool\n");
    let response = agent.run(
        "Use the calculator to compute 42 * 137"
    ).await?;
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 2: API call simulation
    println!("📝 Example 2: Random Fact Tool\n");
    let response = agent.run(
        "Tell me a random science fact"
    ).await?;
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 3: Context usage
    println!("📝 Example 3: File Stats Tool\n");
    let response = agent.run(
        "Show me statistics about the current directory"
    ).await?;
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 4: Structured output
    println!("📝 Example 4: Weather Tool\n");
    let response = agent.run(
        "What's the weather like in San Francisco?"
    ).await?;
    println!("Response: {}\n", response.text);
    println!("---\n");

    // Example 5: Multiple tools in sequence
    println!("📝 Example 5: Using Multiple Tools\n");
    let response = agent.run(
        "Calculate 10 + 20, then tell me a history fact, then show directory stats"
    ).await?;
    println!("Response: {}\n", response.text);

    println!("\n✅ All examples completed!");
    println!("\n💡 Key Takeaways:");
    println!("   1. Tools are easy to create with the Tool trait");
    println!("   2. Use async/await for any I/O operations");
    println!("   3. Return ToolResult::text() or ToolResult::json()");
    println!("   4. Access context like working directory via ToolContext");
    println!("   5. Control approval requirements with requires_approval()");

    Ok(())
}
