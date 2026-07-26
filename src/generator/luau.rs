//! Luau code generator from IR.

use crate::ir::{
    AdditionalProperties, Constraints, ObjectType, Property, Type, TypeDef, TypeModule,
};

/// Configuration for Luau code generation.
#[derive(Debug, Clone)]
pub struct LuauConfig {
    /// Indentation string (default: 4 spaces).
    pub indent: String,
    /// Whether to include constraint annotations.
    pub include_constraints: bool,
    /// Whether to include descriptions as comments.
    pub include_descriptions: bool,
    /// Whether to add `return {}` at the end for module exports.
    pub module_export: bool,
}

impl Default for LuauConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            include_constraints: true,
            include_descriptions: true,
            module_export: true,
        }
    }
}

/// Generator for Luau type definitions from IR.
pub struct LuauGenerator {
    config: LuauConfig,
}

impl LuauGenerator {
    /// Create a new Luau generator with default configuration.
    pub fn new() -> Self {
        Self::with_config(LuauConfig::default())
    }

    /// Create a new Luau generator with custom configuration.
    pub fn with_config(config: LuauConfig) -> Self {
        Self { config }
    }

    /// Generate Luau code from a type module.
    pub fn generate(&self, module: &TypeModule) -> String {
        let mut output = String::new();

        // Generate root type
        if let Some(root_type) = &module.root_type {
            output.push_str(&self.generate_type_def(root_type));
        }

        // Generate definitions
        for def in &module.definitions {
            output.push_str("\n\n");
            output.push_str(&self.generate_type_def(def));
        }

        // Ensure single newline at end
        if !output.ends_with('\n') {
            output.push('\n');
        }

        // Add module export
        if self.config.module_export {
            output.push_str("\nreturn {}\n");
        }

        output
    }

    /// Generate a type definition.
    fn generate_type_def(&self, def: &TypeDef) -> String {
        let mut output = String::new();

        // Add description comment
        if self.config.include_descriptions {
            if let Some(desc) = &def.description {
                output.push_str(&format!("--- {}\n", desc));
            }
            if let Some(label) = &def.label {
                output.push_str(&format!("--- {}\n", label));
            }
        }

        // Generate the type
        let type_str = self.generate_type(&def.ty, 0);
        output.push_str(&format!("export type {} = {}", def.name, type_str));

        output
    }

    /// Generate a type representation.
    fn generate_type(&self, ty: &Type, indent: usize) -> String {
        self.generate_type_inner(ty, indent, false)
    }

    /// Generate a type representation with optional inline mode.
    fn generate_type_inner(&self, ty: &Type, indent: usize, inline: bool) -> String {
        match ty {
            Type::Any => "any".to_string(),
            Type::Never => "never".to_string(),
            Type::Nil => "nil".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::Number => "number".to_string(),
            Type::String => "string".to_string(),
            Type::StringLiteral(s) => format!("\"{}\"", self.escape_string(s)),
            Type::NumberLiteral(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Type::BooleanLiteral(b) => b.to_string(),
            Type::Array(elem) => self.generate_array_type_inner(elem, indent, inline),
            Type::Object(obj) => self.generate_object_type_inner(obj, indent, inline),
            Type::Union(types) => self.generate_union_type_inner(types, indent),
            Type::EnumUnion(types) => self.generate_enum_union_type_inner(types, indent),
            Type::Intersection(types) => self.generate_intersection_type_inner(types, indent),
            Type::Reference(name) => name.clone(),
            Type::Map(value) => self.generate_map_type(value),
            Type::Tuple(elements) => self.generate_tuple_type(elements, indent),
        }
    }

    /// Generate an array type with optional inline mode.
    fn generate_array_type_inner(&self, elem: &Type, indent: usize, _inline: bool) -> String {
        let elem_str = self.generate_type_inner(elem, indent, true);
        if elem.needs_parens() {
            format!("{{ ({}) }}", elem_str)
        } else {
            format!("{{ {} }}", elem_str)
        }
    }

    /// Generate an object type with optional inline mode.
    fn generate_object_type_inner(&self, obj: &ObjectType, indent: usize, inline: bool) -> String {
        // When inline mode is requested, generate single-line format
        if inline {
            return self.generate_object_type_inline(obj);
        }

        let indent_str = self.config.indent.repeat(indent);
        let inner_indent = self.config.indent.repeat(indent + 1);

        let mut output = String::from("{\n");

        // Generate properties
        for prop in &obj.properties {
            output.push_str(&self.generate_property(prop, &inner_indent));
        }

        // Generate additional properties
        if let Some(additional) = &obj.additional_properties {
            output.push_str(&self.generate_additional_properties(additional, &inner_indent));
        }

        output.push_str(&format!("{}}}", indent_str));
        output
    }

    /// Generate an object type in inline (single-line) format.
    fn generate_object_type_inline(&self, obj: &ObjectType) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Generate properties
        for prop in &obj.properties {
            let type_str = self.generate_type_inner(&prop.ty, 0, true);
            let optional_marker = if prop.optional { "?" } else { "" };
            parts.push(format!("{}: {}{}", prop.name, type_str, optional_marker));
        }

        // Generate additional properties
        if let Some(additional) = &obj.additional_properties {
            match additional {
                AdditionalProperties::Any => parts.push("[string]: any".to_string()),
                AdditionalProperties::False => {}
                AdditionalProperties::Schema(ty) => {
                    let type_str = self.generate_type_inner(ty, 0, true);
                    parts.push(format!("[string]: {}", type_str));
                }
            }
        }

        format!("{{ {} }}", parts.join(", "))
    }

    /// Generate a property.
    fn generate_property(&self, prop: &Property, indent_str: &str) -> String {
        let mut output = String::new();

        // Add description comment
        if self.config.include_descriptions
            && let Some(desc) = &prop.description
        {
            output.push_str(&format!("{}--- {}\n", indent_str, desc));
        }

        // Add constraint annotations
        if self.config.include_constraints && !prop.constraints.is_empty() {
            output.push_str(&self.generate_constraints(&prop.constraints, indent_str));
        }
        // Add hoisted additionalProperties constraints (rendered after own)
        if self.config.include_constraints && !prop.extra_constraints.is_empty() {
            output.push_str(&self.generate_constraints(&prop.extra_constraints, indent_str));
        }

        // Generate property type (use inline mode for nested types)
        let type_str = self.generate_type_inner(&prop.ty, 0, true);
        let optional_marker = if prop.optional { "?" } else { "" };

        // For union/intersection property types, wrap in parentheses so the
        // optional marker binds to the whole type (matches legacy output).
        let final_type = if prop.ty.needs_parens() {
            format!("({})", type_str)
        } else {
            type_str
        };

        output.push_str(&format!(
            "{}{}: {}{},\n",
            indent_str, prop.name, final_type, optional_marker
        ));

        output
    }

    /// Generate additional properties.
    fn generate_additional_properties(
        &self,
        additional: &AdditionalProperties,
        indent_str: &str,
    ) -> String {
        match additional {
            AdditionalProperties::Any => format!("{}[string]: any,\n", indent_str),
            AdditionalProperties::False => String::new(),
            AdditionalProperties::Schema(ty) => {
                let type_str = self.generate_type_inner(ty, 0, true);
                format!("{}[string]: {},\n", indent_str, type_str)
            }
        }
    }

    /// Generate constraint annotations.
    fn generate_constraints(&self, constraints: &Constraints, indent_str: &str) -> String {
        let mut output = String::new();

        if let Some(min) = constraints.minimum {
            output.push_str(&format!("{}--- @minimum {}\n", indent_str, min));
        }
        if let Some(max) = constraints.maximum {
            output.push_str(&format!("{}--- @maximum {}\n", indent_str, max));
        }
        if let Some(ex_min) = constraints.exclusive_minimum {
            output.push_str(&format!("{}--- @exclusiveMinimum {}\n", indent_str, ex_min));
        }
        if let Some(ex_max) = constraints.exclusive_maximum {
            output.push_str(&format!("{}--- @exclusiveMaximum {}\n", indent_str, ex_max));
        }
        if let Some(multiple) = constraints.multiple_of {
            output.push_str(&format!("{}--- @multipleOf {}\n", indent_str, multiple));
        }
        if let Some(min_len) = constraints.min_length {
            output.push_str(&format!("{}--- @minLength {}\n", indent_str, min_len));
        }
        if let Some(max_len) = constraints.max_length {
            output.push_str(&format!("{}--- @maxLength {}\n", indent_str, max_len));
        }
        if let Some(pattern) = &constraints.pattern {
            output.push_str(&format!("{}--- @pattern {}\n", indent_str, pattern));
        }
        if let Some(format) = &constraints.format {
            output.push_str(&format!("{}--- @format {}\n", indent_str, format));
        }
        if let Some(min_items) = constraints.min_items {
            output.push_str(&format!("{}--- @minItems {}\n", indent_str, min_items));
        }
        if let Some(max_items) = constraints.max_items {
            output.push_str(&format!("{}--- @maxItems {}\n", indent_str, max_items));
        }
        if let Some(true) = constraints.unique_items {
            output.push_str(&format!("{}--- @uniqueItems true\n", indent_str));
        }
        if let Some(min_props) = constraints.min_properties {
            output.push_str(&format!("{}--- @minProperties {}\n", indent_str, min_props));
        }
        if let Some(max_props) = constraints.max_properties {
            output.push_str(&format!("{}--- @maxProperties {}\n", indent_str, max_props));
        }

        output
    }

    /// Generate a union type. Members are always rendered inline; enclosing
    /// parentheses (when needed) are added by the caller.
    fn generate_union_type_inner(&self, types: &[Type], indent: usize) -> String {
        let parts: Vec<String> = types
            .iter()
            .map(|t| {
                let s = self.generate_type_inner(t, indent, true);
                if t.needs_parens() {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();

        parts.join(" | ")
    }

    /// Generate an enum-derived union of string literals. Never parenthesized.
    fn generate_enum_union_type_inner(&self, types: &[Type], indent: usize) -> String {
        let parts: Vec<String> = types
            .iter()
            .map(|t| self.generate_type_inner(t, indent, true))
            .collect();

        parts.join(" | ")
    }

    /// Generate an intersection type. Members are always rendered inline;
    /// enclosing parentheses (when needed) are added by the caller.
    fn generate_intersection_type_inner(&self, types: &[Type], indent: usize) -> String {
        let parts: Vec<String> = types
            .iter()
            .map(|t| {
                let s = self.generate_type_inner(t, indent, true);
                if t.needs_parens() {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();

        parts.join(" & ")
    }

    /// Generate a map type.
    fn generate_map_type(&self, value: &Type) -> String {
        let value_str = self.generate_type_inner(value, 0, true);
        format!("{{ [string]: {} }}", value_str)
    }

    /// Generate a tuple type.
    fn generate_tuple_type(&self, elements: &[Type], indent: usize) -> String {
        let parts: Vec<String> = elements
            .iter()
            .map(|t| self.generate_type_inner(t, indent, true))
            .collect();
        format!("{{ {} }}", parts.join(", "))
    }

    /// Escape a string for use in Luau.
    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

impl Default for LuauGenerator {
    fn default() -> Self {
        Self::new()
    }
}
