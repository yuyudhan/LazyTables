// FilePath: src/core/error.rs

#![forbid(unsafe_code)]

use thiserror::Error;

/// Result type alias for LazyTables
pub type Result<T> = std::result::Result<T, LazyTablesError>;

/// Main error type for LazyTables
#[derive(Error, Debug)]
pub enum LazyTablesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Event handling error: {0}")]
    Event(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(ConnectionError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not connected to database")]
    NotConnected,

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Connection '{0}' already exists")]
    ConnectionExists(String),

    #[error("Connection '{0}' not found")]
    ConnectionNotFound(String),

    #[error("Password error: {0}")]
    PasswordError(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    #[error("{0}")]
    Other(String),
}

impl From<toml::ser::Error> for LazyTablesError {
    fn from(err: toml::ser::Error) -> Self {
        LazyTablesError::Config(err.to_string())
    }
}

/// Legacy type alias for backwards compatibility
pub type Error = LazyTablesError;

/// Connection error category for better error classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionErrorType {
    /// Network-related issues (cannot reach host, port closed, timeout)
    Network,
    /// Authentication failures (wrong username/password, permission denied)
    Authentication,
    /// Database not found or doesn't exist
    DatabaseNotFound,
    /// SSL/TLS configuration issues
    SslConfiguration,
    /// Invalid configuration (bad host format, invalid port, etc.)
    Configuration,
    /// Database server error (out of memory, too many connections, etc.)
    ServerError,
    /// Unknown or unclassified error
    Unknown,
}

impl ConnectionErrorType {
    /// Get a user-friendly category name
    pub fn category_name(&self) -> &'static str {
        match self {
            Self::Network => "Network Error",
            Self::Authentication => "Authentication Error",
            Self::DatabaseNotFound => "Database Not Found",
            Self::SslConfiguration => "SSL Configuration Error",
            Self::Configuration => "Configuration Error",
            Self::ServerError => "Database Server Error",
            Self::Unknown => "Unknown Error",
        }
    }
}

/// Structured connection error with user-friendly details
#[derive(Debug, Clone)]
pub struct ConnectionError {
    /// Type/category of error
    pub error_type: ConnectionErrorType,
    /// User-friendly message explaining what went wrong
    pub user_message: String,
    /// Original technical error message
    pub technical_details: String,
    /// List of actionable suggestions to fix the problem
    pub recovery_suggestions: Vec<String>,
    /// Optional database error code (e.g., PostgreSQL SQLSTATE)
    pub error_code: Option<String>,
}

impl ConnectionError {
    /// Create a new connection error
    pub fn new(
        error_type: ConnectionErrorType,
        user_message: impl Into<String>,
        technical_details: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            user_message: user_message.into(),
            technical_details: technical_details.into(),
            recovery_suggestions: Vec::new(),
            error_code: None,
        }
    }

    /// Add a recovery suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.recovery_suggestions.push(suggestion.into());
        self
    }

    /// Add multiple recovery suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.recovery_suggestions.extend(suggestions);
        self
    }

    /// Set the error code
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// Get a formatted error message for display
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();

        // Error category and message
        output.push_str(&format!("{}: {}\n", self.error_type.category_name(), self.user_message));

        // Technical details (if different from user message)
        if self.technical_details != self.user_message {
            output.push_str(&format!("\nDetails: {}\n", self.technical_details));
        }

        // Error code (if available)
        if let Some(ref code) = self.error_code {
            output.push_str(&format!("Error Code: {}\n", code));
        }

        // Recovery suggestions
        if !self.recovery_suggestions.is_empty() {
            output.push_str("\nTry the following:\n");
            for (i, suggestion) in self.recovery_suggestions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, suggestion));
            }
        }

        output
    }
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message)
    }
}

impl std::error::Error for ConnectionError {}
