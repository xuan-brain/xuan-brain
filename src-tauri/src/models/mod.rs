//! Domain models for the application
//!
//! These models are used for business logic and API responses.
//! They are separate from database entities to allow flexibility.

pub mod attachment;
pub mod author;
pub mod category;
pub mod comment;
pub mod keyword;
pub mod label;
pub mod paper;
pub mod clipping;  // clipping must come after comment

// Explicit exports to avoid ambiguity between modules
pub use attachment::Attachment;
pub use author::{Author, CreateAuthor};
pub use category::{Category, CategoryNode, CreateCategory, UpdateCategory};
pub use comment::Comment;
pub use keyword::{CreateKeyword, Keyword};
pub use label::{CreateLabel, Label, UpdateLabel};
#[allow(unused_imports)]
pub use paper::{AuthorWithOrder, CreatePaper, Paper, UpdatePaper};
pub use clipping::{Clipping, CreateClipping, UpdateClipping};
