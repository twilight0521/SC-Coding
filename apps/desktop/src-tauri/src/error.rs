use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Timeout")]
    Timeout,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Get a user-friendly error category for UI display
    pub fn category(&self) -> ErrorCategory {
        match self {
            AppError::Database(_) => ErrorCategory::System,
            AppError::Provider(_) | AppError::Network(_) => ErrorCategory::Network,
            AppError::Auth(_) | AppError::PermissionDenied(_) => ErrorCategory::Auth,
            AppError::RateLimit => ErrorCategory::RateLimit,
            AppError::Timeout => ErrorCategory::Timeout,
            AppError::Validation(_) => ErrorCategory::UserInput,
            AppError::NotFound(_) => ErrorCategory::UserInput,
            AppError::FileSystem(_) => ErrorCategory::System,
            AppError::Config(_) => ErrorCategory::UserInput,
            AppError::Internal(_) => ErrorCategory::System,
        }
    }

    /// Whether this error should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::Network(_) | AppError::RateLimit | AppError::Timeout | AppError::Database(_)
        )
    }

    /// Sanitized message safe for logging (removes sensitive info)
    pub fn safe_message(&self) -> String {
        let raw = self.to_string();
        redact_sensitive(&raw)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Network,
    Auth,
    RateLimit,
    Timeout,
    UserInput,
    System,
}

/// Simple redaction: replace long alphanumeric runs (likely API keys)
fn redact_sensitive(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut run = String::new();

    for c in input.chars() {
        if c.is_alphanumeric() {
            run.push(c);
        } else {
            if run.len() >= 20 {
                result.push_str("[REDACTED]");
            } else {
                result.push_str(&run);
            }
            run.clear();
            result.push(c);
        }
    }

    if run.len() >= 20 {
        result.push_str("[REDACTED]");
    } else {
        result.push_str(&run);
    }

    result
}

// ==================== Conversions ====================

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("Database row".to_string()),
            rusqlite::Error::SqliteFailure(err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Validation(err.to_string())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<crate::providers::ProviderError> for AppError {
    fn from(err: crate::providers::ProviderError) -> Self {
        match err {
            crate::providers::ProviderError::NetworkError(msg) => AppError::Network(msg),
            crate::providers::ProviderError::AuthError(msg) => AppError::Auth(msg),
            crate::providers::ProviderError::RateLimitExceeded => AppError::RateLimit,
            crate::providers::ProviderError::Timeout => AppError::Timeout,
            crate::providers::ProviderError::InvalidResponse(msg) => {
                AppError::Provider(format!("Invalid response: {}", msg))
            }
            crate::providers::ProviderError::ProviderNotFound(msg) => AppError::NotFound(msg),
            crate::providers::ProviderError::ModelNotFound(msg) => AppError::NotFound(msg),
            crate::providers::ProviderError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<crate::security::SecurityError> for AppError {
    fn from(err: crate::security::SecurityError) -> Self {
        match err {
            crate::security::SecurityError::NotFound => {
                AppError::NotFound("Secret not found".to_string())
            }
            crate::security::SecurityError::EncryptionError
            | crate::security::SecurityError::DecryptionError => {
                AppError::Internal("Encryption/decryption failed".to_string())
            }
            _ => AppError::Internal(err.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(err.to_string()),
            _ => AppError::FileSystem(err.to_string()),
        }
    }
}

// ==================== Result Type Alias ====================

pub type AppResult<T> = Result<T, AppError>;

// ==================== Tauri Command Result Helpers ====================

/// Convert AppError to a string for Tauri command return.
pub fn to_command_error(err: AppError) -> String {
    err.safe_message()
}

/// Trait for converting internal Result types to Tauri command-friendly Result<T, String>.
pub trait IntoCommandResult<T> {
    fn into_command_result(self) -> Result<T, String>;
}

impl<T> IntoCommandResult<T> for Result<T, AppError> {
    fn into_command_result(self) -> Result<T, String> {
        self.map_err(to_command_error)
    }
}

impl<T> IntoCommandResult<T> for Result<T, String> {
    fn into_command_result(self) -> Result<T, String> {
        self
    }
}

impl<T> IntoCommandResult<T> for Result<T, rusqlite::Error> {
    fn into_command_result(self) -> Result<T, String> {
        self.map_err(|e| to_command_error(AppError::from(e)))
    }
}
