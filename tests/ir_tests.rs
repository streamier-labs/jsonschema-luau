//! Unit tests for IR (Intermediate Representation) types.

use jsonschema_luau::{IrConfig, Type, TypeDef, TypeModule};

#[test]
fn test_ir_config_default() {
    let config = IrConfig::default();
    assert_eq!(config.root_type_name, "Root");
    assert!(config.include_constraints);
    assert!(config.include_descriptions);
}

#[test]
fn test_ir_config_with_root_type_name() {
    let config = IrConfig::with_root_type_name("Person");
    assert_eq!(config.root_type_name, "Person");
}

#[test]
fn test_type_module_new() {
    let module = TypeModule::new();
    assert!(module.root_type.is_none());
    assert!(module.definitions.is_empty());
}

#[test]
fn test_type_def_new() {
    let type_def = TypeDef::new("TestType", Type::String);
    assert_eq!(type_def.name, "TestType");
    assert!(type_def.exported);
    assert!(type_def.description.is_none());
}

#[test]
fn test_type_def_with_description() {
    let type_def = TypeDef::new("TestType", Type::Number).with_description("A test type");
    assert_eq!(type_def.description, Some("A test type".to_string()));
}

#[test]
fn test_type_def_with_export() {
    let type_def = TypeDef::new("TestType", Type::Boolean).with_export(false);
    assert!(!type_def.exported);
}

#[test]
fn test_type_variants() {
    // Test various type variants
    assert_eq!(Type::any(), Type::Any);
    assert_eq!(Type::never(), Type::Never);
    assert_eq!(Type::nil(), Type::Nil);
    assert_eq!(Type::boolean(), Type::Boolean);
    assert_eq!(Type::number(), Type::Number);
    assert_eq!(Type::string(), Type::String);
}

#[test]
fn test_type_literal_variants() {
    let str_lit = Type::StringLiteral("test".to_string());
    let num_lit = Type::NumberLiteral(42.0);
    let bool_lit = Type::BooleanLiteral(true);

    match str_lit {
        Type::StringLiteral(s) => assert_eq!(s, "test"),
        _ => panic!("Expected StringLiteral"),
    }

    match num_lit {
        Type::NumberLiteral(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected NumberLiteral"),
    }

    match bool_lit {
        Type::BooleanLiteral(b) => assert!(b),
        _ => panic!("Expected BooleanLiteral"),
    }
}

#[test]
fn test_type_array() {
    let arr_type = Type::array(Type::String);
    match arr_type {
        Type::Array(inner) => assert_eq!(*inner, Type::String),
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_type_union() {
    let union = Type::union(vec![Type::String, Type::Number]);
    match union {
        Type::Union(variants) => {
            assert_eq!(variants.len(), 2);
            // Types are sorted by debug format, so Number comes before String alphabetically
            assert!(variants.contains(&Type::String));
            assert!(variants.contains(&Type::Number));
        }
        _ => panic!("Expected Union type"),
    }
}

#[test]
fn test_type_reference() {
    let ref_type = Type::reference("SomeType");
    match ref_type {
        Type::Reference(name) => assert_eq!(name, "SomeType"),
        _ => panic!("Expected Reference type"),
    }
}
