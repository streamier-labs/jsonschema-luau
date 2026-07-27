//! Unit tests for the conversion functions.

use jsonschema_luau::{convert_schema, convert_schema_with_name};

#[test]
fn test_convert_simple_schema() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number" }
        },
        "required": ["name"]
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("export type Root"));
    assert!(result.contains("name: string"));
    assert!(result.contains("age: number?"));
}

#[test]
fn test_convert_with_custom_name() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "email": { "type": "string" }
        }
    }"#;

    let result = convert_schema_with_name(schema, "User").expect("Conversion failed");
    assert!(result.contains("export type User"));
}

#[test]
fn test_convert_enum() {
    let schema = r#"{
        "type": "string",
        "enum": ["active", "pending", "deleted"]
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("\"active\""));
    assert!(result.contains("\"pending\""));
    assert!(result.contains("\"deleted\""));
}

#[test]
fn test_convert_array() {
    let schema = r#"{
        "type": "array",
        "items": { "type": "number" }
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("{ number }"));
}

#[test]
fn test_convert_boolean() {
    let schema = r#"{
        "type": "boolean"
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("export type Root = boolean"));
}

#[test]
fn test_convert_null() {
    let schema = r#"{
        "type": "null"
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("export type Root = nil"));
}

#[test]
fn test_convert_any_of() {
    let schema = r#"{
        "anyOf": [
            { "type": "string" },
            { "type": "number" }
        ]
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("string"));
    assert!(result.contains("number"));
}

#[test]
fn test_convert_with_description() {
    let schema = r#"{
        "type": "object",
        "description": "A user object",
        "properties": {
            "name": {
                "type": "string",
                "description": "The user name"
            }
        }
    }"#;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("--- A user object"));
}

#[test]
fn test_convert_invalid_json() {
    let invalid_json = r#"{ "type": "object", "properties": { "name": }"#;
    let result = convert_schema(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_convert_definitions() {
    let schema = r##"{
        "type": "object",
        "properties": {
            "address": { "$ref": "#/definitions/Address" }
        },
        "definitions": {
            "Address": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" }
                }
            }
        }
    }"##;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("export type Address"));
    assert!(result.contains("street: string"));
    assert!(result.contains("city: string"));
}

#[test]
fn test_convert_defs() {
    let schema = r##"{
        "type": "object",
        "properties": {
            "item": { "$ref": "#/$defs/Item" }
        },
        "$defs": {
            "Item": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                }
            }
        }
    }"##;

    let result = convert_schema(schema).expect("Conversion failed");
    assert!(result.contains("export type Item"));
    assert!(result.contains("id: string"));
}
