# Publishing to crates.io

This document describes how to publish domainstack crates to crates.io.

## Workspace Overview

The workspace contains **10 members total**:

**7 Publishable Crates:**
1. **domainstack-derive** - Procedural macros (no workspace dependencies)
2. **domainstack** - Core validation framework
3. **domainstack-http** - HTTP integration utilities
4. **domainstack-envelope** - Error envelope types for APIs
5. **domainstack-schema** - OpenAPI 3.0 schema generation
6. **domainstack-axum** - Axum web framework integration
7. **domainstack-actix** - Actix-web framework integration

**3 Example Crates (not published):**
8. **domainstack-examples** - Core validation examples
9. **examples-axum** - Axum framework examples
10. **examples-actix** - Actix-web framework examples

**Note on versions:**
- domainstack-schema is currently at v0.8.0 (newer crate)
- Other publishable crates will start at v1.0.0 for initial publication

## First-Time Publishing

Due to circular dev-dependencies between `domainstack` and `domainstack-derive`, the first publish requires manual steps:

### Step 1: Publish domainstack-derive

```bash
cd domainstack
# Temporarily comment out dev-dependency on domainstack in domainstack-derive/Cargo.toml
cargo publish -p domainstack-derive --token $CARGO_TOKEN
# Restore the dev-dependency
```

### Step 2: Publish domainstack (core)

```bash
# Now domainstack-derive is on crates.io, so domainstack can reference it
cargo publish -p domainstack --token $CARGO_TOKEN
```

### Step 3: Publish HTTP integration crates

```bash
# domainstack-http only depends on domainstack
cargo publish -p domainstack-http --token $CARGO_TOKEN

# domainstack-envelope only depends on domainstack
cargo publish -p domainstack-envelope --token $CARGO_TOKEN

# domainstack-schema only depends on domainstack
cargo publish -p domainstack-schema --token $CARGO_TOKEN
```

### Step 4: Publish framework integration crates

```bash
# Both framework crates depend on domainstack and domainstack-http
cargo publish -p domainstack-axum --token $CARGO_TOKEN
cargo publish -p domainstack-actix --token $CARGO_TOKEN
```

## Subsequent Releases

After the initial publish, the automated GitHub Actions workflow will handle releases:

1. Update version numbers in all `Cargo.toml` files (workspace and individual crates)
2. Update `CHANGELOG.md` with release notes
3. Commit changes: `git commit -am "Release v1.X.Y"`
4. Create and push tag: `git tag v1.X.Y && git push origin v1.X.Y`
5. GitHub Actions will automatically:
   - Create a GitHub release
   - Publish all eight crates to crates.io in the correct order

**Note:** domainstack-schema may have a different version number (currently v0.8.0) - manage its releases separately if needed.

## Publishing Order

Always publish in this order (respecting dependency chain):

1. **domainstack-derive** - No workspace dependencies (only external deps)
2. **domainstack** - Depends on domainstack-derive
3. **domainstack-http** - Depends on domainstack
4. **domainstack-envelope** - Depends on domainstack
5. **domainstack-schema** - Depends on domainstack
6. **domainstack-axum** - Depends on domainstack + domainstack-http
7. **domainstack-actix** - Depends on domainstack + domainstack-http
8. **domainstack-rocket** - Depends on domainstack + domainstack-http

**Parallel publishing (Step 3):** Crates 3-5 can be published in parallel since they only depend on domainstack.

**Parallel publishing (Step 4):** Crates 6-8 can be published in parallel since they have the same dependencies.

## Version Synchronization

**Most crates** should have synchronized version numbers (starting at v1.0.0):
- domainstack-derive
- domainstack
- domainstack-http
- domainstack-envelope
- domainstack-axum
- domainstack-actix

**Exception:** domainstack-schema is currently at v0.8.0 (newer crate, may evolve independently).

Update synchronized crates together:

```bash
# In workspace Cargo.toml
[workspace.dependencies]
domainstack = { version = "1.X.Y", path = "domainstack", default-features = false }
domainstack-derive = { version = "1.X.Y", path = "domainstack-derive" }
domainstack-http = { version = "1.X.Y", path = "domainstack-http" }
domainstack-envelope = { version = "1.X.Y", path = "domainstack-envelope" }
domainstack-axum = { version = "1.X.Y", path = "domainstack-axum" }
domainstack-actix = { version = "1.X.Y", path = "domainstack-actix" }
domainstack-schema = { version = "0.8.0", path = "domainstack-schema" }  # Separate versioning

# In each crate's Cargo.toml
[package]
version = "1.X.Y"  # or "0.8.Y" for domainstack-schema
```

## Pre-Publish Checklist

### Tests & Quality
- [ ] All tests passing: `cargo test --all`
- [ ] Clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Docs build: `cargo doc --all --no-deps --all-features`
- [ ] All doctests passing (especially schema generation examples)

### Examples Verification
- [ ] Core validation examples: `cargo run -p domainstack-examples --example v2_basic`
- [ ] Schema generation examples:
  - `cargo run -p domainstack-schema --example user_api`
  - `cargo run -p domainstack-schema --example v08_features`
- [ ] HTTP integration examples work
- [ ] Framework integration examples (Axum, Actix) compile and run

### Metadata & Documentation
- [ ] Version numbers match across all synchronized Cargo.toml files
- [ ] domainstack-schema version updated separately if needed (currently v0.8.0)
- [ ] CHANGELOG.md updated with release notes for all affected crates
- [ ] README.md examples use current syntax and features
- [ ] README.md badges show correct version
- [ ] All 7 crates have complete metadata (keywords, categories, description, etc.)

### Git & Release
- [ ] Commit message follows format: "Release v1.X.Y" (or "Release domainstack-schema v0.X.Y")
- [ ] Git tag created: `git tag v1.X.Y`
- [ ] No uncommitted changes
- [ ] Branch pushed to origin

## Troubleshooting

### "no matching package found" errors

This means you're trying to publish a crate that depends on another workspace crate that isn't on crates.io yet. **Publish dependencies first** following the dependency order above.

Example: Cannot publish `domainstack-axum` before publishing both `domainstack` and `domainstack-http`.

### Workspace dependency issues

The workspace dependencies use `path` for local development and Cargo automatically substitutes with the version from crates.io when packaging. This only works if the dependency is already published.

If you see errors about missing workspace dependencies, verify:
1. The dependency is already published to crates.io
2. The version in workspace Cargo.toml matches the published version
3. You're publishing in the correct order

### Dev-dependency circular reference

The `domainstack-derive` crate has `domainstack` in dev-dependencies for tests. For the initial publish, this must be temporarily removed or the package won't verify.

### Version mismatch errors

If different crates reference different versions of workspace dependencies:
1. Check all Cargo.toml files use `workspace = true` for workspace dependencies
2. Verify workspace Cargo.toml has correct versions in `[workspace.dependencies]`
3. Remember domainstack-schema has separate versioning (v0.8.x)

### Documentation build failures

If `cargo doc` fails:
1. Ensure all doctests have correct imports
2. Check schema generation examples use current API
3. Run `cargo doc -p <crate-name>` to isolate which crate is failing
4. Fix any broken doc links or invalid code examples

### Examples not running

If examples fail:
1. Verify feature flags are correct in example `Cargo.toml`
2. Check examples use current API (not deprecated syntax)
3. Ensure workspace dependencies are up to date
4. Run with `--verbose` to see detailed error messages
