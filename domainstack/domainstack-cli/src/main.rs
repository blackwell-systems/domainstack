//! # domainstack-cli
//!
//! Code generation CLI for the domainstack validation ecosystem.
//!
//! Generate TypeScript/Zod schemas, JSON Schema, and OpenAPI specs from Rust `#[validate(...)]` attributes.
//!
//! ## Available Commands
//!
//! - `domainstack zod` - Generate TypeScript/Zod validation schemas
//! - `domainstack json-schema` - Generate JSON Schema (Draft 2020-12)
//! - `domainstack openapi` - Generate OpenAPI 3.0/3.1 specification
//!
//! ## Documentation
//!
//! - See `JSON_SCHEMA_CAPABILITIES.md` for complete JSON Schema feature reference
//! - See `README.md` for usage examples and CLI options
//! - See `examples/json_schema_demo.rs` for example types

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod generators;
mod parser;

/// Unified code generation CLI for domainstack
///
/// Generate TypeScript validators, JSON Schema, and more from Rust validation rules
#[derive(Parser)]
#[command(name = "domainstack")]
#[command(about = "Generate TypeScript validators, JSON Schema, and more from Rust validation rules", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Zod validation schemas (TypeScript)
    #[command(about = "Generate Zod validation schemas from Rust types")]
    Zod(ZodArgs),

    /// Generate JSON Schema (Draft 2020-12)
    #[command(about = "Generate JSON Schema from Rust types")]
    JsonSchema(JsonSchemaArgs),

    /// Generate OpenAPI specification (3.0/3.1)
    #[command(about = "Generate OpenAPI spec from Rust types")]
    Openapi(OpenApiArgs),
}

#[derive(Parser)]
pub struct ZodArgs {
    /// Input directory containing Rust source files
    #[arg(short, long, default_value = "src")]
    pub input: PathBuf,

    /// Output TypeScript file
    #[arg(short, long)]
    pub output: PathBuf,

    /// Watch for changes and regenerate automatically
    #[arg(short, long)]
    pub watch: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser)]
pub struct JsonSchemaArgs {
    /// Input directory containing Rust source files
    #[arg(short, long, default_value = "src")]
    pub input: PathBuf,

    /// Output JSON file
    #[arg(short, long)]
    pub output: PathBuf,

    /// Watch for changes and regenerate automatically
    #[arg(short, long)]
    pub watch: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser)]
pub struct OpenApiArgs {
    /// Input directory containing Rust source files
    #[arg(short, long, default_value = "src")]
    pub input: PathBuf,

    /// Output JSON file
    #[arg(short, long)]
    pub output: PathBuf,

    /// Use OpenAPI 3.1 (default is 3.0)
    #[arg(long)]
    pub openapi_31: bool,

    /// Watch for changes and regenerate automatically
    #[arg(short, long)]
    pub watch: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Zod(args) => commands::zod::run(args)?,
        Commands::JsonSchema(args) => commands::json_schema::run(args)?,
        Commands::Openapi(args) => commands::openapi::run(args)?,
    }

    Ok(())
}
