//! Persistence and static data.

mod embedded_catalog;
mod json_assignment_store;
mod json_network_choice_store;

pub use embedded_catalog::EmbeddedCatalog;
pub use json_assignment_store::JsonAssignmentStore;
pub use json_network_choice_store::JsonNetworkChoiceStore;
