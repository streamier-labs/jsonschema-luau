//! Unit tests for the Luau code generator.

use json_schema_to_luau::{LuauConfig, LuauGenerator, Type, TypeDef, TypeModule};

#[test]
fn test_generator_simple_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::String));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = string"));
}

#[test]
fn test_generator_with_description() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Number).with_description("A test type"));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("--- A test type"));
}

#[test]
fn test_generator_config_no_descriptions() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Number).with_description("A test type"));

    let config = LuauConfig {
        indent: "    ".to_string(),
        include_constraints: true,
        include_descriptions: false,
        module_export: true,
    };
    let generator = LuauGenerator::with_config(config);
    let output = generator.generate(&module);

    assert!(!output.contains("--- A test type"));
}

#[test]
fn test_generator_config_no_module_export() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Boolean));

    let config = LuauConfig {
        indent: "    ".to_string(),
        include_constraints: true,
        include_descriptions: true,
        module_export: false,
    };
    let generator = LuauGenerator::with_config(config);
    let output = generator.generate(&module);

    assert!(!output.contains("return {}"));
}

#[test]
fn test_generator_boolean_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Boolean));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = boolean"));
}

#[test]
fn test_generator_nil_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Nil));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = nil"));
}

#[test]
fn test_generator_any_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::Any));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = any"));
}

#[test]
fn test_generator_array_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::array(Type::Number)));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = { number }"));
}

#[test]
fn test_generator_union_type() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new(
        "Root",
        Type::union(vec![Type::String, Type::Number]),
    ));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    // The output should contain both types in a union (order may vary)
    assert!(output.contains("export type Root ="));
    assert!(output.contains("string"));
    assert!(output.contains("number"));
    assert!(output.contains("|"));
}

#[test]
fn test_generator_string_literal() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new(
        "Root",
        Type::StringLiteral("test".to_string()),
    ));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = \"test\""));
}

#[test]
fn test_generator_number_literal() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::NumberLiteral(42.0)));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = 42"));
}

#[test]
fn test_generator_reference() {
    let mut module = TypeModule::new();
    module.root_type = Some(TypeDef::new("Root", Type::reference("OtherType")));

    let generator = LuauGenerator::new();
    let output = generator.generate(&module);

    assert!(output.contains("export type Root = OtherType"));
}
