//! Cerebro domain layer.
//!
//! Pure business logic: it knows nothing about files, processes, Tauri or the UI.
//! Outer layers plug in through the traits in [`ports`] (dependency inversion).
//!
//! ```text
//! entities   immutable business types (Session, Cast, Snapshot…)
//! ports      traits implemented by the data layer (SessionSource, TerminalFocuser…)
//! strategies reusable policies (stable casting of rooms and characters)
//! usecases   one struct per application action (ObserveSessions, FocusSession…)
//! ```

pub mod entities;
pub mod error;
pub mod ports;
pub mod rng;
pub mod strategies;
pub mod time;
pub mod usecases;

pub use error::DomainError;
