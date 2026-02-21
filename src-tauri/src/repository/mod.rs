//! Repository layer for SurrealDB
//!
//! Provides data access abstraction for all entities.

pub mod paper_repository;
pub mod category_repository;
pub mod label_repository;
pub mod author_repository;
pub mod keyword_repository;
pub mod attachment_repository;

pub use paper_repository::PaperRepository;
pub use category_repository::{CategoryRepository, TreeNodeData};
pub use label_repository::LabelRepository;
pub use author_repository::AuthorRepository;
pub use attachment_repository::AttachmentRepository;
