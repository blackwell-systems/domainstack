use crate::ZodArgs;
use crate::generators;
use crate::parser;
use anyhow::{Context, Result};
use std::fs;

pub fn run(args: ZodArgs) -> Result<()> {
    if args.watch {
        anyhow::bail!("Watch mode is not yet implemented. Coming in v0.2.0!");
    }

    if args.verbose {
        println!("🔍 Parsing Rust files in: {}", args.input.display());
    }

    // Parse Rust files to find types with validation rules
    let parsed_types = parser::parse_directory(&args.input)
        .with_context(|| format!("Failed to parse directory: {}", args.input.display()))?;

    if args.verbose {
        println!("✓ Found {} types with validation rules", parsed_types.len());
    }

    // Generate Zod schemas
    let typescript_code = generators::zod::generate(&parsed_types)
        .context("Failed to generate Zod schemas")?;

    // Write output file
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    fs::write(&args.output, typescript_code)
        .with_context(|| format!("Failed to write output file: {}", args.output.display()))?;

    println!("✓ Generated Zod schemas: {}", args.output.display());
    println!("  {} types processed", parsed_types.len());

    Ok(())
}
