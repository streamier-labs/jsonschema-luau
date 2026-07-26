//! Command-line interface for jsonschema-luau.

use std::io::{self, Read};
use std::path::PathBuf;

use clap::Parser;

use crate::error::Result;
use crate::generator::{LuauConfig, LuauGenerator};
use crate::ir::IrConfig;
use crate::parser::SchemaParser;
use crate::schema::JsonSchema;

/// Convert JSON Schema to Luau type definitions.
#[derive(Parser, Debug)]
#[command(name = "jsonschema-luau")]
#[command(version, about, long_about = None)]
#[command(override_usage = "jsonschema-luau [OPTIONS] [INPUT] [OUTPUT]")]
pub struct Cli {
    /// Input JSON Schema file path.
    /// If not provided or '-', reads from stdin.
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// Output file path.
    /// If not provided, writes to stdout.
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Read input from stdin.
    #[arg(short = 'i', long, conflicts_with = "input")]
    pub stdin: bool,

    /// Write output to file (alternative to positional OUTPUT argument).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Root type name for the generated types.
    /// Defaults to "Root".
    #[arg(short = 't', long, default_value = "Root")]
    pub type_name: String,

    /// Exclude constraint annotations from the output.
    #[arg(long)]
    pub no_constraints: bool,

    /// Exclude description comments from the output.
    #[arg(long)]
    pub no_descriptions: bool,

    /// Exclude the module export (`return {}`) at the end.
    #[arg(long)]
    pub no_module_export: bool,

    /// Indentation string (default: 4 spaces).
    /// Use "tab" for tabs or a number for space count.
    #[arg(long, value_name = "INDENT")]
    pub indent: Option<String>,

    /// Validate the schema without generating output.
    #[arg(long)]
    pub validate: bool,

    /// Print the parsed IR structure for debugging.
    #[arg(long, hide = true)]
    pub debug_ir: bool,
}

impl Cli {
    /// Run the CLI.
    pub fn run(self) -> Result<()> {
        // Read input
        let input_content = self.read_input()?;

        // Parse JSON Schema
        let schema: JsonSchema = serde_json::from_str(&input_content).map_err(|e| {
            crate::error::ConversionError::json_parse_with_location(
                e.to_string(),
                format!("line {}, column {}", e.line(), e.column()),
            )
        })?;

        // Validate only mode
        if self.validate {
            eprintln!("Schema is valid.");
            return Ok(());
        }

        // Debug IR mode
        if self.debug_ir {
            let config = IrConfig::with_root_type_name(&self.type_name);
            let mut parser = SchemaParser::with_config(config);
            let module = parser.parse(&schema)?;
            eprintln!("{:#?}", module);
            return Ok(());
        }

        // Configure parser
        let ir_config = IrConfig::with_root_type_name(&self.type_name);
        let mut parser = SchemaParser::with_config(ir_config);

        // Parse to IR
        let module = parser.parse(&schema)?;

        // Configure generator
        let luau_config = LuauConfig {
            indent: self.get_indent(),
            include_constraints: !self.no_constraints,
            include_descriptions: !self.no_descriptions,
            module_export: !self.no_module_export,
        };
        let generator = LuauGenerator::with_config(luau_config);

        // Generate output
        let output = generator.generate(&module);

        // Write output
        self.write_output(&output)?;

        Ok(())
    }

    /// Read input from file or stdin.
    fn read_input(&self) -> Result<String> {
        if self.stdin {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(crate::error::ConversionError::from)?;
            Ok(buffer)
        } else if let Some(input_path) = &self.input {
            std::fs::read_to_string(input_path).map_err(crate::error::ConversionError::from)
        } else {
            // No input file specified, read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(crate::error::ConversionError::from)?;
            Ok(buffer)
        }
    }

    /// Write output to file or stdout.
    fn write_output(&self, content: &str) -> Result<()> {
        // Determine output path (prefer -o flag over positional argument)
        let output_path = self.output_file.as_ref().or(self.output.as_ref());

        if let Some(path) = output_path {
            std::fs::write(path, content).map_err(crate::error::ConversionError::from)?;
        } else {
            print!("{}", content);
        }

        Ok(())
    }

    /// Get the indentation string.
    fn get_indent(&self) -> String {
        match &self.indent {
            Some(s) if s == "tab" => "\t".to_string(),
            Some(s) if s.chars().all(|c| c.is_ascii_digit()) => {
                let spaces: usize = s.parse().unwrap_or(4);
                " ".repeat(spaces)
            }
            Some(s) => s.clone(),
            None => "    ".to_string(),
        }
    }
}

/// Parse command-line arguments and run.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
