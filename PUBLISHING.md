# Publishing to crates.io

This document describes how to publish domainstack crates to crates.io.

## First-Time Publishing (v0.3.0)

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

### Step 3: Publish domainstack-envelope

```bash
# Both dependencies are now on crates.io
cargo publish -p domainstack-envelope --token $CARGO_TOKEN
```

## Subsequent Releases

After the initial publish, the automated GitHub Actions workflow will handle releases:

1. Update version numbers in all `Cargo.toml` files (workspace and individual crates)
2. Update `CHANGELOG.md` with release notes
3. Commit changes: `git commit -am "Release v0.X.Y"`
4. Create and push tag: `git tag v0.X.Y && git push origin v0.X.Y`
5. GitHub Actions will automatically:
   - Create a GitHub release
   - Publish all three crates to crates.io in the correct order

## Publishing Order

Always publish in this order:
1. **domainstack-derive** - Has no workspace dependencies (only external deps)
2. **domainstack** - Depends on domainstack-derive
3. **domainstack-envelope** - Depends on both domainstack and domainstack-derive

## Version Synchronization

All three crates must have the same version number. Update them together:

```bash
# In workspace Cargo.toml
[workspace.dependencies]
domainstack = { version = "0.X.Y", path = "domainstack", default-features = false }
domainstack-derive = { version = "0.X.Y", path = "domainstack-derive" }
domainstack-envelope = { version = "0.X.Y", path = "domainstack-envelope" }

# In each crate's Cargo.toml
[package]
version = "0.X.Y"
```

## Pre-Publish Checklist

- [ ] All tests passing: `cargo test --all`
- [ ] Clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Docs build: `cargo doc --all --no-deps --all-features`
- [ ] Examples work: `cargo run -p domainstack-examples --example v2_basic`
- [ ] Version numbers match across all Cargo.toml files
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md badges show correct version
- [ ] Commit message follows format: "Release v0.X.Y"

## Troubleshooting

### "no matching package found" errors

This means you're trying to publish a crate that depends on another workspace crate that isn't on crates.io yet. Publish dependencies first.

### Workspace dependency issues

The workspace dependencies use `path` for local development and Cargo automatically substitutes with the version from crates.io when packaging. This only works if the dependency is already published.

### Dev-dependency circular reference

The `domainstack-derive` crate has `domainstack` in dev-dependencies for tests. For the initial publish, this must be temporarily removed or the package won't verify.
