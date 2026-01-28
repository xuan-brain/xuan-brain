use serde::{Serialize, Serializer};
use thiserror::Error;

/// Main error type for xuan-brain application
#[derive(Error, Debug)]
pub enum AppError {
    /// Document parsing errors
    #[error("Document parsing failed: {message}")]
    DocumentParseError { message: String },

    /// File system errors
    #[error("File system error: {path} - {message}")]
    FileSystemError { path: String, message: String },

    /// Database errors
    #[error(transparent)]
    SeaOrmError(#[from] sea_orm::DbErr),

    /// AI engine errors
    #[error("AI engine error: {operation} - {message}")]
    AIError { operation: String, message: String },

    /// Sync errors
    #[error("Sync error: {service} - {message}")]
    SyncError { service: String, message: String },

    /// Plugin errors
    #[error("Plugin error: {plugin_name} - {message}")]
    PluginError {
        plugin_name: String,
        message: String,
    },

    /// Configuration errors
    #[error("Configuration error: {key} - {message}")]
    ConfigError { key: String, message: String },

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    /// Network errors
    #[error("Network error: {url} - {message}")]
    NetworkError { url: String, message: String },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Permission errors
    #[error("Permission denied: {resource}")]
    PermissionError { resource: String },

    /// Resource not found
    #[error("Resource not found: {resource_type} '{resource_id}'")]
    NotFound {
        resource_type: String,
        resource_id: String,
    },

    /// Invalid input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// OCR errors
    #[error("OCR failed: {message}")]
    OCRError { message: String },

    /// PDF specific errors
    #[error("PDF error: {operation} - {message}")]
    PDFError { operation: String, message: String },

    /// IO error wrapper
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// Custom serialization for AppError to handle non-serializable types
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ErrorResponse<'a> {
            error_type: &'a str,
            message: Option<&'a String>,
            path: Option<&'a String>,
            operation: Option<&'a String>,
            service: Option<&'a String>,
            plugin_name: Option<&'a String>,
            key: Option<&'a String>,
            url: Option<&'a String>,
            field: Option<&'a String>,
            resource: Option<&'a String>,
            resource_type: Option<&'a String>,
            resource_id: Option<&'a String>,
        }

        let response = match self {
            AppError::DocumentParseError { message } => ErrorResponse {
                error_type: "DocumentParseError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::FileSystemError { path, message } => ErrorResponse {
                error_type: "FileSystemError",
                message: Some(message),
                path: Some(path),
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::SeaOrmError(err) => ErrorResponse {
                error_type: "SeaOrmError",
                message: Some(&err.to_string()),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::AIError { operation, message } => ErrorResponse {
                error_type: "AIError",
                message: Some(message),
                path: None,
                operation: Some(operation),
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::SyncError { service, message } => ErrorResponse {
                error_type: "SyncError",
                message: Some(message),
                path: None,
                operation: None,
                service: Some(service),
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::PluginError {
                plugin_name,
                message,
            } => ErrorResponse {
                error_type: "PluginError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: Some(plugin_name),
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::ConfigError { key, message } => ErrorResponse {
                error_type: "ConfigError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: Some(key),
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::AuthenticationError { message } => ErrorResponse {
                error_type: "AuthenticationError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::NetworkError { url, message } => ErrorResponse {
                error_type: "NetworkError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: Some(url),
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::ValidationError { field, message } => ErrorResponse {
                error_type: "ValidationError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: Some(field),
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::PermissionError { resource } => ErrorResponse {
                error_type: "PermissionError",
                message: None,
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: Some(resource),
                resource_type: None,
                resource_id: None,
            },
            AppError::NotFound {
                resource_type,
                resource_id,
            } => ErrorResponse {
                error_type: "NotFound",
                message: None,
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: Some(resource_type),
                resource_id: Some(resource_id),
            },
            AppError::InvalidInput { message } => ErrorResponse {
                error_type: "InvalidInput",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::OCRError { message } => ErrorResponse {
                error_type: "OCRError",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::PDFError { operation, message } => ErrorResponse {
                error_type: "PDFError",
                message: Some(message),
                path: None,
                operation: Some(operation),
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::IoError(err) => ErrorResponse {
                error_type: "IoError",
                message: Some(&err.to_string()),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
            AppError::Generic(message) => ErrorResponse {
                error_type: "Generic",
                message: Some(message),
                path: None,
                operation: None,
                service: None,
                plugin_name: None,
                key: None,
                url: None,
                field: None,
                resource: None,
                resource_type: None,
                resource_id: None,
            },
        };

        response.serialize(serializer)
    }
}

impl AppError {
    /// Create a document parse error
    pub fn document_parse(message: impl Into<String>) -> Self {
        AppError::DocumentParseError {
            message: message.into(),
        }
    }

    /// Create a file system error
    pub fn file_system(path: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::FileSystemError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create an AI engine error
    pub fn ai_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::AIError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a sync error
    pub fn sync_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::SyncError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a plugin error
    pub fn plugin_error(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::PluginError {
            plugin_name: plugin_name.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config_error(key: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::ConfigError {
            key: key.into(),
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        AppError::AuthenticationError {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error(url: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::NetworkError {
            url: url.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a permission error
    pub fn permission(resource: impl Into<String>) -> Self {
        AppError::PermissionError {
            resource: resource.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        AppError::NotFound {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        AppError::InvalidInput {
            message: message.into(),
        }
    }

    /// Create an OCR error
    pub fn ocr_error(message: impl Into<String>) -> Self {
        AppError::OCRError {
            message: message.into(),
        }
    }

    /// Create a PDF error
    pub fn pdf_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::PDFError {
            operation: operation.into(),
            message: message.into(),
        }
    }
}

// Implement IpcResponse for AppError to make it compatible with Tauri 2.x IPC

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;
