//! Cerebro data layer: implements the domain ports with real I/O.
//!
//! ```text
//! models        serde DTOs mirroring files on disk, converted into domain entities
//! datasources   one session source per AI tool (Claude Code) and the readers behind it
//! repositories  assignment persistence and the embedded asset catalog
//! services      process inspection, command execution, clock, hook installer, ZeroTier
//! strategies    per-platform terminal focus
//! ```

pub mod datasources;
pub mod models;
pub mod repositories;
pub mod services;
pub mod strategies;
