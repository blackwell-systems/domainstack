# Publishing Guide

This guide covers publishing all domainstack crates to crates.io.

## Crate Overview

| Crate | Registry | Description |
|-------|----------|-------------|
| `domainstack` | crates.io | Core validation library |
| `domainstack-derive` | crates.io | Derive macros |
| `domainstack-schema` | crates.io | OpenAPI schema generation |
| `domainstack-envelope` | crates.io | Response envelope types |
| `domainstack-http` | crates.io | Shared HTTP utilities |
| `domainstack-axum` | crates.io | Axum framework adapter |
| `domainstack-actix` | crates.io | Actix-web framework adapter |
| `domainstack-rocket` | crates.io | Rocket framework adapter |
| `domainstack-wasm` | crates.io + npm | WASM browser validation |

## Publishing Order

Crates must be published in dependency order. Wait for each crate to be indexed on crates.io before publishing dependents.

```
1. domainstack-derive     (no internal deps)
2. domainstack            (depends on derive)
3. domainstack-schema     (depends on core)
4. domainstack-envelope   (depends on core)
5. domainstack-http       (depends on core, envelope)
6. domainstack-axum       (depends on http)
7. domainstack-actix      (depends on http)
8. domainstack-rocket     (depends on http)
9. domainstack-wasm       (depends on core, derive) → also npm
```

## Pre-Release Checklist

### Version Bump

Update version in all Cargo.toml files:

```bash
# Find current versions
grep -r "^version = " */Cargo.toml

# Update versions (use your editor or sed)
# Ensure all crates use the same version for consistency
```

### Dependency Versions

Update inter-crate dependencies to match:

```toml
# In domainstack/Cargo.toml
[dependencies]
domainstack-derive = "1.0.0"  # Match the version you're publishing
```

### Pre-publish Validation

```bash
# Run all tests
cargo test --workspace --all-features

# Check documentation builds
cargo doc --workspace --no-deps

# Dry run publish (checks packaging)
cargo publish --dry-run -p domainstack-derive
cargo publish --dry-run -p domainstack
# ... repeat for each crate
```

## Publishing Commands

### Manual Publishing

```bash
# Authenticate (first time only)
cargo login

# Publish in order
cargo publish -p domainstack-derive
# Wait 1-2 minutes for crates.io indexing

cargo publish -p domainstack
# Wait 1-2 minutes

cargo publish -p domainstack-schema
cargo publish -p domainstack-envelope
cargo publish -p domainstack-http
cargo publish -p domainstack-axum
cargo publish -p domainstack-actix
cargo publish -p domainstack-rocket

# WASM requires special handling (see below)
```

### WASM Publishing

`domainstack-wasm` requires dual publishing. See [WASM_VALIDATION.md](./WASM_VALIDATION.md#publishing) for details.

```bash
cd domainstack-wasm

# 1. Publish Rust crate
cargo publish

# 2. Build and publish npm package
wasm-pack build --target web --release
cd pkg
npm publish --access public
```

## CI/CD Automation

### GitHub Actions Workflow

```yaml
# .github/workflows/publish.yml
name: Publish Crates

on:
  release:
    types: [published]

env:
  CARGO_TERM_COLOR: always

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Publish domainstack-derive
        run: cargo publish -p domainstack-derive
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Wait for crates.io indexing
        run: sleep 60

      - name: Publish domainstack
        run: cargo publish -p domainstack
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Wait for crates.io indexing
        run: sleep 60

      - name: Publish remaining crates
        run: |
          for crate in domainstack-schema domainstack-envelope domainstack-http domainstack-axum domainstack-actix domainstack-rocket; do
            cargo publish -p $crate
            sleep 30
          done
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  publish-wasm:
    runs-on: ubuntu-latest
    needs: publish
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: wasm-pack build --target web --release
        working-directory: domainstack-wasm

      - name: Publish to crates.io
        run: cargo publish -p domainstack-wasm
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Publish to npm
        run: npm publish --access public
        working-directory: domainstack-wasm/pkg
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### Required Secrets

Configure these in GitHub repository settings:

| Secret | Description |
|--------|-------------|
| `CARGO_REGISTRY_TOKEN` | crates.io API token |
| `NPM_TOKEN` | npmjs.com access token (for WASM) |

## Version Strategy

### Semantic Versioning

All crates follow [SemVer](https://semver.org/):

- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Coordinated Releases

All crates share the same version number for simplicity:

```
domainstack           1.0.0
domainstack-derive    1.0.0
domainstack-schema    1.0.0
domainstack-axum      1.0.0
...
```

### Changelog

Maintain a CHANGELOG.md at the workspace root:

```markdown
## [1.0.0] - 2024-01-15

### Added
- Initial stable release
- Core validation with 37 rules
- Derive macros for declarative validation
- Framework adapters (Axum, Actix, Rocket)
- OpenAPI schema generation
- WASM browser validation
```

## Post-Publish Verification

After publishing, verify crates are available:

```bash
# Check crates.io
cargo search domainstack

# Check npm (for WASM)
npm view @domainstack/wasm

# Test installation in a fresh project
cargo new test-domainstack
cd test-domainstack
cargo add domainstack
cargo build
```

## Yanking a Release

If a critical bug is found after publishing:

```bash
# Yank a specific version (discourages use but doesn't delete)
cargo yank --version 1.0.0 domainstack

# Unyank if the issue is resolved
cargo yank --undo --version 1.0.0 domainstack
```

For npm:

```bash
npm deprecate @domainstack/wasm@1.0.0 "Critical bug, use 1.0.1"
```

## Troubleshooting

### "crate not found" during publish

Wait 1-2 minutes for crates.io to index the dependency, then retry.

### "version already exists"

The version is already published. Bump the version number.

### npm publish fails with 403

Check that your npm token has publish permissions and the package name is available.

### WASM build fails

Ensure the `wasm32-unknown-unknown` target is installed:

```bash
rustup target add wasm32-unknown-unknown
```
