<!-- markdownlint-disable MD013 -->

# jsonschema-luau

![GitHub Release Date](https://img.shields.io/github/release-date/streamier-labs/jsonschema-luau?style=flat-square&logo=github&logoColor=FFFFFF&labelColor=111844&color=4B5694)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/streamier-labs/jsonschema-luau/ci.yml?style=flat-square&logo=github&logoColor=FFFFFF&labelColor=111844&color=4B5694)
![GitHub License](https://img.shields.io/github/license/streamier-labs/jsonschema-luau?style=flat-square&logo=github&logoColor=FFFFFF&labelColor=111844&color=4B5694)

Convert JSON Schemas to Luau type definitions with full support for constraints and advanced schema features.

---

## Features

- **Full JSON Schema support** (objects, arrays, primitives, enums, const)
- Handles **`$ref`**, **`definitions`**, and **`$defs`**
- Composition support (**`allOf`**, **`anyOf`**, **`oneOf`**)
- **Constraints preserved** as Luau comments (ranges, string limits, patterns, array bounds)
- Required and optional property handling
- Available both as a CLI tool and a Rust library
- Type-safe conversion with clear error handling

---

## Installation

### Package Managers (npm / npx)

Install globally or execute instantly via npm:

```bash
# Run directly without manual installation
npx @streamier/jsonschema-luau --help

# Or install globally
npm install -g @streamier/jsonschema-luau
```

### Shell Install Script (macOS / Linux / Windows)

You can install pre-built binaries directly using the interactive install scripts generated via `cargo-dist`:

**Linux & macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/streamier-labs/jsonschema-luau/releases/latest/download/jsonschema-luau-installer.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/streamier-labs/jsonschema-luau/releases/latest/download/jsonschema-luau-installer.ps1 | iex"
```

### Toolchain Managers (Mise / Rokit)

If you use toolchain managers for Roblox/Luau projects, this is the best way to ensure consistent tool versions across teams and CI workflows.

**Mise:**

```bash
mise use github:streamier-labs/jsonschema-luau
```

**Rokit:**

```bash
# Add to current project (pinned to rokit.toml)
rokit add streamier-labs/jsonschema-luau

# Install globally
rokit add streamier-labs/jsonschema-luau --global
```

### GitHub Releases

Pre-compiled binaries for Linux, macOS, and Windows (`aarch64` and `x86_64`) are available on the [GitHub Releases](https://github.com/streamier-labs/jsonschema-luau/releases) page.

### Cargo

**CLI Tool:**

```bash
cargo install jsonschema-luau
```

**Rust Library Dependency:**

```bash
cargo add jsonschema-luau
```

---

## Usage

### Command Line Interface (CLI)

```bash
# Convert a file and output to another file
jsonschema-luau schema.json -o types.luau

# Read schema from standard input
cat schema.json | jsonschema-luau - -o types.luau

# Specify a custom root type name (defaults to 'Root')
jsonschema-luau schema.json --type-name MyCustomType
```

### Rust Library

```rust
use jsonschema_luau::convert_schema;

fn main() {
    let json_schema = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": {
                "type": "number",
                "minimum": 0,
                "maximum": 120
            }
        },
        "required": ["name"]
    }"#;

    let luau_types = convert_schema(json_schema).unwrap();
    println!("{}", luau_types);
}
```

**Output:**

```lua
export type Root = {
    --- @minimum 0
    --- @maximum 120
    age: number?,
    name: string,
}
```

---

## Luau Type Mapping & Behavior

The converter maps JSON Schema concepts to the closest viable Luau types. Because **Luau is not TypeScript**, it has different type system capabilities and limitations.

### Primitive Mapping

| JSON Schema             | Luau Output  | Notes                                       |
| :---------------------- | :----------- | :------------------------------------------ |
| `"string"`              | `string`     |                                             |
| `"number"`, `"integer"` | `number`     |                                             |
| `"boolean"`             | `boolean`    |                                             |
| `"null"`                | `nil`        | Often combined: `string \| nil`             |
| enum (strings)          | `"a" \| "b"` | Uses union of literal strings               |
| enum (numbers)          | `number`     | Luau cannot represent numeric literal types |

### Complex Type Mapping

| JSON Schema       | Luau Output                             | Description                                                       |
| :---------------- | :-------------------------------------- | :---------------------------------------------------------------- |
| `array`           | `{ T }`                                 | General array type                                                |
| object map        | `{ [string]: T }`                       | Objects with `additionalProperties` or no `properties`            |
| `anyOf` / `oneOf` | union (`A \| B`)                        | `oneOf` exclusivity cannot be enforced in Luau                    |
| `allOf`           | intersection (`A & B`) or merged object | Intersection for standalone `allOf`, otherwise merged into parent |
| `$ref`, `$defs`   | `export type Name = ...`                | Always exports named types; never inlines                         |

---

## Composition Handling

### `anyOf` / `oneOf` (Union)

Both are converted to a Luau union type, as Luau does not enforce the exclusivity of `oneOf`.

```lua
export type T = A | B
```

### `allOf` (Intersection / Merging)

1. **If the parent schema defines properties**: `allOf` members are **merged** into the parent object type.
2. **Otherwise**: Converted to a Luau intersection (`export type T = A & B`).

---

## Code Examples

### Object with Constraints

```lua
export type Root = {
    --- @minimum 0
    --- @maximum 100
    age: number?,
    name: string,
}
```

### Arrays

```lua
--- @minItems 1
--- @maxItems 10
export type Root = { string }
```

### Enum

```lua
export type Root = "red" | "green" | "blue"
```

### Definitions (`$ref` / `$defs`)

```lua
export type Root = {
    person: Person?,
}

export type Person = {
    age: number?,
    name: string?,
}
```

> [!NOTE]
> Referenced types like `Person` are always exported as standalone named types.

---

## Limitations

Luau features a simpler type system than JSON Schema. The following features degrade gracefully:

- **Tuple schemas** (`items: [A, B, C]`) → Unsupported.
- **Conditionals** (`if` / `then` / `else`) → Ignored.
- **Dependencies** (`dependencies`, `dependentSchemas`, `dependentRequired`) → Ignored.
- **Pattern matching** (`patternProperties`, `propertyNames`) → Ignored or simplified.
- **Remote `$ref` resolution** → Only local fragments (`#/...`) are supported.
- **Number literal enums** → Fall back to `number`.
- **Exclusive constraints** → Cannot be enforced by the Luau type checker; documented via doc-comments only.

---

## Troubleshooting & FAQ

### Why is my numeric enum turned into `number`?

Luau does not support numeric literal types (e.g., `1 | 2 | 3`). Numeric enums in JSON Schema degrade to `number`.

### Why does my object turn into `{ [string]: any }`?

This occurs when the schema represents an object that permits arbitrary properties without declaring specific properties (`properties` is absent or empty, and `additionalProperties` defaults to `true`).

### Why is a type inlined instead of exported?

Only types resolved via `$ref` to a root-level definition (`#/definitions/Name` or `#/$defs/Name`) are exported as named types. Other complex nested types are inlined for conciseness.

---

## API Reference (Rust)

### `convert_schema(&str) -> Result<String>`

Parses the raw JSON Schema string and returns the generated Luau type definitions.

### `SchemaConverter`

Used for advanced configurations or maintaining state across multiple conversions:

```rust
let mut converter = SchemaConverter::new();
let luau = converter.convert(&schema)?;
let luau_with_name = converter.convert_with_name(&schema, "MyType")?;
```

---

## Development

This repository uses [`devenv`](https://devenv.sh/) to manage the development environment, tooling, and pre-commit hooks.

### Getting Started

1. Install [`nix`](https://nixos.org/) and [`devenv`](https://devenv.sh/).
2. Activate the shell environment:

```bash
devenv shell
```

### Included Tools & Hooks

The environment provides pre-configured tools including `cargo-dist` for packaging and `treefmt` for code formatting.

Automatic pre-commit checks will enforce:

- **Code Formatting:** `nixfmt`, `taplo` (TOML), `yamlfmt`, `rustfmt`, and `markdownlint`.
- **Static Analysis & Safety:** `clippy`, `actionlint` (GitHub Actions), `typos`, `deadnix`, and `statix`.
- **Hygiene Checks:** Line ending normalization, merge conflict checks, trailing whitespace cleanup, and commit message checking (`commitizen`).

To run formatting manually:

```bash
treefmt
```

---

## License

[MIT](LICENSE)
