use std::sync::Arc;

use crate::DomainError;
use crate::ports::{HookInstaller, HookSetup};

/// Reports and repairs Cerebro's hook entries in Claude Code's settings.
pub struct ManageHooks {
    installer: Arc<dyn HookInstaller>,
}

impl ManageHooks {
    pub fn new(installer: Arc<dyn HookInstaller>) -> Self {
        Self { installer }
    }

    pub fn status(&self) -> Result<HookSetup, DomainError> {
        self.installer.status()
    }

    /// Installs or repairs the hooks. Does nothing when they are already up to date.
    pub fn install(&self) -> Result<HookSetup, DomainError> {
        if self.installer.status()? != HookSetup::Installed {
            self.installer.install()?;
        }
        self.installer.status()
    }
}
