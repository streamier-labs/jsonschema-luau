//! JSON Schema to Luau type definition converter.
//!
//! This binary converts JSON Schema documents to Luau type definitions.
//!
//! # Usage Examples
//!
//! Generate Luau types to stdout:
//! ```sh
//! jsonschema-luau schema.json
//! ```
//!
//! Write output to a file:
//! ```sh
//! jsonschema-luau schema.json types.luau
//! ```
//!
//! Read from stdin:
//! ```sh
//! cat schema.json | jsonschema-luau
//! ```
//!
//! Read from stdin and write to a file:
//! ```sh
//! cat schema.json | jsonschema-luau -o types.luau
//! ```
//!
//! Use a custom root type name:
//! ```sh
//! jsonschema-luau schema.json -t MyType
//! ```

use json_schema_to_luau::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
