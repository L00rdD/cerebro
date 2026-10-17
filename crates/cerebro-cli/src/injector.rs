use std::{path::PathBuf, sync::Arc};

use cerebro_data::{
    datasources::{
        ClaudeCodeDeps, ClaudeCodeSource, ClaudeRegistrySource, HookAgentSource, HookSessionSource,
    },
    repositories::{EmbeddedCatalog, JsonAssignmentStore},
    services::{ClaudeSettingsHookInstaller, PsProcessInspector, SystemClock, SystemCommandRunner},
};
use cerebro_domain::{
    DomainError,
    ports::{CatalogSource, Clock},
    usecases::{ManageHooks, ObserveSessions, ObserveSessionsDeps},
};
use cerebro_protocol::Paths;

pub struct Container {
    pub observe: ObserveSessions,
    pub hooks: ManageHooks,
}

pub fn build() -> Result<Container, DomainError> {
    let paths = Paths::detect()
        .ok_or_else(|| DomainError::Platform("cannot find the home directory".into()))?;
    let runner = Arc::new(SystemCommandRunner);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    // One source per AI tool: supporting another tool means adding its source here.
    let claude_code = ClaudeCodeSource::new(ClaudeCodeDeps {
        registry: Arc::new(ClaudeRegistrySource::new(paths.claude_sessions_dir())),
        hooks: Arc::new(HookSessionSource::new(paths.hook_sessions_dir())),
        agents: Arc::new(HookAgentSource::new(paths.clone())),
        processes: Arc::new(PsProcessInspector::new(runner.clone())),
        clock: clock.clone(),
    });

    let observe = ObserveSessions::new(ObserveSessionsDeps {
        sources: vec![Arc::new(claude_code)],
        store: Arc::new(JsonAssignmentStore::new(paths.assignments_file())),
        clock,
        catalog: EmbeddedCatalog.catalog()?,
    });

    Ok(Container {
        observe,
        hooks: ManageHooks::new(Arc::new(ClaudeSettingsHookInstaller::new(
            paths.claude_settings_file(),
            hook_binary_path(),
        ))),
    })
}

/// Same rule as the app: the hook ships next to the executable.
fn hook_binary_path() -> PathBuf {
    let name = if cfg!(windows) {
        "cerebro-hook.exe"
    } else {
        "cerebro-hook"
    };
    std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}
