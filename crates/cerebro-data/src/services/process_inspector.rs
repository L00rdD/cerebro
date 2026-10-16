use std::collections::HashMap;
use std::sync::Arc;

use cerebro_domain::ports::{ProcessInfo, ProcessInspector};

use super::CommandRunner;

/// Asks `ps` (macOS, Linux) for the start time of all pids in one call.
pub struct PsProcessInspector {
    runner: Arc<dyn CommandRunner>,
}

impl PsProcessInspector {
    pub fn new(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl ProcessInspector for PsProcessInspector {
    fn inspect(&self, pids: &[u32]) -> HashMap<u32, ProcessInfo> {
        if pids.is_empty() {
            return HashMap::new();
        }
        if cfg!(windows) {
            // EXERCISE 10: query Windows for real start times. Until then trust the registry.
            return pids.iter().map(|&pid| (pid, ProcessInfo { start: None })).collect();
        }

        let list = pids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        // Claude Code records `procStart` in UTC; `ps` prints local time unless told otherwise.
        match self.runner.run("env", &["TZ=UTC", "ps", "-o", "pid=,lstart=", "-p", &list]) {
            // `ps` exits with 1 when some pids are gone: the output is still valid.
            Ok(output) => output
                .stdout
                .lines()
                .filter_map(parse_ps_line)
                .map(|(pid, start)| (pid, ProcessInfo { start: Some(start) }))
                .collect(),
            // Can't tell: better show possibly stale sessions than hide live ones.
            Err(_) => pids.iter().map(|&pid| (pid, ProcessInfo { start: None })).collect(),
        }
    }
}

/// Parses `" 52117 Thu Oct 15 08:43:57 2026"` into `(52117, "Thu Oct 15 08:43:57 2026")`.
///
/// EXERCISE 7 asks for a zero-allocation version returning a `&str`.
pub fn parse_ps_line(line: &str) -> Option<(u32, String)> {
    let line = line.trim();
    let (pid, rest) = line.split_once(char::is_whitespace)?;
    let start = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    Some((pid.parse().ok()?, start)).filter(|(_, s)| !s.is_empty())
}

/// A `ps` line that borrows from the original text instead of copying it (EXERCISE 7).
#[derive(Debug, PartialEq, Eq)]
pub struct PsLine<'a> {
    pub pid: u32,
    /// Raw start time, possibly with irregular spaces (`"Thu Oct 15  8:43:57 2026"`).
    pub start: &'a str,
}

impl PsLine<'_> {
    /// EXERCISE 7: compare with a recorded start time, ignoring whitespace, without allocating.
    pub fn started_at(&self, recorded: &str) -> bool {
        todo!("Exercise 7: compare {:?} with {recorded:?}", self.start)
    }
}

/// EXERCISE 7: zero-allocation version of [`parse_ps_line`].
pub fn parse_ps_line_borrowed(line: &str) -> Option<PsLine<'_>> {
    todo!("Exercise 7: parse {line:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::CommandOutput;
    use std::io;

    struct FakeRunner(&'static str);
    impl CommandRunner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            assert_eq!(program, "env");
            assert_eq!(args[..2], ["TZ=UTC", "ps"]);
            assert_eq!(args.last(), Some(&"1,2"));
            Ok(CommandOutput { success: false, stdout: self.0.into(), stderr: String::new() })
        }
    }

    #[test]
    fn parses_lines() {
        assert_eq!(parse_ps_line("  52117 Thu Oct 15  8:43:57 2026"), Some((52117, "Thu Oct 15 8:43:57 2026".into())));
        assert_eq!(parse_ps_line("garbage"), None);
        assert_eq!(parse_ps_line(""), None);
    }

    #[test]
    #[ignore = "exercise 7"]
    fn borrowed_parse_points_into_the_input() {
        let line = String::from("  52117 Thu Oct 15  8:43:57 2026");
        let parsed = parse_ps_line_borrowed(&line).unwrap();
        assert_eq!(parsed.pid, 52117);
        // Same memory as `line`: no copy was made.
        let inside = line.as_ptr() as usize..line.as_ptr() as usize + line.len();
        assert!(inside.contains(&(parsed.start.as_ptr() as usize)));
        assert!(parsed.started_at("Thu Oct 15 8:43:57 2026"));
        assert!(!parsed.started_at("Thu Oct 15 8:43:58 2026"));
        assert_eq!(parse_ps_line_borrowed("garbage"), None);
    }

    #[test]
    #[cfg(not(windows))]
    fn only_returns_alive_pids() {
        let inspector = PsProcessInspector::new(Arc::new(FakeRunner("    1 Thu Oct 15 08:00:00 2026\n")));
        let alive = inspector.inspect(&[1, 2]);
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[&1].start.as_deref(), Some("Thu Oct 15 08:00:00 2026"));
    }
}
