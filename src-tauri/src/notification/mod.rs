// src-tauri/src/notification/mod.rs

pub mod emitter;
pub mod types;

pub use emitter::NotificationEmitter;
pub use types::{NotificationDisplay, NotificationPayload, NotificationType};
