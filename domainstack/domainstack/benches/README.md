# Serde Integration Performance Benchmark

This document explains the methodology and results of benchmarking `ValidateOnDeserialize` (integrated validation) versus separate `Deserialize` + `.validate()` (two-step validation).

## TL;DR

| Scenario | Overhead |
|----------|----------|
| Valid input (success path) | **<2%** |
| Invalid input (error path) | **~17%** |

**Conclusion:** Integrated validation adds <2% overhead on the success path. Error path has higher overhead (~17%) due to error wrapping in `serde::Error`.

---

## Methodology

### Test Setup

```
Iterations per run: 500,000
Runs: 5 (using median for stability)
Warmup: 50,000 iterations
Build: Release mode (--release)
```

### Test Types

Three struct types with identical validation rules:

```rust
// 1. Baseline: No validation
#[derive(Deserialize)]
struct UserNoValidation { email, age, username }

// 2. Two-step: Deserialize, then validate
#[derive(Deserialize, Validate)]
struct UserSeparate {
    #[validate(email, max_len = 255)] email,
    #[validate(range(min = 18, max = 120))] age,
    #[validate(alphanumeric, min_len = 3, max_len = 20)] username,
}

// 3. Integrated: Validate during deserialization
#[derive(ValidateOnDeserialize)]
struct UserIntegrated { /* same validations */ }
```

### Measurement

- Each benchmark runs 5 times with 500K iterations each
- Median time is used (more stable than mean)
- Range (min-max) reported to show variance

---

## Results

### Valid Input (All Validations Pass)

```
1. Deserialize only (baseline) 126ns/op  (range: 125-137ns)
2. Deserialize + .validate()   675ns/op  (range: 665-708ns)
3. ValidateOnDeserialize       686ns/op  (range: 682-693ns)
```

**Analysis:**

| Metric | Value |
|--------|-------|
| Validation cost (two-step - baseline) | 549ns |
| Integrated cost (integrated - baseline) | 560ns |
| **Integrated overhead vs two-step** | **11ns (1.6%)** |

### Invalid Input (Validations Fail)

```
2. Deserialize + .validate() [err]   1.37µs/op  (range: 1.33-1.37µs)
3. ValidateOnDeserialize [err]       1.60µs/op  (range: 1.57-1.62µs)
```

**Analysis:**

| Metric | Value |
|--------|-------|
| **Integrated overhead vs two-step** | **~230ns (17%)** |

---

## Why the Difference?

### Success Path (~1.6% overhead)

The integrated approach has minimal overhead because:

1. **Same validation logic** - Both run identical validation code
2. **Struct copy is cheap** - Copying from intermediate to final struct is negligible for small types
3. **Compiler optimizations** - In release mode, much of the intermediate struct handling is optimized away

### Error Path (~17% overhead)

The error path has higher overhead because:

1. **Error wrapping** - `ValidateOnDeserialize` wraps `ValidationError` in `serde::Error`:
   ```rust
   serde::de::Error::custom(format!("Validation failed: {}", e))
   ```
2. **String formatting** - The `format!()` call adds allocation overhead
3. **Error type conversion** - Converting between error types has cost

---

## Recommendations

### When Overhead Matters

For most applications, 1-2% overhead is negligible. Consider the absolute numbers:

| Operation | Time |
|-----------|------|
| Deserialize + validate (success) | ~675-686ns |
| HTTP request/response | ~1-100ms |
| Database query | ~1-50ms |

Validation overhead is **~0.001%** of a typical API request.

### When to Use Each Approach

| Scenario | Recommendation |
|----------|----------------|
| API boundaries | `ValidateOnDeserialize` - negligible overhead, safer |
| High-frequency hot paths (>100K/sec) | Benchmark your specific case |
| Error-heavy workloads | `Validate` separate - lower error overhead |
| Draft/partial data | `Validate` separate - flexibility needed |

---

## Running the Benchmark

```bash
cargo run --example serde_bench --release --features serde,regex
```

The benchmark source is: `benches/serde_validation.rs`

---

## Summary

The benchmark confirms <2% overhead for integrated validation on the success path. This is negligible for virtually all use cases - validation overhead is ~0.001% of a typical API request.
