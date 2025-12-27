//! Benchmark: ValidateOnDeserialize vs Separate Validation
//!
//! Measures the overhead of integrated validation during deserialization
//! compared to the two-step approach (deserialize then validate).
//!
//! Run with:
//! ```sh
//! cargo run --example serde_benchmark --release --features serde,regex
//! ```

use std::time::{Duration, Instant};

use domainstack::Validate;
use domainstack_derive::ValidateOnDeserialize;
use serde::Deserialize;

const ITERATIONS: u32 = 500_000;
const WARMUP_ITERATIONS: u32 = 50_000;
const RUNS: u32 = 5;

// Type using ValidateOnDeserialize (integrated validation)
#[derive(ValidateOnDeserialize, Debug)]
struct UserIntegrated {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    username: String,
}

// Type using separate Validate (two-step)
#[derive(Deserialize, Validate, Debug)]
struct UserSeparate {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(alphanumeric)]
    #[validate(min_len = 3)]
    #[validate(max_len = 20)]
    username: String,
}

// Baseline: No validation at all
#[derive(Deserialize, Debug)]
struct UserNoValidation {
    email: String,
    age: u8,
    username: String,
}

fn benchmark<F>(name: &str, iterations: u32, mut f: F) -> Duration
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..WARMUP_ITERATIONS {
        f();
    }

    // Multiple runs for stability
    let mut times = Vec::with_capacity(RUNS as usize);
    for _ in 0..RUNS {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        times.push(start.elapsed());
    }

    // Use median for stability
    times.sort();
    let median = times[times.len() / 2];
    let min = times[0];
    let max = times[times.len() - 1];

    let per_op = median / iterations;
    println!(
        "{:30} {:>8.2?}/op  (median of {} runs, range: {:.2?}-{:.2?})",
        name, per_op, RUNS, min / iterations, max / iterations
    );

    median
}

fn main() {
    println!("=== Serde Validation Benchmark ===\n");
    println!("Iterations per run: {}", ITERATIONS);
    println!("Runs: {} (using median)", RUNS);
    println!("Warmup: {}\n", WARMUP_ITERATIONS);

    let valid_json = r#"{"email": "alice@example.com", "age": 25, "username": "alice123"}"#;

    println!("--- Valid Input (all validations pass) ---\n");

    // Baseline: deserialize only, no validation
    let baseline = benchmark("1. Deserialize only (baseline)", ITERATIONS, || {
        let _: UserNoValidation = serde_json::from_str(valid_json).unwrap();
    });

    // Two-step: deserialize then validate
    let two_step = benchmark("2. Deserialize + .validate()", ITERATIONS, || {
        let user: UserSeparate = serde_json::from_str(valid_json).unwrap();
        user.validate().unwrap();
    });

    // Integrated: ValidateOnDeserialize
    let integrated = benchmark("3. ValidateOnDeserialize", ITERATIONS, || {
        let _: UserIntegrated = serde_json::from_str(valid_json).unwrap();
    });

    println!("\n--- Analysis ---\n");

    let validation_cost = two_step.saturating_sub(baseline);
    let integrated_cost = integrated.saturating_sub(baseline);
    let overhead = integrated.saturating_sub(two_step);

    println!(
        "Validation cost (2-step - baseline):     {:>10.2?}",
        validation_cost
    );
    println!(
        "Integrated cost (integrated - baseline): {:>10.2?}",
        integrated_cost
    );
    println!(
        "Integrated overhead (vs 2-step):         {:>10.2?}",
        overhead
    );

    if two_step.as_nanos() > 0 {
        let overhead_pct = (integrated.as_nanos() as f64 / two_step.as_nanos() as f64 - 1.0) * 100.0;
        println!(
            "\nIntegrated overhead vs 2-step:           {:>10.1}%",
            overhead_pct
        );

        if overhead_pct < 0.0 {
            println!("\n[!] Integrated is FASTER than 2-step (likely measurement noise)");
        } else if overhead_pct < 5.0 {
            println!("\n[✓] Overhead is under 5% as documented");
        } else if overhead_pct < 10.0 {
            println!("\n[~] Overhead is between 5-10%");
        } else {
            println!("\n[!] Overhead exceeds 10% - documentation may need update");
        }
    }

    // Also test with invalid input to see error path performance
    println!("\n--- Invalid Input (validation fails) ---\n");

    let invalid_json = r#"{"email": "not-an-email", "age": 15, "username": "ab"}"#;

    benchmark("2. Deserialize + .validate() [err]", ITERATIONS, || {
        let user: UserSeparate = serde_json::from_str(invalid_json).unwrap();
        let _ = user.validate(); // Ignore error
    });

    benchmark("3. ValidateOnDeserialize [err]", ITERATIONS, || {
        let _: Result<UserIntegrated, _> = serde_json::from_str(invalid_json);
    });

    println!("\n=============================================");
}
