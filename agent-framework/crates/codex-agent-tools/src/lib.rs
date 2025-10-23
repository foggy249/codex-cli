//! Standard tool implementations for the agent framework

pub mod shell;
pub mod file;

use codex_agent_core::Tool;

/// Collection of standard tools
pub struct StandardTools;

impl StandardTools {
    /// Get all standard tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(shell::ShellTool),
            Box::new(file::ReadFileTool),
            Box::new(file::WriteFileTool),
        ]
    }

    /// Get only code-related tools
    pub fn code_tools() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(shell::ShellTool),
            Box::new(file::ReadFileTool),
            Box::new(file::WriteFileTool),
        ]
    }

    /// Get only read-only tools
    pub fn read_only() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(file::ReadFileTool),
        ]
    }
}
