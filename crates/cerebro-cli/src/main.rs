//! `cerebro`: command line companion (EXERCISE 3).
//!
//! Target usage:
//! ```text
//! cerebro list     one line per live session (uses `Display for Session`, exercise 2)
//! cerebro stats    agents by type, busiest session… (uses `compute_stats`, exercise 5)
//! cerebro hooks    hook installation status
//! ```
//!
//! Reuse the same adapters as the app: look at `src-tauri/src/injector.rs`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    if let Some("list" | "stats" | "hooks") = command.as_deref() {
        eprintln!("not implemented yet: see EXERCISES.md, exercise 3");
        ExitCode::FAILURE
    } else {
        eprintln!("usage: cerebro <list|stats|hooks>");
        ExitCode::from(2)
    }
}
