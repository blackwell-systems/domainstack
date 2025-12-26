use crate::generators;
use crate::parser;
use crate::JsonSchemaArgs;
use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn run(args: JsonSchemaArgs) -> Result<()> {
    // Run initial generation
    generate(&args)?;

    // If watch mode, start watching for changes
    if args.watch {
        watch(&args)?;
    }

    Ok(())
}

/// Generate JSON Schema from Rust files
fn generate(args: &JsonSchemaArgs) -> Result<()> {
    if args.verbose {
        println!("Parsing Rust files in: {}", args.input.display());
    }

    // Parse Rust files to find types with validation rules
    let parsed_types = parser::parse_directory(&args.input)
        .with_context(|| format!("Failed to parse directory: {}", args.input.display()))?;

    if args.verbose {
        println!(
            "[ok] Found {} types with validation rules",
            parsed_types.len()
        );
    }

    // Generate JSON Schema
    let json_schema = generators::json_schema::generate(&parsed_types)
        .context("Failed to generate JSON Schema")?;

    // Write output file
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    fs::write(&args.output, &json_schema)
        .with_context(|| format!("Failed to write output file: {}", args.output.display()))?;

    println!("[ok] Generated JSON Schema: {}", args.output.display());
    println!("  {} types processed", parsed_types.len());

    Ok(())
}

/// Watch for file changes and regenerate
fn watch(args: &JsonSchemaArgs) -> Result<()> {
    println!(
        "\n[watch] Watching for changes in: {}",
        args.input.display()
    );
    println!("[watch] Press Ctrl+C to stop\n");

    // Create a channel to receive events
    let (tx, rx) = channel();

    // Create a debounced watcher with 500ms delay
    let mut debouncer =
        new_debouncer(Duration::from_millis(500), tx).context("Failed to create file watcher")?;

    // Watch the input directory recursively
    debouncer
        .watcher()
        .watch(args.input.as_ref(), RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch directory: {}", args.input.display()))?;

    // Process events
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter for .rs file changes only
                let rust_changes: Vec<_> =
                    events.iter().filter(|e| is_rust_file(&e.path)).collect();

                if !rust_changes.is_empty() {
                    if args.verbose {
                        for event in &rust_changes {
                            println!("[change] {}", event.path.display());
                        }
                    }

                    println!("\n[watch] Changes detected, regenerating...");

                    match generate(args) {
                        Ok(()) => println!("[watch] Regeneration complete\n"),
                        Err(e) => {
                            eprintln!("[error] Regeneration failed: {}", e);
                            eprintln!("[watch] Waiting for more changes...\n");
                        }
                    }
                }
            }
            Ok(Err(error)) => {
                eprintln!("[error] Watch error: {:?}", error);
            }
            Err(e) => {
                eprintln!("[error] Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check if a path is a Rust source file
fn is_rust_file(path: &Path) -> bool {
    path.extension().map(|ext| ext == "rs").unwrap_or(false)
}
