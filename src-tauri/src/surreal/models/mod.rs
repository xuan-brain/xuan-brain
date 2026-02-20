//! SurrealDB data models
//!
//! Rust structs representing SurrealDB records with serde serialization.

pub mod paper;
pub mod author;
pub mod keyword;
pub mod label;
pub mod category;
pub mod attachment;
pub mod relations;

pub use paper::*;
pub use author::*;
pub use keyword::*;
pub use label::*;
pub use category::*;
