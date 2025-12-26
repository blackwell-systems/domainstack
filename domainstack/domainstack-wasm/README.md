# domainstack-wasm

WASM browser validation for domainstack. Run the same validation logic in both browser and server.

## Quick Start

```bash
# Build WASM package
wasm-pack build --target web --release
```

```javascript
import init, { createValidator, validate } from '@domainstack/wasm';

// Initialize WASM module
await init();

// Create validator instance
const validator = createValidator();

// Validate data
const result = validator.validate('Booking', JSON.stringify({
  guest_email: 'invalid',
  rooms: 15
}));

if (!result.ok) {
  result.errors?.forEach(e => {
    console.log(`${e.path}: ${e.message}`);
  });
}
```

## Why WASM?

- **Zero drift** — Same Rust validation code runs in browser and server
- **Type safety** — Validation errors match server response format exactly
- **Small bundle** — ~60KB uncompressed, smaller gzipped

## Registering Types

Types must be registered before validation:

```rust
use domainstack::prelude::*;
use domainstack_wasm::register_type;
use serde::Deserialize;

#[derive(Debug, Validate, Deserialize)]
struct Booking {
    #[validate(email)]
    guest_email: String,

    #[validate(range(min = 1, max = 10))]
    rooms: u8,
}

// Register at app initialization
register_type::<Booking>("Booking");
```

## API

### Rust

| Function | Description |
|----------|-------------|
| `register_type::<T>(name)` | Register a type for validation |
| `validate(type_name, json)` | Validate JSON string |
| `validate_object(type_name, value)` | Validate JS object |
| `createValidator()` | Create validator instance |

### JavaScript/TypeScript

```typescript
interface ValidationResult {
  ok: boolean;
  errors?: Violation[];      // Validation failures
  error?: SystemError;       // System error (unknown type, parse failure)
}

interface Violation {
  path: string;
  code: string;
  message: string;
  meta?: Record<string, string>;
}

interface SystemError {
  code: 'unknown_type' | 'parse_error';
  message: string;
}
```

## Building

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build with wasm-pack
wasm-pack build --target web --release

# Output in pkg/
ls pkg/
```

## Documentation

- [Implementation Guide](../domainstack/docs/WASM_VALIDATION.md) — Architecture, API design, phases
- [Publishing Guide](../domainstack/docs/PUBLISHING.md) — Dual crates.io + npm workflow

## License

Apache-2.0
