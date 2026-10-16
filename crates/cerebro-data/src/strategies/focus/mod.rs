//! Bringing a session's terminal to the front.
//!
//! macOS: walk up the process tree from the Claude pid to the hosting `.app`, then let a
//! [`TabSelector`] pick the exact tab by tty; otherwise just activate the app.
//! Other platforms: not implemented yet (EXERCISE 10).

mod iterm;
mod process_tree;
mod terminal_app;

use std::sync::Arc;

use cerebro_domain::DomainError;
use cerebro_domain::ports::{FocusOutcome, FocusTarget, TerminalFocuser};

pub use iterm::ITermTabSelector;
pub use process_tree::{HostApp, find_host_app, tty_of};
pub use terminal_app::TerminalAppTabSelector;

use crate::services::CommandRunner;

/// Selects the tab owning a tty inside one specific terminal application.
pub trait TabSelector: Send + Sync {
    /// True when this selector knows the app at `bundle_path` (e.g. `/Applications/iTerm.app`).
    fn handles(&self, bundle_path: &str) -> bool;
    /// Selects the tab for `tty` (e.g. `/dev/ttys003`). Returns `false` if no tab matched.
    fn select(&self, runner: &dyn CommandRunner, tty: &str) -> Result<bool, DomainError>;
}

pub struct MacFocuser {
    runner: Arc<dyn CommandRunner>,
    selectors: Vec<Box<dyn TabSelector>>,
}

impl MacFocuser {
    /// EXERCISE 6 adds `TerminalAppTabSelector` to this list.
    pub fn new(runner: Arc<dyn CommandRunner>) -> Self {
        Self {
            runner,
            selectors: vec![Box::new(ITermTabSelector)],
        }
    }

    pub fn with_selectors(
        runner: Arc<dyn CommandRunner>,
        selectors: Vec<Box<dyn TabSelector>>,
    ) -> Self {
        Self { runner, selectors }
    }

    fn activate(&self, host: &HostApp) -> Result<(), DomainError> {
        let output = self
            .runner
            .run("open", &["-a", &host.bundle_path])
            .map_err(|e| DomainError::Platform(e.to_string()))?;
        if output.success {
            Ok(())
        } else {
            Err(DomainError::Platform(output.stderr))
        }
    }
}

impl TerminalFocuser for MacFocuser {
    fn focus(&self, target: &FocusTarget) -> Result<FocusOutcome, DomainError> {
        let Some(host) = find_host_app(self.runner.as_ref(), target.pid) else {
            return Ok(FocusOutcome::NotFound);
        };
        if let (Some(selector), Some(tty)) = (
            self.selectors.iter().find(|s| s.handles(&host.bundle_path)),
            tty_of(self.runner.as_ref(), target.pid),
        ) && selector.select(self.runner.as_ref(), &tty)?
        {
            return Ok(FocusOutcome::Tab { app: host.name });
        }
        self.activate(&host)?;
        Ok(FocusOutcome::App { app: host.name })
    }
}

/// Placeholder for platforms without an implementation yet.
pub struct UnsupportedFocuser;

impl TerminalFocuser for UnsupportedFocuser {
    fn focus(&self, _target: &FocusTarget) -> Result<FocusOutcome, DomainError> {
        Ok(FocusOutcome::NotFound)
    }
}

/// The best focuser for the current OS.
pub fn platform_focuser(runner: Arc<dyn CommandRunner>) -> Arc<dyn TerminalFocuser> {
    if cfg!(target_os = "macos") {
        Arc::new(MacFocuser::new(runner))
    } else {
        Arc::new(UnsupportedFocuser)
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use std::collections::HashMap;
    use std::io;
    use std::sync::Mutex;

    use crate::services::{CommandOutput, CommandRunner};

    /// Answers commands from a table keyed by `"program arg1 arg2…"` and records every call.
    #[derive(Default)]
    pub struct ScriptedRunner {
        pub answers: HashMap<String, String>,
        pub calls: Mutex<Vec<String>>,
    }

    impl ScriptedRunner {
        pub fn answer(mut self, command: &str, stdout: &str) -> Self {
            self.answers.insert(command.into(), stdout.into());
            self
        }
    }

    impl CommandRunner for ScriptedRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            let key = std::iter::once(program)
                .chain(args.iter().copied())
                .collect::<Vec<_>>()
                .join(" ");
            self.calls.lock().unwrap().push(key.clone());
            let stdout = self
                .answers
                .iter()
                .find(|(k, _)| key.starts_with(k.as_str()))
                .map(|(_, v)| v.clone());
            Ok(CommandOutput {
                success: true,
                stdout: stdout.unwrap_or_default(),
                stderr: String::new(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::ScriptedRunner;
    use super::*;

    fn tree() -> ScriptedRunner {
        ScriptedRunner::default()
            .answer(
                "ps -o ppid=,comm= -p 300",
                "  200 /Users/me/.local/bin/claude\n",
            )
            .answer("ps -o ppid=,comm= -p 200", "  100 -zsh\n")
            .answer(
                "ps -o ppid=,comm= -p 100",
                "    1 /Applications/iTerm.app/Contents/MacOS/iTerm2\n",
            )
            .answer("ps -o tty= -p 300", "ttys003\n")
    }

    fn target() -> FocusTarget {
        FocusTarget {
            pid: 300,
            cwd: "/w".into(),
        }
    }

    #[test]
    fn selects_the_iterm_tab_by_tty() {
        let runner = Arc::new(tree().answer("osascript", "found\n"));
        let outcome = MacFocuser::new(runner.clone()).focus(&target()).unwrap();
        assert_eq!(
            outcome,
            FocusOutcome::Tab {
                app: "iTerm".into()
            }
        );
        let calls = runner.calls.lock().unwrap();
        assert!(calls.iter().any(|c| c.contains("/dev/ttys003")));
    }

    #[test]
    fn activates_the_app_when_no_tab_matches() {
        let runner = Arc::new(tree().answer("osascript", "missing\n"));
        let outcome = MacFocuser::new(runner.clone()).focus(&target()).unwrap();
        assert_eq!(
            outcome,
            FocusOutcome::App {
                app: "iTerm".into()
            }
        );
        assert!(
            runner
                .calls
                .lock()
                .unwrap()
                .iter()
                .any(|c| c == "open -a /Applications/iTerm.app")
        );
    }

    #[test]
    fn unknown_host_is_not_found() {
        let runner = Arc::new(
            ScriptedRunner::default().answer("ps -o ppid=,comm= -p 300", "    1 /usr/bin/tmux\n"),
        );
        assert_eq!(
            MacFocuser::new(runner).focus(&target()).unwrap(),
            FocusOutcome::NotFound
        );
    }
}
