// src-tauri/src/notification/emitter.rs

use super::types::{NotificationDisplay, NotificationPayload, NotificationType};
use tauri::{AppHandle, Emitter};

pub struct NotificationEmitter<'a> {
    app_handle: &'a AppHandle,
    notification_type: NotificationType,
    title: String,
    message: String,
    display: Option<NotificationDisplay>,
    persistent: Option<bool>,
    duration: Option<u64>,
    details: Option<String>,
}

impl<'a> NotificationEmitter<'a> {
    pub fn new(app_handle: &'a AppHandle, notification_type: NotificationType) -> Self {
        Self {
            app_handle,
            notification_type,
            title: String::new(),
            message: String::new(),
            display: None,
            persistent: None,
            duration: None,
            details: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn display(mut self, display: NotificationDisplay) -> Self {
        self.display = Some(display);
        self
    }

    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = Some(persistent);
        self
    }

    pub fn duration(mut self, duration: u64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn send(self) -> Result<(), Box<dyn std::error::Error>> {
        let event_name = format!("notification:{}", self.notification_type_as_str());
        let payload = NotificationPayload {
            notification_type: self.notification_type,
            title: self.title,
            message: self.message,
            display: self.display,
            persistent: self.persistent,
            duration: self.duration,
            details: self.details,
        };

        self.app_handle.emit(&event_name, payload)?;
        Ok(())
    }

    fn notification_type_as_str(&self) -> &'static str {
        match self.notification_type {
            NotificationType::Success => "success",
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
        }
    }
}

// Convenience constructors
impl<'a> NotificationEmitter<'a> {
    pub fn success(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Success)
            .title("Success")
    }

    pub fn info(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Info)
            .title("Info")
    }

    pub fn warning(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Warning)
            .title("Warning")
    }

    pub fn error(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Error)
            .title("Error")
    }
}
