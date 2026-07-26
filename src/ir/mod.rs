//! Intermediate Representation (IR) for type definitions.
//!
//! This module provides a language-agnostic representation of types that can be
//! derived from JSON Schema and then rendered to various target languages.

mod types;

pub use types::*;

/// Configuration for IR generation.
#[derive(Debug, Clone)]
pub struct IrConfig {
    /// Root type name for the schema.
    pub root_type_name: String,
    /// Whether to include constraint annotations.
    pub include_constraints: bool,
    /// Whether to include descriptions as comments.
    pub include_descriptions: bool,
}

impl Default for IrConfig {
    fn default() -> Self {
        Self {
            root_type_name: "Root".to_string(),
            include_constraints: true,
            include_descriptions: true,
        }
    }
}

impl IrConfig {
    /// Create a new IR config with a custom root type name.
    pub fn with_root_type_name(name: impl Into<String>) -> Self {
        Self {
            root_type_name: name.into(),
            ..Default::default()
        }
    }
}
