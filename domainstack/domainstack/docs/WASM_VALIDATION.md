# WASM Browser Validation

> **Status:** Proposal
> **Target:** domainstack 2.0

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

## Goals

1. **Zero drift** — Not codegen, actual same Rust code compiled to WASM
2. **Instant feedback** — Validate on keystroke without server round-trip
3. **Consistent errors** — Identical `ValidationError` structure on both sides
4. **Small bundle** — Target < 50KB gzipped for validation logic
5. **Type-safe bindings** — TypeScript definitions generated from Rust types

## Architecture

### Crate Structure

```
domainstack-wasm/
├── Cargo.toml
├── src/
│   ├── lib.rs           # WASM entry point
│   ├── registry.rs      # Type registry for dynamic dispatch
│   ├── bindings.rs      # JS/TS bindings via wasm-bindgen
│   └── serde_bridge.rs  # JSON ↔ Rust conversion
├── js/
│   ├── index.ts         # TypeScript wrapper
│   └── types.ts         # Generated type definitions
└── examples/
    └── react-form/      # React integration example
```

### Core Components

```rust
// 1. Validation Registry — register types at compile time
#[wasm_bindgen]
pub struct ValidationRegistry {
    validators: HashMap<&'static str, Box<dyn DynValidator>>,
}

// 2. Dynamic Validator Trait — type-erased validation
trait DynValidator: Send + Sync {
    fn validate_json(&self, json: &str) -> Result<(), ValidationError>;
    fn schema(&self) -> Schema;
}

// 3. WASM Entry Point
#[wasm_bindgen]
pub fn validate(type_name: &str, json: &str) -> JsValue {
    REGISTRY.with(|r| {
        match r.get(type_name) {
            Some(validator) => {
                match validator.validate_json(json) {
                    Ok(()) => JsValue::NULL,
                    Err(e) => serde_wasm_bindgen::to_value(&e).unwrap(),
                }
            }
            None => panic!("Unknown type: {}", type_name),
        }
    })
}
```

## API Design

### Rust Side — Registration Macro

```rust
use domainstack::prelude::*;
use domainstack_wasm::wasm_validate;

#[derive(Validate, Deserialize)]
#[wasm_validate]  // Registers for WASM export
struct Booking {
    #[validate(email)]
    guest_email: String,

    #[validate(range(min = 1, max = 10))]
    rooms: u8,
}

// Generated registration (by proc macro):
inventory::submit! {
    WasmValidator::new::<Booking>("Booking")
}
```

### JavaScript/TypeScript Side

```typescript
// npm install @domainstack/wasm

import { createValidator } from '@domainstack/wasm';

// Initialize WASM module (async, do once at app start)
const validator = await createValidator();

// Validate any registered type
const result = validator.validate('Booking', {
  guest_email: 'invalid',
  rooms: 15
});

if (!result.ok) {
  // Same structure as server response!
  result.errors.forEach(violation => {
    console.log(violation.path);    // "guest_email"
    console.log(violation.code);    // "invalid_email"
    console.log(violation.message); // "Invalid email format"
  });
}

// TypeScript types generated from Rust
interface ValidationResult {
  ok: boolean;
  errors?: Violation[];
}

interface Violation {
  path: string;
  code: string;
  message: string;
  meta?: Record<string, string>;
}
```

### React Integration

```tsx
import { useValidator } from '@domainstack/react';

function BookingForm() {
  const { validate, errors } = useValidator('Booking');

  const handleChange = (field: string, value: unknown) => {
    setForm(prev => ({ ...prev, [field]: value }));

    // Instant validation on change
    const result = validate({ ...form, [field]: value });
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

## Implementation Phases

### Phase 1: Core WASM Infrastructure
- [ ] Create `domainstack-wasm` crate
- [ ] Implement `ValidationRegistry` with `inventory` crate
- [ ] Add `#[wasm_validate]` proc macro to `domainstack-derive`
- [ ] Basic `validate(type_name, json)` function
- [ ] Error serialization to JS-compatible format

### Phase 2: TypeScript Bindings
- [ ] Generate TypeScript type definitions
- [ ] Create npm package structure
- [ ] Add async initialization wrapper
- [ ] Publish to npm as `@domainstack/wasm`

### Phase 3: Framework Integrations
- [ ] React hooks (`@domainstack/react`)
- [ ] Vue composables (`@domainstack/vue`)
- [ ] Svelte stores (`@domainstack/svelte`)
- [ ] Vanilla JS helpers

### Phase 4: Developer Experience
- [ ] Hot reload support (re-init WASM on change)
- [ ] DevTools integration (validation timeline)
- [ ] Bundle size optimization
- [ ] Documentation and examples

## Technical Considerations

### Bundle Size

Target budget: **< 50KB gzipped**

Strategies:
- Use `wasm-opt` with size optimization (`-Oz`)
- Exclude unused validators via feature flags
- Lazy-load validation logic per type
- Consider `wasm-bindgen` alternatives (`wasm-pack` size analysis)

```toml
# Cargo.toml optimizations
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # No unwinding code
```

### Async Validation

Async rules (DB checks) can't run in browser. Strategy:

```rust
#[derive(Validate)]
struct User {
    #[validate(email)]                    // ✓ Runs in WASM
    email: String,

    #[validate(async, skip_wasm)]         // ✗ Server-only
    #[validate(unique_in_db)]
    username: String,
}
```

The `skip_wasm` attribute excludes async validators from WASM bundle.

### Cross-Field Validation

Cross-field rules work identically:

```rust
#[derive(Validate)]
#[validate(check = "self.end > self.start", message = "End must be after start")]
struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}
```

Compiles to WASM and produces same error structure.

### Error Message Localization

Two approaches:

**Option A: Server-side messages (smaller bundle)**
```typescript
const result = validator.validate('Booking', data);
// Returns error codes, client maps to localized messages
const message = i18n.t(`validation.${result.errors[0].code}`);
```

**Option B: WASM-embedded messages (self-contained)**
```rust
// Compile with locale feature
domainstack-wasm = { features = ["locale-en", "locale-es"] }
```

Recommend Option A for bundle size.

## Dependencies

```toml
[dependencies]
wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"
js-sys = "0.3"
inventory = "0.3"           # Compile-time registration
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

## Testing Strategy

### Rust Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_validation_registry() {
        let registry = ValidationRegistry::new();
        registry.register::<Booking>("Booking");

        let result = registry.validate("Booking", r#"{"guest_email": "bad"}"#);
        assert!(result.is_err());
    }
}
```

### WASM Integration Tests
```rust
#[wasm_bindgen_test]
fn test_validate_in_browser() {
    let result = validate("Booking", r#"{"guest_email": "test@example.com", "rooms": 2}"#);
    assert!(result.is_null()); // No errors
}
```

### E2E Tests
```typescript
// Playwright test
test('validates booking form', async ({ page }) => {
  await page.fill('[name=guest_email]', 'invalid');
  await page.blur('[name=guest_email]');

  await expect(page.locator('.error')).toContainText('Invalid email');
});
```

## Migration Path

For existing projects using Zod codegen:

```typescript
// Before: Generated Zod schemas
import { bookingSchema } from './generated/schemas';
const result = bookingSchema.safeParse(data);

// After: WASM validation (drop-in replacement)
import { validate } from '@domainstack/wasm';
const result = validate('Booking', data);

// Same error structure — UI code unchanged!
```

## Open Questions

1. **Registry initialization** — Explicit `init()` or auto-init on first validate?
2. **Streaming validation** — Validate partial objects as user types?
3. **Schema export** — Also expose OpenAPI schemas from WASM for client-side form generation?
4. **Offline support** — Service worker caching strategy for WASM module?

## Success Metrics

- Bundle size < 50KB gzipped (for 10 types)
- Validation latency < 1ms for typical objects
- Zero drift: Server and client produce byte-identical error JSON
- Adoption: 50% of domainstack users enable WASM within 6 months

## References

- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [inventory crate](https://docs.rs/inventory) — Compile-time plugin registration
- [serde-wasm-bindgen](https://docs.rs/serde-wasm-bindgen) — Efficient JS ↔ Rust serialization
