# Claude Code Context

This file provides context for Claude Code when working on the domainstack project.

## Dual Documentation Strategy

This project maintains documentation in two locations to serve different audiences:

### 1. GitHub Repository Root (`/README.md`)
- **Purpose**: Entry point for GitHub users browsing the repository
- **Audience**: Developers discovering the project on GitHub
- **Content**: Overview, quick start, feature highlights, links to detailed docs
- **Keep**: Concise and scannable

### 2. Workspace Documentation (`/domainstack/domainstack/docs/`)
- **Purpose**: Detailed documentation published to crates.io
- **Audience**: Users who have installed the crate and need reference material
- **Content**: Comprehensive guides, API reference, examples, patterns
- **Location**: `domainstack/domainstack/docs/*.md`

### Key Files:
| Location | File | Purpose |
|----------|------|---------|
| Root | `README.md` | GitHub landing page |
| Root | `CHANGELOG.md` | Version history for all crates |
| Workspace | `domainstack/domainstack/docs/INSTALLATION.md` | Complete installation guide |
| Workspace | `domainstack/domainstack/docs/DERIVE_MACRO.md` | Derive macro reference |
| Workspace | `domainstack/domainstack/docs/RULES.md` | All 37 validation rules |
| Workspace | `domainstack/domainstack/docs/JSON_SCHEMA.md` | JSON Schema generation |
| Workspace | `domainstack/domainstack/docs/OPENAPI_SCHEMA.md` | OpenAPI schema generation |
| Workspace | `domainstack/domainstack/docs/HTTP_INTEGRATION.md` | Framework adapters |

### When Updating Documentation:
1. **New features**: Update both root README.md (brief mention) AND detailed docs in workspace
2. **Bug fixes**: Update CHANGELOG.md at root
3. **API changes**: Update relevant docs in `domainstack/domainstack/docs/`
4. **Examples**: Add to `examples/` directory with entry in `examples/README.md`

## Project Structure

```
domainstack/                    # Repository root
├── README.md                   # GitHub landing page
├── CHANGELOG.md                # Version history
├── CLAUDE.md                   # This file
├── domainstack/                # Cargo workspace
│   ├── domainstack/            # Core validation crate
│   │   ├── src/
│   │   └── docs/               # Detailed documentation for crates.io
│   ├── domainstack-derive/     # Proc macros (Validate, ToSchema, ToJsonSchema)
│   ├── domainstack-schema/     # Schema generation (OpenAPI, JSON Schema)
│   ├── domainstack-envelope/   # HTTP error envelopes
│   ├── domainstack-axum/       # Axum framework adapter
│   ├── domainstack-actix/      # Actix-web framework adapter
│   ├── domainstack-rocket/     # Rocket framework adapter
│   ├── domainstack-http/       # Shared HTTP utilities
│   └── domainstack-wasm/       # WASM browser validation
└── examples/                   # Example applications
```

## Crate Publishing Order

When publishing to crates.io, crates must be published in dependency order:

1. `domainstack` (core)
2. `domainstack-derive` (depends on core)
3. `domainstack-schema` (depends on core)
4. `domainstack-envelope` (depends on core)
5. `domainstack-http` (depends on core, envelope)
6. `domainstack-axum` (depends on http)
7. `domainstack-actix` (depends on http)
8. `domainstack-rocket` (depends on http)
9. `domainstack-wasm` (depends on core, derive)

## Feature Flags

Key feature flags to be aware of:

- `domainstack`: `derive`, `regex`, `async`, `chrono`, `serde`
- `domainstack-derive`: `schema` (enables ToJsonSchema derive macro)

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with specific features
cargo test -p domainstack-derive --features schema

# Run clippy
cargo clippy --workspace --all-features
```
