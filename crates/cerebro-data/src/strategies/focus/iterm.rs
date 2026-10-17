use cerebro_domain::DomainError;

use super::TabSelector;
use crate::services::CommandRunner;

/// Selects the iTerm2 window, tab and split pane whose session owns a tty.
pub struct ITermTabSelector;

impl TabSelector for ITermTabSelector {
    fn handles(&self, bundle_path: &str) -> bool {
        bundle_path.ends_with("/iTerm.app")
    }

    fn select(&self, runner: &dyn CommandRunner, tty: &str) -> Result<bool, DomainError> {
        // `tty` comes from `tty_of`, which only allows `/dev/` + alphanumerics: safe to embed.
        let script = format!(
            r#"tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if tty of s is "{tty}" then
                    select w
                    select t
                    select s
                    activate
                    return "found"
                end if
            end repeat
        end repeat
    end repeat
end tell
return "missing""#
        );
        let output = runner
            .run("osascript", &["-e", &script])
            .map_err(|error| DomainError::Platform(error.to_string()))?;
        if !output.success {
            return Err(DomainError::Platform(format!(
                "iTerm2 AppleScript failed: {}",
                output.stderr.trim()
            )));
        }
        Ok(output.stdout.trim() == "found")
    }
}
