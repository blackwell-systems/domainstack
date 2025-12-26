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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Zod(args) => commands::zod::run(args)?,
        Commands::JsonSchema(args) => commands::json_schema::run(args)?,
    }

    Ok(())
}
