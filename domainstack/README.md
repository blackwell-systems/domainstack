# domainstack workspace

This is the Cargo workspace containing all domainstack crates.

- **GitHub/docs**: See [root README](../README.md)
- **crates.io**: See [domainstack crate](./domainstack/README.md)

## Workspace members

Run `cargo build --all` to build all crates.

| Crate | Description |
|-------|-------------|
| `domainstack` | Core validation library |
| `domainstack-derive` | `#[derive(Validate)]` macro (supports structs, tuple structs, enums) |
| `domainstack-schema` | OpenAPI schema generation |
| `domainstack-envelope` | HTTP error envelope integration |
| `domainstack-http` | Framework-agnostic HTTP helpers |
| `domainstack-axum` | Axum framework adapter |
| `domainstack-actix` | Actix-web framework adapter |
| `domainstack-rocket` | Rocket framework adapter |
| `domainstack-cli` | Code generation CLI (Zod, JSON Schema) with watch mode |
| `domainstack-wasm` | WASM browser validation |
