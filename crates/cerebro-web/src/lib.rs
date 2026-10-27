//! Cerebro on a phone: serves the UI and a live feed of snapshots over HTTP.
//!
//! Presentation layer, like the desktop app, but free of Tauri so any host can run it
//! (the tray app today, `cerebro serve` in exercise 13).
//!
//! ```text
//! dto      view models shared by every UI (desktop panel and phone)
//! assets   where the built UI files come from (embedded, or a folder)
//! key      the secret in the phone link
//! server   the HTTP server: page, snapshot API, server-sent events
//! ```

pub mod dto;

mod assets;
mod key;
mod server;

pub use assets::{Asset, Assets, DirAssets, mime_type_of};
pub use key::AccessKey;
pub use server::{DEFAULT_PORT, Publisher, WebServer};
