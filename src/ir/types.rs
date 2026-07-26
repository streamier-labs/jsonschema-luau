//! IR Type definitions for representing schema types.

/// A complete type definition module containing all types.
#[derive(Debug, Clone, Default)]
pub struct TypeModule {
    /// The root type definition.
    pub root_type: Option<TypeDef>,
    /// All named type definitions (from definitions/$defs).
    pub definitions: Vec<TypeDef>,
}

impl TypeModule {
    /// Create a new empty type module.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A named type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    /// The name of the type (PascalCase).
    pub name: String,
    /// Optional description for the type.
    pub description: Option<String>,
    /// Optional auto-generated label/comment (e.g. for pure composition types).
    pub label: Option<String>,
    /// The actual type.
    pub ty: Type,
    /// Whether this type should be exported.
    pub exported: bool,
}

impl TypeDef {
    /// Create a new type definition.
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            description: None,
            label: None,
            ty,
            exported: true,
        }
    }

    /// Add a description to the type definition.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a label comment to the type definition.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set whether the type should be exported.
    pub fn with_export(mut self, exported: bool) -> Self {
        self.exported = exported;
        self
    }
}

/// Represents a type in the IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Any type (accepts anything).
    Any,
    /// Never type (accepts nothing).
    Never,
    /// Nil/null type.
    Nil,
    /// Boolean type.
    Boolean,
    /// Number type (includes integers).
    Number,
    /// String type.
    String,
    /// Literal string value.
    StringLiteral(String),
    /// Literal number value.
    NumberLiteral(f64),
    /// Literal boolean value.
    BooleanLiteral(bool),
    /// Array type with element type.
    Array(Box<Type>),
    /// Object type with properties.
    Object(Box<ObjectType>),
    /// Union of types (anyOf, oneOf, multi-type).
    Union(Vec<Type>),
    /// Union derived from a JSON Schema `enum` of string literals.
    /// Rendered without parentheses to mirror the legacy output.
    EnumUnion(Vec<Type>),
    /// Intersection of types (allOf).
    Intersection(Vec<Type>),
    /// Reference to another type by name.
    Reference(String),
    /// Map/dictionary type with value type.
    Map(Box<Type>),
    /// Tuple-like array with specific item types.
    Tuple(Vec<Type>),
}

impl Type {
    /// Create an any type.
    pub fn any() -> Self {
        Type::Any
    }

    /// Create a never type.
    pub fn never() -> Self {
        Type::Never
    }

    /// Create a nil type.
    pub fn nil() -> Self {
        Type::Nil
    }

    /// Create a boolean type.
    pub fn boolean() -> Self {
        Type::Boolean
    }

    /// Create a number type.
    pub fn number() -> Self {
        Type::Number
    }

    /// Create a string type.
    pub fn string() -> Self {
        Type::String
    }

    /// Create a string literal type.
    pub fn string_literal(s: impl Into<String>) -> Self {
        Type::StringLiteral(s.into())
    }

    /// Create a number literal type.
    pub fn number_literal(n: f64) -> Self {
        Type::NumberLiteral(n)
    }

    /// Create a boolean literal type.
    pub fn boolean_literal(b: bool) -> Self {
        Type::BooleanLiteral(b)
    }

    /// Create an array type.
    pub fn array(element: Type) -> Self {
        Type::Array(Box::new(element))
    }

    /// Create an object type.
    pub fn object(obj: ObjectType) -> Self {
        Type::Object(Box::new(obj))
    }

    /// Create a union type (anyOf, oneOf, multi-type). Preserves the
    /// source order of variants and flattens/deduplicates nested unions.
    pub fn union(types: Vec<Type>) -> Self {
        // Flatten nested unions
        let mut flattened: Vec<Type> = Vec::new();
        for ty in types {
            match ty {
                Type::Union(inner) => flattened.extend(inner),
                Type::EnumUnion(inner) => flattened.extend(inner),
                other => flattened.push(other),
            }
        }

        // Deduplicate while preserving order (keep first occurrence)
        let mut deduped: Vec<Type> = Vec::new();
        for ty in flattened {
            if !deduped.contains(&ty) {
                deduped.push(ty);
            }
        }

        match deduped.len() {
            0 => Type::Never,
            1 => deduped.remove(0),
            _ => Type::Union(deduped),
        }
    }

    /// Create an enum-derived union of string literals (no parentheses).
    pub fn enum_union(types: Vec<Type>) -> Self {
        match types.len() {
            0 => Type::Never,
            1 => types.into_iter().next().unwrap(),
            _ => Type::EnumUnion(types),
        }
    }

    /// Create an intersection type.
    pub fn intersection(types: Vec<Type>) -> Self {
        // Flatten nested intersections
        let mut flattened = Vec::new();
        for ty in types {
            match ty {
                Type::Intersection(inner) => flattened.extend(inner),
                other => flattened.push(other),
            }
        }

        match flattened.len() {
            0 => Type::Any,
            1 => flattened.remove(0),
            _ => Type::Intersection(flattened),
        }
    }

    /// Create a reference type.
    pub fn reference(name: impl Into<String>) -> Self {
        Type::Reference(name.into())
    }

    /// Create a map type.
    pub fn map(value: Type) -> Self {
        Type::Map(Box::new(value))
    }

    /// Create a tuple type.
    pub fn tuple(elements: Vec<Type>) -> Self {
        Type::Tuple(elements)
    }

    /// Check if this type is a simple primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Any
                | Type::Never
                | Type::Nil
                | Type::Boolean
                | Type::Number
                | Type::String
                | Type::StringLiteral(_)
                | Type::NumberLiteral(_)
                | Type::BooleanLiteral(_)
        )
    }

    /// Check if this type needs parentheses when used in a union/intersection.
    pub fn needs_parens(&self) -> bool {
        matches!(self, Type::Union(_) | Type::Intersection(_))
    }
}

/// Represents an object type with properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectType {
    /// The object's properties.
    pub properties: Vec<Property>,
    /// Whether additional properties are allowed.
    pub additional_properties: Option<AdditionalProperties>,
}

impl ObjectType {
    /// Create a new empty object type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a property to the object.
    pub fn with_property(mut self, prop: Property) -> Self {
        self.properties.push(prop);
        self
    }

    /// Set additional properties behavior.
    pub fn with_additional_properties(mut self, additional: AdditionalProperties) -> Self {
        self.additional_properties = Some(additional);
        self
    }

    /// Check if the object has any properties.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.additional_properties.is_none()
    }
}

/// A property of an object type.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The property name.
    pub name: String,
    /// The property type.
    pub ty: Type,
    /// Whether the property is optional.
    pub optional: bool,
    /// Optional description for the property.
    pub description: Option<String>,
    /// Constraints for the property.
    pub constraints: Constraints,
    /// Constraints hoisted from an `additionalProperties` sub-schema.
    pub extra_constraints: Constraints,
}

impl Property {
    /// Create a new required property.
    pub fn required(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            optional: false,
            description: None,
            constraints: Constraints::default(),
            extra_constraints: Constraints::default(),
        }
    }

    /// Create a new optional property.
    pub fn optional(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            optional: true,
            description: None,
            constraints: Constraints::default(),
            extra_constraints: Constraints::default(),
        }
    }

    /// Add a description to the property.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set constraints for the property.
    pub fn with_constraints(mut self, constraints: Constraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set extra (additionalProperties) constraints for the property.
    pub fn with_extra_constraints(mut self, constraints: Constraints) -> Self {
        self.extra_constraints = constraints;
        self
    }

    /// Mark the property as optional.
    pub fn make_optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Additional properties configuration for an object.
#[derive(Debug, Clone, PartialEq)]
pub enum AdditionalProperties {
    /// Additional properties are allowed with any type.
    Any,
    /// Additional properties are not allowed.
    False,
    /// Additional properties must match the specified schema.
    Schema(Box<Type>),
}

impl AdditionalProperties {
    /// Create additional properties that allow any type.
    pub fn any() -> Self {
        AdditionalProperties::Any
    }

    /// Create additional properties that disallow any extra properties.
    pub fn none() -> Self {
        AdditionalProperties::False
    }

    /// Create additional properties with a specific schema.
    pub fn schema(ty: Type) -> Self {
        AdditionalProperties::Schema(Box::new(ty))
    }
}

/// Constraints that can be applied to types.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Constraints {
    /// Minimum value for numbers.
    pub minimum: Option<f64>,
    /// Maximum value for numbers.
    pub maximum: Option<f64>,
    /// Exclusive minimum for numbers.
    pub exclusive_minimum: Option<f64>,
    /// Exclusive maximum for numbers.
    pub exclusive_maximum: Option<f64>,
    /// Multiple of constraint for numbers.
    pub multiple_of: Option<f64>,
    /// Minimum length for strings.
    pub min_length: Option<usize>,
    /// Maximum length for strings.
    pub max_length: Option<usize>,
    /// Pattern for strings.
    pub pattern: Option<String>,
    /// Format for strings.
    pub format: Option<String>,
    /// Minimum items for arrays.
    pub min_items: Option<usize>,
    /// Maximum items for arrays.
    pub max_items: Option<usize>,
    /// Unique items for arrays.
    pub unique_items: Option<bool>,
    /// Minimum properties for objects.
    pub min_properties: Option<usize>,
    /// Maximum properties for objects.
    pub max_properties: Option<usize>,
}

impl Constraints {
    /// Create empty constraints.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there are any constraints.
    pub fn is_empty(&self) -> bool {
        self.minimum.is_none()
            && self.maximum.is_none()
            && self.exclusive_minimum.is_none()
            && self.exclusive_maximum.is_none()
            && self.multiple_of.is_none()
            && self.min_length.is_none()
            && self.max_length.is_none()
            && self.pattern.is_none()
            && self.format.is_none()
            && self.min_items.is_none()
            && self.max_items.is_none()
            && self.unique_items.is_none()
            && self.min_properties.is_none()
            && self.max_properties.is_none()
    }

    /// Set minimum value.
    pub fn with_minimum(mut self, min: f64) -> Self {
        self.minimum = Some(min);
        self
    }

    /// Set maximum value.
    pub fn with_maximum(mut self, max: f64) -> Self {
        self.maximum = Some(max);
        self
    }

    /// Set exclusive minimum.
    pub fn with_exclusive_minimum(mut self, min: f64) -> Self {
        self.exclusive_minimum = Some(min);
        self
    }

    /// Set exclusive maximum.
    pub fn with_exclusive_maximum(mut self, max: f64) -> Self {
        self.exclusive_maximum = Some(max);
        self
    }

    /// Set multiple of.
    pub fn with_multiple_of(mut self, multiple: f64) -> Self {
        self.multiple_of = Some(multiple);
        self
    }

    /// Set minimum length.
    pub fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Set maximum length.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set pattern.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set minimum items.
    pub fn with_min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    /// Set maximum items.
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Set unique items.
    pub fn with_unique_items(mut self, unique: bool) -> Self {
        self.unique_items = Some(unique);
        self
    }

    /// Set minimum properties.
    pub fn with_min_properties(mut self, min: usize) -> Self {
        self.min_properties = Some(min);
        self
    }

    /// Set maximum properties.
    pub fn with_max_properties(mut self, max: usize) -> Self {
        self.max_properties = Some(max);
        self
    }
}
