use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod generators;
mod parser;

/// Unified code generation CLI for domainstack
///
/// Generate TypeScript validators, GraphQL schemas, and more from Rust validation rules
#[derive(Parser)]
#[command(name = "domainstack")]
#[command(about = "Generate TypeScript validators, GraphQL schemas, and more from Rust validation rules", long_about = None)]
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
}

#[derive(Parser)]
struct ZodArgs {
    /// Input directory containing Rust source files
    #[arg(short, long, default_value = "src")]
    input: PathBuf,

    /// Output TypeScript file
    #[arg(short, long)]
    output: PathBuf,

    /// Watch for changes and regenerate (coming soon)
    #[arg(short, long)]
    watch: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Zod(args) => commands::zod::run(args)?,
    }

    Ok(())
}
