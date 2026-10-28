//! OS-facing helpers.

mod clock;
mod command_runner;
mod hook_installer;
mod local_network;
mod process_inspector;
mod zerotier;

pub use clock::SystemClock;
pub use command_runner::{CommandOutput, CommandRunner, SystemCommandRunner};
pub use hook_installer::ClaudeSettingsHookInstaller;
pub use local_network::{LocalNetwork, lan_addresses};
pub use process_inspector::{
    ProcessInfo, ProcessInspector, PsLine, PsProcessInspector, parse_ps_line,
    parse_ps_line_borrowed,
};
pub use zerotier::ZeroTierNetwork;
