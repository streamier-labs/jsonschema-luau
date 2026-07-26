//! JSON Schema to Luau type definition converter.
//!
//! This library converts JSON Schema documents to Luau type definitions using
//! an Intermediate Representation (IR) architecture:
//!
//! 1. **Parse**: JSON Schema → IR (Intermediate Representation)
//! 2. **Generate**: IR → Luau type definitions
//!
//! # Example
//!
//! ```rust
//! use json_schema_to_luau::convert_schema;
//!
//! let schema = r#"{
//!     "type": "object",
//!     "properties": {
//!         "name": { "type": "string" },
//!         "age": { "type": "number" }
//!     },
//!     "required": ["name"]
//! }"#;
//!
//! let luau = convert_schema(schema).unwrap();
//! println!("{}", luau);
//! ```

pub mod cli;
pub mod error;
pub mod generator;
pub mod ir;
pub mod parser;
pub mod schema;

pub use error::{ConversionError, Result};
pub use generator::{LuauConfig, LuauGenerator};
pub use ir::{IrConfig, Type, TypeDef, TypeModule};
pub use parser::SchemaParser;
pub use schema::JsonSchema;

/// Convert a JSON Schema string to Luau type definitions.
///
/// This is the main entry point for converting JSON Schemas to Luau.
/// It uses the default configuration and generates a root type named "Root".
///
/// # Arguments
///
/// * `json_schema` - A JSON string containing a valid JSON Schema.
///
/// # Returns
///
/// A `Result` containing the generated Luau type definitions or an error.
///
/// # Example
///
/// ```rust
/// use json_schema_to_luau::convert_schema;
///
/// let schema = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
/// let luau = convert_schema(schema).unwrap();
/// assert!(luau.contains("export type Root"));
/// ```
pub fn convert_schema(json_schema: &str) -> Result<String> {
    let schema: JsonSchema = serde_json::from_str(json_schema)?;
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema)?;
    let generator = LuauGenerator::new();
    Ok(generator.generate(&module))
}

/// Convert a JSON Schema string to Luau with a custom root type name.
///
/// # Arguments
///
/// * `json_schema` - A JSON string containing a valid JSON Schema.
/// * `type_name` - The name for the root type (will be converted to PascalCase).
///
/// # Returns
///
/// A `Result` containing the generated Luau type definitions or an error.
///
/// # Example
///
/// ```rust
/// use json_schema_to_luau::convert_schema_with_name;
///
/// let schema = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
/// let luau = convert_schema_with_name(schema, "Person").unwrap();
/// assert!(luau.contains("export type Person"));
/// ```
pub fn convert_schema_with_name(json_schema: &str, type_name: &str) -> Result<String> {
    let schema: JsonSchema = serde_json::from_str(json_schema)?;
    let config = IrConfig::with_root_type_name(type_name);
    let mut parser = SchemaParser::with_config(config);
    let module = parser.parse(&schema)?;
    let generator = LuauGenerator::new();
    Ok(generator.generate(&module))
}

/// Convert a parsed JSON Schema to Luau type definitions.
///
/// This function is useful when you have already parsed the JSON Schema
/// and want to convert it without re-parsing.
///
/// # Arguments
///
/// * `schema` - A parsed JSON Schema.
///
/// # Returns
///
/// A `Result` containing the generated Luau type definitions or an error.
pub fn convert_parsed_schema(schema: &JsonSchema) -> Result<String> {
    let mut parser = SchemaParser::new();
    let module = parser.parse(schema)?;
    let generator = LuauGenerator::new();
    Ok(generator.generate(&module))
}

/// Convert a parsed JSON Schema to Luau with a custom root type name.
///
/// # Arguments
///
/// * `schema` - A parsed JSON Schema.
/// * `type_name` - The name for the root type (will be converted to PascalCase).
///
/// # Returns
///
/// A `Result` containing the generated Luau type definitions or an error.
pub fn convert_parsed_schema_with_name(schema: &JsonSchema, type_name: &str) -> Result<String> {
    let config = IrConfig::with_root_type_name(type_name);
    let mut parser = SchemaParser::with_config(config);
    let module = parser.parse(schema)?;
    let generator = LuauGenerator::new();
    Ok(generator.generate(&module))
}
