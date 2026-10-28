//! On-disk and command-line formats owned by other programs (Claude Code, `ZeroTier`) or by the
//! app itself.

mod assignments_file;
mod catalog_files;
mod phone_networks_file;
mod registry_entry;
mod zerotier;

pub use assignments_file::AssignmentsFile;
pub use catalog_files::{CharactersFile, RoomsFile};
pub use phone_networks_file::PhoneNetworksFile;
pub use registry_entry::RegistryEntry;
pub(crate) use registry_entry::folder_name_of;
pub use zerotier::{ZeroTierInfo, ZeroTierNetworkEntry};
