//! Dependency wiring: the only place that knows which adapter implements which port.

use std::path::PathBuf;
use std::sync::Arc;

use cerebro_data::datasources::{ClaudeRegistrySource, HookAgentSource, HookSessionSource};
use cerebro_data::repositories::{EmbeddedCatalog, JsonAssignmentStore};
use cerebro_data::services::{ClaudeSettingsHookInstaller, PsProcessInspector, SystemClock, SystemCommandRunner};
use cerebro_data::strategies::focus::platform_focuser;
use cerebro_domain::DomainError;
use cerebro_domain::ports::CatalogSource;
use cerebro_domain::usecases::{FocusSession, ManageHooks, ObserveSessions, ObserveSessionsDeps};
use cerebro_protocol::Paths;

pub struct Container {
    pub observe: ObserveSessions,
    pub focus: FocusSession,
    pub hooks: ManageHooks,
}

pub fn build() -> Result<Container, DomainError> {
    let paths = Paths::detect().ok_or_else(|| DomainError::Platform("cannot find the home directory".into()))?;
    let runner = Arc::new(SystemCommandRunner);

    let observe = ObserveSessions::new(ObserveSessionsDeps {
        registry: Arc::new(ClaudeRegistrySource::new(paths.claude_sessions_dir())),
        hooks: Arc::new(HookSessionSource::new(paths.hook_sessions_dir())),
        agents: Arc::new(HookAgentSource::new(paths.clone())),
        processes: Arc::new(PsProcessInspector::new(runner.clone())),
        store: Arc::new(JsonAssignmentStore::new(paths.assignments_file())),
        clock: Arc::new(SystemClock),
        catalog: EmbeddedCatalog.catalog()?,
    });

    Ok(Container {
        observe,
        focus: FocusSession::new(platform_focuser(runner)),
        hooks: ManageHooks::new(Arc::new(ClaudeSettingsHookInstaller::new(
            paths.claude_settings_file(),
            hook_binary_path(),
        ))),
    })
}

/// The hook ships as a sidecar next to the app executable
/// (`Cerebro.app/Contents/MacOS/cerebro-hook` once bundled, `target/<profile>/` in dev).
fn hook_binary_path() -> PathBuf {
    let name = if cfg!(windows) { "cerebro-hook.exe" } else { "cerebro-hook" };
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}
