//! Protocol types and utilities
//!
//! Re-exports protocol types from the `codex-protocol` crate for convenience.

// Re-export all protocol types
pub use codex_protocol::protocol::*;
pub use codex_protocol::models::*;
pub use codex_protocol::items::*;
pub use codex_protocol::user_input::*;
pub use codex_protocol::ConversationId;

/// Helper to create a UserInput from a string
pub fn text_input(text: impl Into<String>) -> UserInput {
    UserInput::Text {
        text: text.into(),
    }
}

/// Helper to create a UserTurn Op
pub fn user_turn(
    text: impl Into<String>,
    config: &crate::config::AgentConfig,
) -> Op {
    Op::UserTurn {
        items: vec![text_input(text)],
        cwd: config.working_directory.clone(),
        approval_policy: config.approval_policy,
        sandbox_policy: config.sandbox_policy.clone(),
        model: config.model.clone(),
        effort: config.reasoning_effort,
        summary: config.reasoning_summary,
        final_output_json_schema: None,
    }
}
