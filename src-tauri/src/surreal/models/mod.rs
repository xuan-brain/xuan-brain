//! SurrealDB data models
//!
//! Rust structs representing SurrealDB records with serde serialization.

pub mod author;
pub mod category;
pub mod keyword;
pub mod label;
pub mod paper;
pub mod relations;

pub use author::*;
pub use category::*;
pub use keyword::*;
pub use label::*;
pub use paper::*;
