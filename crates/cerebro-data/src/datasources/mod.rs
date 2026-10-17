//! Session sources, one per AI tool, and the readers behind them.

mod claude_code;
mod claude_registry;
mod hook_agents;
mod hook_sessions;

pub use claude_code::{
    AgentReader, CLAUDE_CODE, ClaudeCodeDeps, ClaudeCodeSource, ObservedSession, SessionReader,
};
pub use claude_registry::{ClaudeRegistrySource, RegistryProblem, RegistryScan};
pub use hook_agents::HookAgentSource;
pub use hook_sessions::HookSessionSource;
