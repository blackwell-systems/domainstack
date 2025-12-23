# Code Coverage Guide

This project uses `cargo-llvm-cov` for code coverage tracking, integrated with Codecov for CI.

## Quick Start

### One-Time Setup

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Add llvm-tools component
rustup component add llvm-tools-preview
```

### Run Coverage Locally

**Terminal output (default):**
```bash
cargo llvm-cov --all-features --workspace
```

**HTML report (recommended):**
```bash
cargo llvm-cov --all-features --workspace --html
# Opens target/llvm-cov/html/index.html
```

**Using the helper script:**
```bash
./scripts/coverage.sh
# Generates HTML and opens in browser automatically
```

## Output Formats

### Terminal Summary
```bash
cargo llvm-cov --all-features --workspace
```
Shows per-file coverage with:
- Region coverage (branches)
- Function coverage
- Line coverage

Example output:
```
Filename                             Lines      Missed Lines     Cover
------------------------------------------------------------------------
domainstack/src/error.rs               182                 0   100.00%
domainstack/src/rules/string.rs        123                 0   100.00%
domainstack-derive/src/lib.rs          329                73    77.81%
------------------------------------------------------------------------
TOTAL                                 1358                78    94.26%
```

### HTML Report (Interactive)
```bash
cargo llvm-cov --all-features --workspace --html
```
Creates browsable HTML report at `target/llvm-cov/html/index.html`

**Features:**
- Click files to see line-by-line coverage
- Red lines = not covered
- Green lines = covered
- Filter by coverage percentage
- Search functionality

### JSON Report (CI/Programmatic)
```bash
cargo llvm-cov --all-features --workspace --json --output-path coverage.json
```

### lcov Format (Codecov)
```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```
This is what CI uses to upload to Codecov.

## Understanding Coverage Metrics

### Line Coverage
Percentage of code lines executed by tests.
- **Target**: 80%+ for core library
- **Good**: Lines in error paths tested
- **Bad**: Large functions with no tests

### Region Coverage (Branches)
Percentage of conditional branches tested.
- **Example**: `if` statement with 2 branches (true/false)
- **Target**: 70%+
- **Important**: Tests both success and error cases

### Function Coverage
Percentage of functions called by tests.
- **Target**: 90%+
- **Easy win**: Often 100% achievable

## Current Coverage

As of the latest run:

| Crate | Line Coverage | Status |
|-------|--------------|--------|
| **domainstack** | 98-100% | ✅ Excellent |
| **domainstack-derive** | 77% | ⚠️ Needs improvement |
| **domainstack-envelope** | 100% | ✅ Excellent |
| **Overall** | 94% | ✅ Great |

## Improving Coverage

### Find Untested Code

```bash
# Generate HTML report and look for red lines
cargo llvm-cov --all-features --workspace --html
```

### Common Low-Coverage Areas

1. **Error paths** - Easy to miss in happy-path testing
   ```rust
   // Make sure to test the Err case!
   if condition {
       Ok(value)
   } else {
       Err(error)  // ← Test this path too
   }
   ```

2. **Proc macros** (domainstack-derive) - Hard to test
   - Use trybuild for compile-fail tests
   - Test generated code examples

3. **Edge cases** - Boundary conditions
   ```rust
   // Test: empty, min, max, overflow
   validate_length(0, 255)
   ```

### Writing Tests for Coverage

```rust
#[test]
fn test_error_path() {
    let result = validate("", &rules::non_empty());
    
    // Test the Err case
    assert!(result.is_err());
    
    // Also verify error details
    let err = result.unwrap_err();
    assert_eq!(err.violations[0].code, "non_empty");
}
```

## CI Integration

Coverage runs automatically on every push/PR:

1. **GitHub Actions** runs tests with coverage
2. **Codecov** receives and analyzes results
3. **PR comments** show coverage changes
4. **Badge** on README updates automatically

### Codecov Configuration

See `codecov.yml` for settings:
- Project target: 80%
- Patch target: 70%
- Ignore: examples, tests, benches

## Troubleshooting

### "command not found: cargo-llvm-cov"
```bash
cargo install cargo-llvm-cov
```

### "error: component 'llvm-tools' is unavailable"
```bash
rustup component add llvm-tools-preview
```

### Coverage seems wrong
```bash
# Clean and rebuild
cargo llvm-cov clean
cargo llvm-cov --all-features --workspace
```

### Coverage for specific crate only
```bash
cargo llvm-cov --all-features -p domainstack --html
```

### See uncovered lines in terminal
```bash
cargo llvm-cov --all-features --workspace --show-missing-lines
```

## Tips

1. **Run coverage before committing** - Catch regressions early
2. **Use HTML report** - Much easier to spot gaps than terminal output
3. **Focus on critical paths** - 100% coverage isn't always necessary
4. **Test error cases** - Often the least covered code
5. **Ignore generated code** - Configure in codecov.yml

## Further Reading

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov documentation](https://docs.codecov.io/)
- [LLVM coverage mapping](https://llvm.org/docs/CoverageMappingFormat.html)
