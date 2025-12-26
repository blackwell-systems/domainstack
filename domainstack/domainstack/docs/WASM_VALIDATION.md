# WASM Browser Validation

Run the exact same Rust validation rules in the browser via WebAssembly. Zero translation drift, instant client-side feedback.

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│            Single Source of Truth                           │
│                                                             │
│    #[derive(Validate)]                                      │
│    struct Booking { ... }                                   │
│                                                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          │                         │
          ▼                         ▼
   ┌─────────────┐          ┌─────────────────┐
   │   Server    │          │   Browser       │
   │   (native)  │          │   (WASM)        │
   │             │          │                 │
   │ validate()  │          │ validate()      │
   │     ↓       │          │     ↓           │
   │ Same errors │  ◄────►  │ Same errors     │
   └─────────────┘          └─────────────────┘
```

The same `Validate` impl runs in both targets.

## Benefits

- **Zero drift** — Same Rust code compiled to WASM, not codegen
- **Instant feedback** — Validate on keystroke without server round-trip
- **Consistent errors** — Identical `ValidationError` structure on both sides
- **Small bundle** — ~60KB uncompressed, smaller gzipped
- **Type-safe bindings** — TypeScript definitions generated from Rust types

## Installation

### Rust

```toml
[dependencies]
domainstack = { version = "1.0", features = ["derive"] }
domainstack-wasm = "1.0"
```

### JavaScript/TypeScript

```bash
npm install @domainstack/wasm
```

## Quick Start

### 1. Define Validated Types

```rust
use domainstack::prelude::*;
use serde::Deserialize;

#[derive(Debug, Validate, Deserialize)]
struct Booking {
    #[validate(email)]
    guest_email: String,

    #[validate(range(min = 1, max = 10))]
    rooms: u8,
}
```

### 2. Register for WASM

```rust
use domainstack_wasm::register_type;

// Call once at initialization
fn init_validators() {
    register_type::<Booking>("Booking");
    register_type::<User>("User");
    // ... register all types you want available in browser
}
```

### 3. Build WASM

```bash
# Install target
rustup target add wasm32-unknown-unknown

# Build with wasm-pack
wasm-pack build --target web --release
```

### 4. Use in JavaScript

```typescript
import init, { createValidator } from '@domainstack/wasm';

// Initialize WASM module (once at app start)
await init();
const validator = createValidator();

// Validate any registered type
const result = validator.validate('Booking', JSON.stringify({
  guest_email: 'invalid',
  rooms: 15
}));

if (result.ok) {
  submitForm();
} else if (result.error) {
  // System error (unknown type, parse failure)
  console.error(`[${result.error.code}] ${result.error.message}`);
} else {
  // Validation errors
  result.errors.forEach(v => {
    console.log(`${v.path}: ${v.message}`);
  });
}
```

## API Reference

### TypeScript Types

```typescript
interface ValidationResult {
  ok: boolean;
  errors?: Violation[];      // Present when ok=false (validation failures)
  error?: SystemError;       // Present when ok=false (system error)
}

interface Violation {
  path: string;              // "guest_email", "rooms[0].adults"
  code: string;              // "invalid_email", "out_of_range"
  message: string;           // Human-readable message
  meta?: Record<string, string>;
}

interface SystemError {
  code: 'unknown_type' | 'parse_error';
  message: string;
}
```

### Result Guarantees

| Scenario | `ok` | `errors` | `error` |
|----------|------|----------|---------|
| Validation passed | `true` | `undefined` | `undefined` |
| Validation failed | `false` | `Violation[]` | `undefined` |
| Unknown type name | `false` | `undefined` | `SystemError` |
| JSON parse failed | `false` | `undefined` | `SystemError` |

### Validator Methods

```typescript
const validator = createValidator();

// Validate JSON string
validator.validate(typeName: string, json: string): ValidationResult

// Validate JS object (serializes internally)
validator.validateObject(typeName: string, obj: any): ValidationResult

// Check if type is registered
validator.hasType(typeName: string): boolean

// List registered types
validator.getTypes(): string[]
```

### Rust Functions

```rust
// Register a type for WASM validation
pub fn register_type<T>(type_name: &'static str)
where
    T: Validate + DeserializeOwned + 'static;

// Check if type is registered
pub fn is_type_registered(type_name: &str) -> bool;

// Get all registered type names
pub fn registered_types() -> Vec<&'static str>;
```

## React Integration

```tsx
import { useEffect, useState } from 'react';
import init, { createValidator, Validator } from '@domainstack/wasm';

function useValidator() {
  const [validator, setValidator] = useState<Validator | null>(null);

  useEffect(() => {
    init().then(() => setValidator(createValidator()));
  }, []);

  return validator;
}

function BookingForm() {
  const validator = useValidator();
  const [errors, setErrors] = useState<Violation[]>([]);

  const handleChange = (field: string, value: string) => {
    if (!validator) return;

    const result = validator.validate('Booking', JSON.stringify({
      ...form,
      [field]: value
    }));

    setErrors(result.errors ?? []);
  };

  return (
    <form>
      <input
        name="guest_email"
        onChange={e => handleChange('guest_email', e.target.value)}
      />
      {errors.find(e => e.path === 'guest_email')?.message}
    </form>
  );
}
```

## Handling Async Validators

Async validators (database checks, API calls) cannot run in the browser. Mark them to skip WASM:

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]                    // Runs in WASM
    email: String,

    #[validate(async, skip_wasm)]         // Server-only
    username: String,
}
```

## Cross-Field Validation

Cross-field rules work identically in WASM:

```rust
#[derive(Validate)]
#[validate(custom = date_range_valid)]
struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

fn date_range_valid(r: &DateRange) -> Result<(), ValidationError> {
    if r.end <= r.start {
        return Err(ValidationError::single("end", "invalid_range", "End must be after start"));
    }
    Ok(())
}
```

## Error Localization

Return error codes and localize on the client:

```typescript
const result = validator.validate('Booking', data);

if (!result.ok && result.errors) {
  const messages = result.errors.map(e => ({
    path: e.path,
    message: i18n.t(`validation.${e.code}`, e.meta)
  }));
}
```

## WASM vs Zod Codegen

| Approach | Pros | Cons |
|----------|------|------|
| **WASM** | Zero drift, same code | Requires WASM support, async init |
| **Zod codegen** | Pure JS, no WASM | Translation layer, potential drift |

**Use WASM when:**
- Validation accuracy is critical
- You need identical error structures
- Building SPAs with modern browser support

**Use Zod codegen when:**
- Targeting older browsers without WASM
- Building static sites
- Bundle size is extremely constrained

## Building for Production

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # No unwinding code
```

```bash
wasm-pack build --target web --release
```

Output in `pkg/`:
- `domainstack_wasm.js` — JavaScript bindings
- `domainstack_wasm.d.ts` — TypeScript types
- `domainstack_wasm_bg.wasm` — WASM binary

## See Also

- [Publishing Guide](./PUBLISHING.md) — Dual crates.io + npm workflow
- [domainstack-wasm README](../../domainstack-wasm/README.md) — Crate documentation
