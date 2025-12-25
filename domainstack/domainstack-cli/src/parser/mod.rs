mod ast;
mod validation;

use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

pub use ast::ParsedType;
pub use validation::ValidationRule;

/// Parse all Rust files in a directory and extract types with validation rules
pub fn parse_directory(path: &Path) -> Result<Vec<ParsedType>> {
    let mut parsed_types = Vec::new();

    // Walk the directory recursively
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let file_path = entry.path();

        // Parse the file
        match parse_file(file_path) {
            Ok(mut types) => parsed_types.append(&mut types),
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", file_path.display(), e);
                // Continue parsing other files
            }
        }
    }

    Ok(parsed_types)
}

/// Parse a single Rust file and extract types with validation rules
fn parse_file(path: &Path) -> Result<Vec<ParsedType>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax_tree = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse Rust syntax: {}", path.display()))?;

    ast::extract_validated_types(&syntax_tree)
}
