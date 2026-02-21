//! Paper command module
//!
//! This module contains all paper-related Tauri commands, organized into submodules:
//! - `dtos`: Data Transfer Objects for API responses
//! - `utils`: Helper functions
//! - `query`: Read operations (get papers, search)
//! - `mutation`: Write operations (create, update, delete)
//! - `import`: Import operations (DOI, arXiv, PMID, PDF)
//! - `attachment`: Attachment operations

mod dtos;
mod utils;
mod query;
mod mutation;
mod import;
mod attachment;

// Re-export all commands
pub use query::*;
pub use mutation::*;
pub use import::*;
pub use attachment::*;
