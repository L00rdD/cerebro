//! One struct per application action. Each takes its dependencies as trait objects.

mod focus_session;
mod manage_hooks;
mod manage_network;
mod observe_sessions;

pub use focus_session::FocusSession;
pub use manage_hooks::ManageHooks;
pub use manage_network::ManageNetwork;
pub use observe_sessions::{ObserveSessions, ObserveSessionsDeps};
