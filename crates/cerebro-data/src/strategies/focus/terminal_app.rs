use cerebro_domain::DomainError;

use super::TabSelector;
use crate::services::CommandRunner;

/// EXERCISE 6: select the macOS Terminal.app tab whose tty matches.
///
/// `AppleScript` hint: Terminal tabs expose a `tty` property; select a tab with
/// `set selected of t to true` and bring its window forward with `set index of w to 1`.
pub struct TerminalAppTabSelector;

impl TabSelector for TerminalAppTabSelector {
    fn handles(&self, bundle_path: &str) -> bool {
        todo!("Exercise 6: recognise Terminal.app in {bundle_path}")
    }

    fn select(&self, runner: &dyn CommandRunner, tty: &str) -> Result<bool, DomainError> {
        let _ = runner;
        todo!("Exercise 6: select the tab owning {tty}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::focus::fake::ScriptedRunner;

    #[test]
    #[ignore = "exercise 6"]
    fn handles_only_terminal_app() {
        assert!(TerminalAppTabSelector.handles("/System/Applications/Utilities/Terminal.app"));
        assert!(!TerminalAppTabSelector.handles("/Applications/iTerm.app"));
    }

    #[test]
    #[ignore = "exercise 6"]
    fn runs_an_applescript_mentioning_the_tty() {
        let runner = ScriptedRunner::default().answer("osascript", "found\n");
        assert!(
            TerminalAppTabSelector
                .select(&runner, "/dev/ttys007")
                .unwrap()
        );
        let calls = runner.calls.lock().unwrap();
        assert!(calls[0].contains("Terminal") && calls[0].contains("/dev/ttys007"));
    }

    #[test]
    #[ignore = "exercise 6"]
    fn reports_a_missing_tab() {
        let runner = ScriptedRunner::default().answer("osascript", "missing\n");
        assert!(
            !TerminalAppTabSelector
                .select(&runner, "/dev/ttys007")
                .unwrap()
        );
    }
}
