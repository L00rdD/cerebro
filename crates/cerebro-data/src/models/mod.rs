//! On-disk formats owned by other programs (Claude Code) or by the app itself.

mod assignments_file;
mod catalog_files;
mod registry_entry;

pub use assignments_file::AssignmentsFile;
pub use catalog_files::{CharactersFile, RoomsFile};
pub use registry_entry::RegistryEntry;
pub(crate) use registry_entry::folder_name_of;
