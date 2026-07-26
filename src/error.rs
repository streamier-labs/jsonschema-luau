//! Error types for the JSON Schema to Luau converter.

use thiserror::Error;

/// Result type alias for conversion operations.
pub type Result<T> = std::result::Result<T, ConversionError>;

/// Error type for conversion failures.
#[derive(Error, Debug)]
pub enum ConversionError {
    /// Failed to parse JSON.
    #[error("Failed to parse JSON: {message}")]
    JsonParse {
        /// Error message.
        message: String,
        /// Location in the JSON where the error occurred.
        location: Option<String>,
    },

    /// Failed to parse JSON Schema.
    #[error("Invalid JSON Schema: {message}")]
    SchemaParse {
        /// Error message.
        message: String,
        /// Path in the schema where the error occurred.
        path: Option<String>,
    },

    /// Unsupported schema feature.
    #[error("Unsupported schema feature: {feature}")]
    UnsupportedFeature {
        /// The unsupported feature.
        feature: String,
        /// Optional context about where the feature was encountered.
        context: Option<String>,
    },

    /// Unsupported $ref format.
    #[error(
        "Unsupported $ref format: '{ref_path}'. Only local references (#/definitions/... or #/$defs/...) are supported."
    )]
    UnsupportedRef {
        /// The reference path.
        ref_path: String,
    },

    /// Type resolution error.
    #[error("Failed to resolve type: {message}")]
    TypeResolution {
        /// Error message.
        message: String,
    },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with message.
    #[error("{0}")]
    Other(String),
}

impl ConversionError {
    /// Create a JSON parse error.
    pub fn json_parse(message: impl Into<String>) -> Self {
        ConversionError::JsonParse {
            message: message.into(),
            location: None,
        }
    }

    /// Create a JSON parse error with location.
    pub fn json_parse_with_location(
        message: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        ConversionError::JsonParse {
            message: message.into(),
            location: Some(location.into()),
        }
    }

    /// Create a schema parse error.
    pub fn schema_parse(message: impl Into<String>) -> Self {
        ConversionError::SchemaParse {
            message: message.into(),
            path: None,
        }
    }

    /// Create a schema parse error with path.
    pub fn schema_parse_with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        ConversionError::SchemaParse {
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Create an unsupported feature error.
    pub fn unsupported_feature(feature: impl Into<String>) -> Self {
        ConversionError::UnsupportedFeature {
            feature: feature.into(),
            context: None,
        }
    }

    /// Create an unsupported feature error with context.
    pub fn unsupported_feature_with_context(
        feature: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        ConversionError::UnsupportedFeature {
            feature: feature.into(),
            context: Some(context.into()),
        }
    }

    /// Create an unsupported $ref error.
    pub fn unsupported_ref(ref_path: impl Into<String>) -> Self {
        ConversionError::UnsupportedRef {
            ref_path: ref_path.into(),
        }
    }

    /// Create a type resolution error.
    pub fn type_resolution(message: impl Into<String>) -> Self {
        ConversionError::TypeResolution {
            message: message.into(),
        }
    }

    /// Create a generic error.
    pub fn other(message: impl Into<String>) -> Self {
        ConversionError::Other(message.into())
    }
}

impl From<serde_json::Error> for ConversionError {
    fn from(err: serde_json::Error) -> Self {
        ConversionError::JsonParse {
            message: err.to_string(),
            location: Some(format!("line {}, column {}", err.line(), err.column())),
        }
    }
}
