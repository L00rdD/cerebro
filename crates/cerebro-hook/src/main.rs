//! Entry point called by Claude Code for every hooked event.
//!
//! Contract with Claude Code: read the event JSON on stdin, print nothing
//! (stdout of some hooks is injected into the conversation) and always exit 0
//! so a Cerebro problem can never block a Claude session.

use std::io::Read;

fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }
    let Some(paths) = cerebro_protocol::Paths::detect() else {
        return;
    };
    // Errors are deliberately swallowed: see the contract above.
    let _ = cerebro_hook::handle(&input, &paths, cerebro_protocol::now_ms());
}
