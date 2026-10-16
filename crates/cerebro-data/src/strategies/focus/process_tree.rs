use crate::services::CommandRunner;

/// The GUI application hosting a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostApp {
    /// e.g. `/Applications/iTerm.app`
    pub bundle_path: String,
    /// e.g. `iTerm`
    pub name: String,
}

/// Safety net against cycles or very deep trees.
const MAX_DEPTH: usize = 32;

/// Walks up the parent chain until a process lives inside a `.app` bundle.
pub fn find_host_app(runner: &dyn CommandRunner, pid: u32) -> Option<HostApp> {
    let mut current = pid;
    for _ in 0..MAX_DEPTH {
        let output = runner.run("ps", &["-o", "ppid=,comm=", "-p", &current.to_string()]).ok()?;
        let line = output.stdout.trim();
        let (parent, command) = line.split_once(char::is_whitespace)?;
        if let Some(app) = bundle_of(command.trim()) {
            return Some(app);
        }
        current = parent.trim().parse().ok()?;
        if current <= 1 {
            return None;
        }
    }
    None
}

/// `/Applications/Visual Studio Code.app/Contents/Frameworks/Code Helper.app/…`
/// → the outermost bundle, `/Applications/Visual Studio Code.app`.
fn bundle_of(command: &str) -> Option<HostApp> {
    let end = command.find(".app/")? + ".app".len();
    let bundle_path = &command[..end];
    let name = bundle_path.rsplit('/').next()?.trim_end_matches(".app");
    Some(HostApp { bundle_path: bundle_path.to_owned(), name: name.to_owned() })
}

/// Controlling terminal of a process, e.g. `/dev/ttys003`.
pub fn tty_of(runner: &dyn CommandRunner, pid: u32) -> Option<String> {
    let output = runner.run("ps", &["-o", "tty=", "-p", &pid.to_string()]).ok()?;
    let tty = output.stdout.trim();
    let valid = !tty.is_empty() && tty != "??" && tty.chars().all(|c| c.is_ascii_alphanumeric());
    valid.then(|| format!("/dev/{tty}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outermost_bundle_wins() {
        let app = bundle_of("/Applications/Visual Studio Code.app/Contents/Frameworks/Code Helper (Plugin).app/Contents/MacOS/x").unwrap();
        assert_eq!(app.bundle_path, "/Applications/Visual Studio Code.app");
        assert_eq!(app.name, "Visual Studio Code");
        assert_eq!(bundle_of("/bin/zsh"), None);
    }
}
