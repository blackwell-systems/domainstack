# Contributing to domainstack

Thank you for your interest in contributing to domainstack! This guide will help you get started with development.

## Testing

```bash
cd domainstack

# Run all tests (149 unit + doc tests across all crates)
cargo test --all-features

# Test specific crate
cargo test -p domainstack --all-features
cargo test -p domainstack-derive
cargo test -p domainstack-envelope

# Run with coverage
cargo llvm-cov --all-features --workspace --html
```

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests to ensure everything passes
5. Submit a pull request

## Questions?

Feel free to open an issue if you have any questions or need help getting started.
