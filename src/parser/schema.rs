//! JSON Schema to IR parser.

use convert_case::{Case, Casing};
use std::collections::{HashMap, HashSet};

use crate::error::{ConversionError, Result};
use crate::ir::{
    AdditionalProperties, Constraints, IrConfig, ObjectType, Property, Type, TypeDef, TypeModule,
};
use crate::schema::{
    AdditionalProperties as SchemaAdditionalProperties, JsonSchema, SchemaObject, SchemaType,
    SingleType,
};

/// Parser for converting JSON Schema to IR.
pub struct SchemaParser {
    /// Configuration for parsing.
    config: IrConfig,
    /// Collected definitions from the schema.
    definitions: HashMap<String, JsonSchema>,
    /// Set of already generated type names.
    generated_types: HashSet<String>,
}

impl SchemaParser {
    /// Create a new schema parser with default configuration.
    pub fn new() -> Self {
        Self::with_config(IrConfig::default())
    }

    /// Create a new schema parser with custom configuration.
    pub fn with_config(config: IrConfig) -> Self {
        Self {
            config,
            definitions: HashMap::new(),
            generated_types: HashSet::new(),
        }
    }

    /// Parse a JSON Schema into an IR type module.
    pub fn parse(&mut self, schema: &JsonSchema) -> Result<TypeModule> {
        // Extract definitions first
        self.extract_definitions(schema);

        let mut module = TypeModule::new();

        // Parse root type
        let root_name = self.config.root_type_name.to_case(Case::Pascal);
        let root_type = self.parse_to_type_def(schema, &root_name)?;
        module.root_type = Some(root_type);

        // Parse all definitions
        self.parse_definitions(&mut module)?;

        Ok(module)
    }

    /// Extract definitions from the schema.
    fn extract_definitions(&mut self, schema: &JsonSchema) {
        if let JsonSchema::Object(obj) = schema {
            // Extract from both definitions and $defs
            for defs in [&obj.definitions, &obj.defs].into_iter().flatten() {
                self.definitions.extend(defs.clone());
            }
        }
    }

    /// Parse all collected definitions into the module.
    fn parse_definitions(&mut self, module: &mut TypeModule) -> Result<()> {
        let mut def_names: Vec<_> = self.definitions.keys().cloned().collect();
        def_names.sort();

        for def_name in def_names {
            let pascal_name = def_name.to_case(Case::Pascal);
            if !self.generated_types.contains(&pascal_name)
                && let Some(def_schema) = self.definitions.get(&def_name).cloned()
            {
                let type_def = self.parse_to_type_def(&def_schema, &pascal_name)?;
                module.definitions.push(type_def);
            }
        }

        Ok(())
    }

    /// Parse a schema into a type definition.
    fn parse_to_type_def(&mut self, schema: &JsonSchema, name: &str) -> Result<TypeDef> {
        self.generated_types.insert(name.to_string());

        match schema {
            JsonSchema::Boolean(true) => Ok(TypeDef::new(name, Type::any())),
            JsonSchema::Boolean(false) => Ok(TypeDef::new(name, Type::never())),
            JsonSchema::Object(obj) => self.parse_object_to_def(obj, name),
        }
    }

    /// Parse an object schema into a type definition.
    fn parse_object_to_def(&mut self, obj: &SchemaObject, name: &str) -> Result<TypeDef> {
        let description = obj.description.clone();
        let label = Self::composition_label(obj);
        let ty = self.parse_schema_object_to_type(obj)?;

        let mut type_def = TypeDef::new(name, ty);
        if let Some(desc) = description {
            type_def = type_def.with_description(desc);
        }
        if let Some(label) = label {
            type_def = type_def.with_label(label);
        }

        Ok(type_def)
    }

    /// Determine the auto-generated label for a top-level pure composition type.
    /// Mirrors the legacy converter's behaviour: a comment is added only when the
    /// schema is resolved purely through allOf (without parent properties),
    /// anyOf, or oneOf.
    fn composition_label(obj: &SchemaObject) -> Option<&'static str> {
        if obj.all_of.is_some() {
            let parent_has_props = obj.properties.is_some()
                || obj.additional_properties.is_some()
                || obj.required.is_some();
            if !parent_has_props {
                return Some("Intersection type (all conditions must be met)");
            }
            // allOf with parent properties is merged into an object; no label.
            return None;
        }
        if obj.any_of.is_some() {
            return Some("Union type (any of these types)");
        }
        if obj.one_of.is_some() {
            return Some("Union type (exactly one of these types)");
        }
        None
    }

    /// Parse a schema to an IR type (for inline use).
    fn parse_to_type(&mut self, schema: &JsonSchema) -> Result<Type> {
        match schema {
            JsonSchema::Boolean(true) => Ok(Type::any()),
            JsonSchema::Boolean(false) => Ok(Type::never()),
            JsonSchema::Object(obj) => self.parse_schema_object_to_type(obj),
        }
    }

    /// Parse a schema object to an IR type.
    fn parse_schema_object_to_type(&mut self, obj: &SchemaObject) -> Result<Type> {
        // Handle $ref
        if let Some(ref_path) = &obj.ref_ {
            return self.resolve_ref(ref_path);
        }

        // Handle enum
        if let Some(enum_values) = &obj.enum_ {
            return Ok(self.parse_enum(enum_values));
        }

        // Handle const
        if let Some(const_value) = &obj.const_ {
            return Ok(self.parse_const(const_value));
        }

        // Handle composition types
        if let Some(ty) = self.parse_composition(obj)? {
            return Ok(ty);
        }

        // Handle type-specific parsing
        self.parse_type_specific(obj)
    }

    /// Resolve a $ref to a type reference.
    /// Note: Luau handles circular references naturally through type aliases,
    /// so we don't need to detect or prevent them here.
    fn resolve_ref(&mut self, ref_path: &str) -> Result<Type> {
        let def_name = ref_path
            .strip_prefix("#/definitions/")
            .or_else(|| ref_path.strip_prefix("#/$defs/"))
            .ok_or_else(|| ConversionError::unsupported_ref(ref_path))?;

        let pascal_name = def_name.to_case(Case::Pascal);
        Ok(Type::reference(pascal_name))
    }

    /// Parse enum values to a type.
    ///
    /// Mirrors the legacy converter: an all-number enum collapses to `number`,
    /// an all-string enum becomes a paren-free union of string literals (in
    /// source order), and any other enum becomes a catch-all union.
    fn parse_enum(&self, values: &[serde_json::Value]) -> Type {
        let (all_strings, all_numbers) =
            values
                .iter()
                .fold((true, true), |(strings, numbers), v| match v {
                    serde_json::Value::String(_) => (strings, false),
                    serde_json::Value::Number(_) => (false, numbers),
                    _ => (false, false),
                });

        if all_numbers {
            return Type::number();
        }

        if all_strings {
            let literals: Vec<Type> = values
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => Type::string_literal(s),
                    _ => unreachable!(),
                })
                .collect();
            return Type::enum_union(literals);
        }

        // Mixed/boolean/null enum: emit the legacy catch-all union (no parens).
        Type::enum_union(vec![
            Type::string(),
            Type::number(),
            Type::boolean(),
            Type::nil(),
        ])
    }

    /// Parse a const value to a literal type.
    ///
    /// Mirrors the legacy converter: numeric consts collapse to `number`.
    fn parse_const(&self, value: &serde_json::Value) -> Type {
        match value {
            serde_json::Value::String(s) => Type::string_literal(s),
            serde_json::Value::Number(_) => Type::number(),
            serde_json::Value::Bool(b) => Type::boolean_literal(*b),
            serde_json::Value::Null => Type::nil(),
            _ => Type::any(),
        }
    }

    /// Parse composition types (allOf, anyOf, oneOf).
    fn parse_composition(&mut self, obj: &SchemaObject) -> Result<Option<Type>> {
        // Handle allOf
        if let Some(all_of) = &obj.all_of {
            let has_parent_props = obj.properties.is_some()
                || obj.additional_properties.is_some()
                || obj.required.is_some();

            if has_parent_props {
                // Merge parent properties with non-ref allOf sub-schemas; refs
                // become separate intersection members.
                let mut merged_props = obj.properties.clone().unwrap_or_default();
                let mut merged_required = obj.required.clone().unwrap_or_default();
                let mut ref_types: Vec<Type> = Vec::new();

                for sub in all_of {
                    if let JsonSchema::Object(sub_obj) = sub {
                        if let Some(ref_path) = &sub_obj.ref_ {
                            ref_types.push(self.resolve_ref(ref_path)?);
                        } else {
                            if let Some(sub_props) = &sub_obj.properties {
                                merged_props.extend(sub_props.clone());
                            }
                            if let Some(sub_req) = &sub_obj.required {
                                merged_required.extend(sub_req.clone());
                            }
                        }
                    }
                }

                let mut merged = obj.clone();
                merged.properties = if merged_props.is_empty() {
                    None
                } else {
                    Some(merged_props)
                };
                merged.required = if merged_required.is_empty() {
                    None
                } else {
                    Some(merged_required)
                };
                merged.all_of = None;
                merged.description = None;

                let merged_type = self.parse_schema_object_to_type(&merged)?;

                if ref_types.is_empty() {
                    return Ok(Some(merged_type));
                }
                let mut parts = vec![merged_type];
                parts.extend(ref_types);
                return Ok(Some(Type::intersection(parts)));
            } else {
                // Create intersection type
                let types: Result<Vec<Type>> =
                    all_of.iter().map(|s| self.parse_to_type(s)).collect();
                return Ok(Some(Type::intersection(types?)));
            }
        }

        // Handle anyOf
        if let Some(any_of) = &obj.any_of {
            let types: Result<Vec<Type>> = any_of.iter().map(|s| self.parse_to_type(s)).collect();
            return Ok(Some(Type::union(types?)));
        }

        // Handle oneOf (treated as union in Luau)
        if let Some(one_of) = &obj.one_of {
            let types: Result<Vec<Type>> = one_of.iter().map(|s| self.parse_to_type(s)).collect();
            return Ok(Some(Type::union(types?)));
        }

        Ok(None)
    }

    /// Parse type-specific schema.
    fn parse_type_specific(&mut self, obj: &SchemaObject) -> Result<Type> {
        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            // Handle empty types (shouldn't happen with valid schemas, but be defensive)
            if types.is_empty() {
                return Ok(Type::any());
            }

            // Handle union of primitive types
            if types.len() > 1 {
                let type_strings: Vec<Type> =
                    types.iter().map(|t| self.single_type_to_type(t)).collect();
                return Ok(Type::union(type_strings));
            }

            // Handle single type
            let single_type = types[0];
            return self.parse_single_type(obj, single_type);
        }

        // Infer object if properties exist
        if obj.properties.is_some() || obj.additional_properties.is_some() {
            return self.parse_object_type(obj);
        }

        Ok(Type::any())
    }

    /// Get single types from SchemaType.
    fn get_single_types(schema_type: &SchemaType) -> Vec<&SingleType> {
        match schema_type {
            SchemaType::Single(single) => vec![single],
            SchemaType::Multiple(types) => types.iter().collect(),
        }
    }

    /// Convert a single type to an IR type.
    fn single_type_to_type(&self, single: &SingleType) -> Type {
        match single {
            SingleType::String => Type::string(),
            SingleType::Number | SingleType::Integer => Type::number(),
            SingleType::Boolean => Type::boolean(),
            SingleType::Null => Type::nil(),
            SingleType::Array => Type::array(Type::any()),
            SingleType::Object => Type::map(Type::any()),
        }
    }

    /// Parse a single type schema.
    fn parse_single_type(&mut self, obj: &SchemaObject, single: &SingleType) -> Result<Type> {
        match single {
            SingleType::Object => self.parse_object_type(obj),
            SingleType::Array => self.parse_array_type(obj),
            SingleType::String | SingleType::Number | SingleType::Integer => {
                Ok(self.single_type_to_type(single))
            }
            SingleType::Boolean => Ok(Type::boolean()),
            SingleType::Null => Ok(Type::nil()),
        }
    }

    /// Parse an object type schema.
    fn parse_object_type(&mut self, obj: &SchemaObject) -> Result<Type> {
        let mut object = ObjectType::new();

        // Parse properties
        if let Some(properties) = &obj.properties {
            let required_set: HashSet<_> = obj
                .required
                .as_ref()
                .map(|r| r.iter().cloned().collect())
                .unwrap_or_default();

            let mut prop_names: Vec<_> = properties.keys().cloned().collect();
            prop_names.sort();

            for prop_name in prop_names {
                if let Some(prop_schema) = properties.get(&prop_name) {
                    let is_required = required_set.contains(&prop_name);
                    let prop = self.parse_property(prop_schema, &prop_name, is_required)?;
                    object.properties.push(prop);
                }
            }
        }

        // Parse additionalProperties
        if let Some(additional) = &obj.additional_properties {
            object.additional_properties = Some(self.parse_additional_properties(additional)?);
        }

        Ok(Type::object(object))
    }

    /// Parse a property.
    fn parse_property(
        &mut self,
        schema: &JsonSchema,
        name: &str,
        is_required: bool,
    ) -> Result<Property> {
        let description = if let JsonSchema::Object(obj) = schema {
            obj.description.clone()
        } else {
            None
        };

        let constraints = self.extract_constraints(schema);
        let extra_constraints = self.extract_additional_constraints(schema);
        let ty = self.parse_to_type(schema)?;

        let mut prop = if is_required {
            Property::required(name, ty)
        } else {
            Property::optional(name, ty)
        };

        if let Some(desc) = description {
            prop = prop.with_description(desc);
        }

        if self.config.include_constraints {
            if !constraints.is_empty() {
                prop = prop.with_constraints(constraints);
            }
            if !extra_constraints.is_empty() {
                prop = prop.with_extra_constraints(extra_constraints);
            }
        }

        Ok(prop)
    }

    /// Parse additionalProperties.
    fn parse_additional_properties(
        &mut self,
        additional: &SchemaAdditionalProperties,
    ) -> Result<AdditionalProperties> {
        match additional {
            SchemaAdditionalProperties::Boolean(true) => Ok(AdditionalProperties::any()),
            SchemaAdditionalProperties::Boolean(false) => Ok(AdditionalProperties::none()),
            SchemaAdditionalProperties::Schema(schema) => {
                let ty = self.parse_to_type(schema)?;
                Ok(AdditionalProperties::schema(ty))
            }
        }
    }

    /// Parse an array type schema.
    fn parse_array_type(&mut self, obj: &SchemaObject) -> Result<Type> {
        let item_type = if let Some(items) = &obj.items {
            self.parse_to_type(items)?
        } else {
            Type::any()
        };

        Ok(Type::array(item_type))
    }

    /// Extract constraints from an `additionalProperties` sub-schema, if any.
    /// These are hoisted onto the enclosing property (mirroring the legacy
    /// converter, which rendered them as a separate constraint block after
    /// the property's own constraints).
    fn extract_additional_constraints(&self, schema: &JsonSchema) -> Constraints {
        if let JsonSchema::Object(obj) = schema
            && let Some(SchemaAdditionalProperties::Schema(add_schema)) = &obj.additional_properties
        {
            return self.extract_constraints(add_schema);
        }
        Constraints::new()
    }

    /// Extract constraints from a schema.
    fn extract_constraints(&self, schema: &JsonSchema) -> Constraints {
        if let JsonSchema::Object(obj) = schema {
            let mut constraints = Constraints::new();

            // Numeric constraints
            if let Some(min) = obj.minimum {
                constraints = constraints.with_minimum(min);
            }
            if let Some(max) = obj.maximum {
                constraints = constraints.with_maximum(max);
            }
            if let Some(ex_min) = obj.exclusive_minimum {
                constraints = constraints.with_exclusive_minimum(ex_min);
            }
            if let Some(ex_max) = obj.exclusive_maximum {
                constraints = constraints.with_exclusive_maximum(ex_max);
            }
            if let Some(multiple) = obj.multiple_of {
                constraints = constraints.with_multiple_of(multiple);
            }

            // String constraints
            if let Some(min_len) = obj.min_length {
                constraints = constraints.with_min_length(min_len);
            }
            if let Some(max_len) = obj.max_length {
                constraints = constraints.with_max_length(max_len);
            }
            if let Some(pattern) = &obj.pattern {
                constraints = constraints.with_pattern(pattern);
            }
            if let Some(format) = &obj.format {
                constraints = constraints.with_format(format);
            }

            // Array constraints
            if let Some(min_items) = obj.min_items {
                constraints = constraints.with_min_items(min_items);
            }
            if let Some(max_items) = obj.max_items {
                constraints = constraints.with_max_items(max_items);
            }
            if let Some(true) = obj.unique_items {
                constraints = constraints.with_unique_items(true);
            }

            // Object constraints
            if let Some(min_props) = obj.min_properties {
                constraints = constraints.with_min_properties(min_props);
            }
            if let Some(max_props) = obj.max_properties {
                constraints = constraints.with_max_properties(max_props);
            }

            constraints
        } else {
            Constraints::new()
        }
    }
}

impl Default for SchemaParser {
    fn default() -> Self {
        Self::new()
    }
}
