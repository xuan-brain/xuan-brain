//! Clip command module
//!
//! This module contains all clip-related Tauri commands:
//! - `dtos`: Data Transfer Objects
//! - `utils`: Helper functions for image processing
//! - `query`: Read operations (list_clips, get_clip)
//! - `mutation`: Write operations (create_clip)

mod dtos;
mod mutation;
mod query;
mod utils;

// Re-export all commands
pub use mutation::create_clip;
pub use query::{get_clip, list_clips};
