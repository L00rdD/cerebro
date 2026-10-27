//! OS-facing helpers.

mod clock;
mod command_runner;
mod hook_installer;
mod process_inspector;
mod zerotier;

pub use clock::SystemClock;
pub use command_runner::{CommandOutput, CommandRunner, SystemCommandRunner};
pub use hook_installer::ClaudeSettingsHookInstaller;
pub use process_inspector::{
    ProcessInfo, ProcessInspector, PsLine, PsProcessInspector, parse_ps_line,
    parse_ps_line_borrowed,
};
pub use zerotier::ZeroTierNetwork;
