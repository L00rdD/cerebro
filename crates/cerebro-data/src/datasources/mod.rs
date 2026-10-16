//! Readers for session and agent state on disk.

mod claude_registry;
mod hook_agents;
mod hook_sessions;

pub use claude_registry::{ClaudeRegistrySource, RegistryProblem, RegistryScan};
pub use hook_agents::HookAgentSource;
pub use hook_sessions::HookSessionSource;
