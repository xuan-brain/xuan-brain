//! Clip command module
//!
//! This module contains all clip-related Tauri commands:
//! - `dtos`: Data Transfer Objects
//! - `utils`: Helper functions for image processing
//! - `query`: Read operations (list_clips, get_clip)
//! - `mutation`: Write operations (create_clip, add_clip_comment, update_clip_comment, delete_clip_comment)

mod dtos;
mod mutation;
mod query;
mod utils;

// Re-export all commands
pub use mutation::{add_clip_comment, create_clip, delete_clip_comment, update_clip_comment};
pub use query::{get_clip, list_clips};
