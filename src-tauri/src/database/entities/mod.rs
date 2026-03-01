//! SeaORM entity definitions
//!
//! Each entity corresponds to a database table.

pub mod attachment;
pub mod author;
pub mod category;
pub mod clip_label;
pub mod clipping;
pub mod comment;
pub mod keyword;
pub mod label;
pub mod paper;
pub mod paper_author;
pub mod paper_category;
pub mod paper_keyword;
pub mod paper_label;

#[allow(unused_imports)]
pub use attachment::Entity as Attachment;
#[allow(unused_imports)]
pub use author::Entity as Author;
#[allow(unused_imports)]
pub use category::Entity as Category;
#[allow(unused_imports)]
pub use clip_label::Entity as ClipLabel;
#[allow(unused_imports)]
pub use clipping::Entity as Clipping;
#[allow(unused_imports)]
pub use comment::Entity as Comment;
#[allow(unused_imports)]
pub use keyword::Entity as Keyword;
#[allow(unused_imports)]
pub use label::Entity as Label;
#[allow(unused_imports)]
pub use paper::Entity as Paper;
#[allow(unused_imports)]
pub use paper_author::Entity as PaperAuthor;
#[allow(unused_imports)]
pub use paper_category::Entity as PaperCategory;
#[allow(unused_imports)]
pub use paper_keyword::Entity as PaperKeyword;
#[allow(unused_imports)]
pub use paper_label::Entity as PaperLabel;
