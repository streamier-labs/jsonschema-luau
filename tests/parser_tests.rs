//! Unit tests for the schema parser.

use jsonschema_luau::{JsonSchema, SchemaParser, Type};

#[test]
fn test_parse_simple_object() {
    let schema_json = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number" }
        },
        "required": ["name"]
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    assert_eq!(root.name, "Root");
}

#[test]
fn test_parse_array_type() {
    let schema_json = r#"{
        "type": "array",
        "items": { "type": "string" }
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
}

#[test]
fn test_parse_enum_type() {
    let schema_json = r#"{
        "type": "string",
        "enum": ["active", "pending", "deleted"]
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    // String enums become an enum-derived union of string literals.
    match root.ty {
        Type::EnumUnion(variants) => assert_eq!(variants.len(), 3),
        _ => panic!("Expected enum union type for enum"),
    }
}

#[test]
fn test_parse_with_definitions() {
    let schema_json = r##"{
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

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    // Should have definitions
    assert!(!module.definitions.is_empty());
}

#[test]
fn test_parse_with_defs() {
    let schema_json = r##"{
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

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    // Should have definitions
    assert!(!module.definitions.is_empty());
}

#[test]
fn test_parse_with_custom_type_name() {
    let schema_json = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let config = jsonschema_luau::IrConfig::with_root_type_name("Person");
    let mut parser = SchemaParser::with_config(config);
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    assert_eq!(root.name, "Person");
}

#[test]
fn test_parse_boolean_type() {
    let schema_json = r#"{
        "type": "boolean"
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    assert_eq!(root.ty, Type::Boolean);
}

#[test]
fn test_parse_null_type() {
    let schema_json = r#"{
        "type": "null"
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    assert_eq!(root.ty, Type::Nil);
}

#[test]
fn test_parse_any_of() {
    let schema_json = r#"{
        "anyOf": [
            { "type": "string" },
            { "type": "number" }
        ]
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    match root.ty {
        Type::Union(variants) => assert!(variants.len() >= 2),
        _ => panic!("Expected union type for anyOf"),
    }
}

#[test]
fn test_parse_with_description() {
    let schema_json = r#"{
        "type": "object",
        "description": "A user object",
        "properties": {
            "name": {
                "type": "string",
                "description": "The user name"
            }
        }
    }"#;

    let schema: JsonSchema = serde_json::from_str(schema_json).expect("Failed to parse schema");
    let mut parser = SchemaParser::new();
    let module = parser.parse(&schema).expect("Failed to convert to IR");

    assert!(module.root_type.is_some());
    let root = module.root_type.unwrap();
    assert_eq!(root.description, Some("A user object".to_string()));
}
