use thiserror::Error;

/// Main error type for the xuan-brain application
#[derive(Error, Debug)]
pub enum AppError {
    /// Document parsing errors
    #[error("Document parsing failed: {message}")]
    DocumentParseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// File system errors
    #[error("File system error: {path} - {message}")]
    FileSystemError {
        path: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database errors
    #[error("Database error: {message}")]
    #[from(sea_orm::DbErr)]
    DatabaseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// AI engine errors
    #[error("AI engine error: {operation} - {message}")]
    AIError {
        operation: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Sync errors
    #[error("Sync error: {service} - {message}")]
    SyncError {
        service: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Plugin errors
    #[error("Plugin error: {plugin_name} - {message}")]
    PluginError {
        plugin_name: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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
    OCRError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// PDF specific errors
    #[error("PDF error: {operation} - {message}")]
    PDFError { operation: String, message: String },

    /// IO error wrapper
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

impl AppError {
    /// Create a document parse error
    pub fn document_parse(message: impl Into<String>) -> Self {
        AppError::DocumentParseError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a document parse error with source
    pub fn document_parse_with_source(
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        AppError::DocumentParseError {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a file system error
    pub fn file_system(path: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::FileSystemError {
            path: path.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a file system error with source
    pub fn file_system_with_source(
        path: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        AppError::FileSystemError {
            path: path.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        AppError::DatabaseError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a database error with source
    pub fn database_with_source(
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        AppError::DatabaseError {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create an AI engine error
    pub fn ai_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::AIError {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a sync error
    pub fn sync_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::SyncError {
            service: service.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a plugin error
    pub fn plugin_error(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::PluginError {
            plugin_name: plugin_name.into(),
            message: message.into(),
            source: None,
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
            source: None,
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

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;
