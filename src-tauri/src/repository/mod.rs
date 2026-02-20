//! Repository layer for SurrealDB
//!
//! Provides data access abstraction for all entities.

pub mod paper_repository;
pub mod category_repository;
pub mod label_repository;
pub mod author_repository;
pub mod keyword_repository;

pub use paper_repository::PaperRepository;
