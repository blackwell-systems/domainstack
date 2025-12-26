# Running Examples

Examples are included in the repository (not published to crates.io). Clone the repo to try them:

```bash
# Clone the repository
git clone https://github.com/blackwell-systems/domainstack.git
cd domainstack/domainstack

# Manual validation examples
cargo run --example email_primitive --features regex
cargo run --example booking_aggregate --features regex
cargo run --example age_primitive

# Derive macro examples
cargo run --example v2_basic
cargo run --example v2_nested
cargo run --example v2_collections
cargo run --example v2_custom

# HTTP integration examples
cargo run --example v3_error_envelope_basic
cargo run --example v3_error_envelope_nested

# Builder customization examples
cargo run --example v4_builder_customization

# Cross-field validation examples
cargo run --example v5_cross_field_validation

# Async validation examples
cargo run --example async_validation --features async
cargo run --example async_sqlite --features async

# Phantom types examples
cargo run --example phantom_types --features regex

# OpenAPI schema generation examples
cd domainstack-schema
cargo run --example user_api
cargo run --example v08_features

# Framework examples
cd examples-axum && cargo run    # Axum booking service
cd examples-actix && cargo run   # Actix-web booking service
cd examples-rocket && cargo run  # Rocket booking service
```

## Example Categories

### Manual Validation Examples
Located in `domainstack/domainstack/examples/`:
- `email_primitive.rs` - Email validation with newtype pattern
- `booking_aggregate.rs` - Complex booking domain with nested validation
- `age_primitive.rs` - Age validation with range rules

### Derive Macro Examples
Located in `domainstack/domainstack-examples/`:
- `v2_basic.rs` - Basic `#[derive(Validate)]` usage
- `v2_nested.rs` - Nested struct validation
- `v2_collections.rs` - Collection validation with array indices
- `v2_custom.rs` - Custom validation rules

### HTTP Integration Examples
- `v3_error_envelope_basic.rs` - Error envelope pattern basics
- `v3_error_envelope_nested.rs` - Nested errors with HTTP responses

### Advanced Examples
- `v4_builder_customization.rs` - Customize error codes and messages
- `v5_cross_field_validation.rs` - Cross-field business rules
- `async_validation.rs` - Database and API validation
- `async_sqlite.rs` - SQLite uniqueness checks
- `phantom_types.rs` - Type-state validation with phantom types

### Framework Examples
Full working services:
- `examples-axum/` - Axum booking service with DomainJson
- `examples-actix/` - Actix-web booking service
- `examples-rocket/` - Rocket booking service

### OpenAPI Schema Examples
Located in `domainstack/domainstack-schema/examples/`:
- `user_api.rs` - Basic schema generation
- `v08_features.rs` - Advanced schema features
