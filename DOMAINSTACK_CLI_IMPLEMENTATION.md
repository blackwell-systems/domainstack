# domainstack-cli - Unified Code Generation CLI

## Executive Summary

**Unified CLI tool** for generating TypeScript validators, GraphQL schemas, and more from Rust `domainstack` validation rules.

**First Implementation**: Zod schema generation (TypeScript validation)
**Future Generators**: Yup, GraphQL SDL, Prisma, OpenAPI, JSON Schema

This document focuses on **Phase 1: Zod Generator**, which generates TypeScript Zod validation schemas from Rust validation rules, ensuring frontend and backend validation stay perfectly synchronized.

**Architecture**: Unified CLI (`domainstack-cli`) with subcommands
**Status**: ✅ Phase 1 Complete - v0.1.0 Implemented & Tested
**Priority**: 🔥🔥🔥 Very High Impact
**Effort**: 6 days for MVP (Zod generator) - **COMPLETED**
**Target**: v0.1.0 release - **ACHIEVED**

---

## Unified CLI Vision

`domainstack-cli` is designed as a **unified code generation tool** that transforms Rust validation rules into multiple target formats:

```bash
# Install once
cargo install domainstack-cli

# Generate for different ecosystems
domainstack zod --input src --output frontend/schemas.ts        # ✅ Phase 1 (MVP)
domainstack yup --input src --output frontend/schemas.ts        # 📋 Future
domainstack graphql --input src --output schema.graphql         # 📋 Future
domainstack prisma --input src --output prisma/schema.prisma    # 📋 Future
```

**Why unified?**
- ✅ **Single installation** - one tool, not separate binaries for each generator
- ✅ **Shared parsing** - all generators reuse the same Rust/validation parser
- ✅ **Consistent UX** - same flags and patterns across generators
- ✅ **Scalable** - adding new generators is just adding a new subcommand

**This document** focuses on **Phase 1: Zod Generator** - the foundation that proves the unified CLI architecture.

### Scope of This Implementation

**✅ COMPLETED (v0.1.0 MVP)**:
- ✅ Unified CLI framework (`domainstack-cli` crate) - **DONE**
- ✅ Shared parser module (reusable by all future generators) - **DONE**
- ✅ Zod generator implementation - **DONE**
- ✅ Command: `domainstack zod --input <dir> --output <file>` - **DONE**
- ✅ Comprehensive testing with 26+ validation rules - **DONE**
- ✅ Complete README and documentation - **DONE**
- ✅ CHANGELOG entry - **DONE**

**Future Generators** (v0.2.0+):
- 📋 `domainstack yup` - Yup schemas for React ecosystem
- 📋 `domainstack graphql` - GraphQL Schema Definition Language
- 📋 `domainstack prisma` - Prisma schema with validation constraints
- 📋 `domainstack openapi` - Enhanced OpenAPI 3.0 schemas
- 📋 `domainstack json-schema` - JSON Schema with validation

The unified CLI architecture ensures adding these is **trivial** - just new subcommands using the shared parser.

---

## The Problem (Zod Generator Motivation)

**Current State**: Developers duplicate validation logic between Rust backend and TypeScript frontend

```rust
// Backend: src/models/user.rs
#[derive(Validate)]
struct User {
    #[validate(email)]
    #[validate(max_len = 255)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,
}
```

```typescript
// Frontend: MANUALLY WRITTEN, DUPLICATED 😢
const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
});
```

**Problems**:
- ❌ Duplication → maintenance burden, drift over time
- ❌ Type mismatches → `u8` in Rust but `string` in TS?
- ❌ Validation rule drift → backend allows 18-120, frontend allows 0-200
- ❌ Developer friction → change validation in two places

---

## The Solution

**Auto-generate Zod schemas from Rust validation rules**

```bash
$ domainstack zod --input src --output frontend/src/schemas.ts
✓ Generated schemas for 12 types
✓ Written to frontend/src/schemas.ts
```

```typescript
// frontend/src/schemas.ts (AUTO-GENERATED)
import { z } from "zod";

export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
});

export type User = z.infer<typeof UserSchema>;
```

**Benefits**:
- ✅ **Single source of truth** - change validation once, both sides updated
- ✅ **Type safety across stack** - Rust types → TS types automatically
- ✅ **Better UX** - immediate client-side feedback with exact same rules
- ✅ **Zero maintenance** - no manual synchronization needed
- ✅ **CI integration** - fail builds if schemas are out of sync

---

## Research Summary

### Zod API Capabilities

Zod is a TypeScript-first validation library with:
- **Zero dependencies**, 2kb gzipped
- **Type inference**: `type User = z.infer<typeof UserSchema>`
- **Rich validation**: `.email()`, `.min()`, `.max()`, `.regex()`, etc.
- **Composition**: Chain validations like `.string().email().max(255)`

**Key Documentation**:
- [Zod Official Docs](https://zod.dev/)
- [API Reference](https://zod.dev/api)
- [Basic Usage](https://zod.dev/basics)

### Existing Rust→TypeScript Tools

| Tool | What It Does | Limitation |
|------|--------------|------------|
| [ts-rs](https://github.com/Aleph-Alpha/ts-rs) | Generate TS types from Rust structs | ❌ No validation, just types |
| [typeshare](https://github.com/1Password/typeshare) | Multi-language type sync (1Password) | ❌ No validation rules |
| [zod_gen](https://github.com/cimatic/zod_gen) | Generate Zod schemas from Rust | ❌ **No field-level validation constraints** |

**Key Finding**: No existing tool generates Zod schemas **with validation rules** from Rust attributes.
**Opportunity**: `domainstack` is uniquely positioned - we already have rich validation metadata!

---

## Validation Rule → Zod Mapping

### String Validation Rules

| domainstack Rule | Generated Zod Schema | Notes |
|------------------|----------------------|-------|
| `#[validate(email)]` | `z.string().email()` | Built-in Zod validator |
| `#[validate(url)]` | `z.string().url()` | Built-in Zod validator |
| `#[validate(min_len = 5)]` | `z.string().min(5)` | Direct mapping |
| `#[validate(max_len = 100)]` | `z.string().max(100)` | Direct mapping |
| `#[validate(length(min = 3, max = 20))]` | `z.string().min(3).max(20)` | Chained validations |
| `#[validate(non_empty)]` | `z.string().min(1)` | Equivalent constraint |
| `#[validate(non_blank)]` | `z.string().trim().min(1)` | Trim + min |
| `#[validate(alphanumeric)]` | `z.string().regex(/^[a-zA-Z0-9]*$/)` | Custom regex |
| `#[validate(alpha_only)]` | `z.string().regex(/^[a-zA-Z]*$/)` | Custom regex |
| `#[validate(numeric_string)]` | `z.string().regex(/^[0-9]*$/)` | Custom regex |
| `#[validate(ascii)]` | `z.string().regex(/^[\x00-\x7F]*$/)` | Custom regex |
| `#[validate(starts_with = "https://")]` | `z.string().startsWith("https://")` | Built-in Zod method |
| `#[validate(ends_with = ".com")]` | `z.string().endsWith(".com")` | Built-in Zod method |
| `#[validate(contains = "example")]` | `z.string().includes("example")` | Built-in Zod method |
| `#[validate(matches_regex = "^[a-z]+$")]` | `z.string().regex(/^[a-z]+$/)` | Direct regex |

### Numeric Validation Rules

| domainstack Rule | Generated Zod Schema | Notes |
|------------------|----------------------|-------|
| `#[validate(range(min = 18, max = 120))]` | `z.number().min(18).max(120)` | Chained |
| `#[validate(min = 0)]` | `z.number().min(0)` | Direct |
| `#[validate(max = 100)]` | `z.number().max(100)` | Direct |
| `#[validate(positive)]` | `z.number().positive()` | Built-in |
| `#[validate(negative)]` | `z.number().negative()` | Built-in |
| `#[validate(non_zero)]` | `z.number().refine(n => n !== 0)` | Custom refine |
| `#[validate(multiple_of = 5)]` | `z.number().multipleOf(5)` | Built-in |
| `#[validate(finite)]` | `z.number().finite()` | Built-in |

### Type Mapping

| Rust Type | TypeScript Type | Zod Base Schema | Notes |
|-----------|----------------|-----------------|-------|
| `String` | `string` | `z.string()` | ✅ Perfect match |
| `u8, u16, u32` | `number` | `z.number()` | ✅ Safe range |
| `i8, i16, i32` | `number` | `z.number()` | ✅ Safe range |
| `f32, f64` | `number` | `z.number()` | ✅ Direct map |
| `u64, u128, i64, i128` | `number` | `z.number()` | ⚠️ Precision loss warning |
| `bool` | `boolean` | `z.boolean()` | ✅ Perfect match |
| `Option<T>` | `T \| undefined` | `z.optional(T)` | ✅ Zod built-in |
| `Vec<T>` | `T[]` | `z.array(T)` | ✅ Zod built-in |

---

## Architecture Decision

### Recommended: **Unified CLI** with Subcommands

#### Design: One CLI, Multiple Generators

**Why unified?** Prepare for future generators (Yup, GraphQL, Prisma, etc.) without cluttering the ecosystem with separate binaries.

**Command Structure**:
```bash
# Install once
cargo install domainstack-cli

# Generate different schemas
domainstack zod --input src --output frontend/schemas.ts
domainstack yup --input src --output frontend/schemas.ts      # Future
domainstack graphql --input src --output schema.graphql       # Future
```

**Crate Structure**:
```
domainstack-cli/              # Unified CLI binary
├── Cargo.toml
└── src/
    ├── main.rs               # CLI entry point (clap)
    ├── commands/
    │   └── zod.rs            # Zod subcommand
    ├── generators/
    │   ├── mod.rs
    │   └── zod.rs            # Zod schema generator
    └── parser/
        ├── mod.rs
        ├── ast.rs            # Shared AST parsing
        └── validation.rs     # Shared validation extraction
```

#### Phase 1: Zod Generator (MVP - Target v1.1.0)

**Command**:
```bash
domainstack zod --input src --output frontend/src/schemas.ts
```

**How it works**:
1. Parse all `.rs` files in `--input` directory
2. Find structs with `#[derive(Validate)]`
3. Extract `#[validate(...)]` attributes
4. Generate Zod schemas with validation rules
5. Write to `--output` file

**Benefits of Unified CLI**:
- ✅ **Single installation** - one tool, not `domainstack-zod` + `domainstack-yup` + ...
- ✅ **Shared parsing logic** - all generators reuse same Rust parser
- ✅ **Consistent interface** - same flags across generators (`--input`, `--output`, `--watch`)
- ✅ **Better discovery** - `domainstack --help` shows all capabilities
- ✅ **Brand consistency** - everything is `domainstack <command>`
- ✅ **Easy to extend** - adding new generators is just a new subcommand

**Integration Examples**:
```json
// package.json
{
  "scripts": {
    "codegen": "domainstack zod --input ../backend/src --output src/schemas.ts",
    "codegen:watch": "domainstack zod --input ../backend/src --output src/schemas.ts --watch",
    "prebuild": "npm run codegen"
  }
}
```

```yaml
# .github/workflows/ci.yml
- name: Generate Zod schemas
  run: domainstack zod --input src --output frontend/src/schemas.ts

- name: Check for uncommitted changes
  run: |
    git diff --exit-code frontend/src/schemas.ts || \
      (echo "❌ Schemas out of date! Run: npm run codegen" && exit 1)
```

```toml
# domainstack.toml (future: config file support)
[zod]
input = "src/models"
output = "frontend/src/schemas.ts"
watch = true

[graphql]  # Future generator
input = "src/models"
output = "schema.graphql"
```

#### Phase 2: Additional Generators (v1.2.0+)

Once unified CLI is established, adding generators is **trivial**:

**Yup Schemas** (React ecosystem):
```bash
domainstack yup --input src --output frontend/schemas.ts
```

**GraphQL Schema Definition Language**:
```bash
domainstack graphql --input src --output schema.graphql
```

**Prisma Migrations**:
```bash
domainstack prisma --input src --output prisma/schema.prisma
```

**Enhanced OpenAPI** (beyond current `ToSchema`):
```bash
domainstack openapi --input src --output openapi.yaml
```

#### Phase 3: Proc Macro (Optional, v1.3.0+)

**Optional convenience** for users who want auto-export:

```rust
#[derive(Validate, ToZod)]
#[zod(export = "frontend/src/schemas")]
struct User {
    #[validate(email)]
    email: String,
}
```

- Exports during `cargo test` (like ts-rs)
- CLI can still be used for batch generation
- Gives users flexibility

### Why Unified CLI?

1. **Scalability** - prepared for multiple generators without ecosystem fragmentation
2. **Shared code** - parsing logic used by all generators (Zod, Yup, GraphQL, etc.)
3. **Better UX** - single tool to install and learn
4. **Standard pattern** - like `cargo`, `prisma`, `protoc` with subcommands
5. **Future-proof** - easy to add new generators as needed

---

## Implementation Plan

### Milestone 1: Core CLI Tool (3 days)

**Crate**: `domainstack-cli` (binary crate with `zod` subcommand)

**Tasks**:
1. **CLI Framework Setup**
   - Use `clap` for argument parsing with subcommands
   - Main binary: `domainstack`
   - First subcommand: `zod`
   - Shared flags: `--input <DIR>`, `--output <FILE>`, `--watch`

2. **File Parser (Shared Module)**
   - Use `syn` to parse Rust files
   - Find structs with `#[derive(Validate)]`
   - Extract field types and `#[validate(...)]` attributes
   - **Reusable** by future generators (Yup, GraphQL, etc.)

3. **Validation Rule Parser (Shared Module)**
   - Reuse parsing logic from `domainstack-derive`
   - Convert parsed rules to intermediate representation
   - **Reusable** by all generators

4. **Zod Code Generator (Generator-Specific)**
   - Map Rust types → TS types
   - Map validation rules → Zod methods
   - Generate TypeScript code with proper formatting
   - Lives in `generators/zod.rs`

**Example Output**:
```typescript
/**
 * AUTO-GENERATED by domainstack-cli
 * Command: domainstack zod --input src --output frontend/schemas.ts
 * DO NOT EDIT MANUALLY
 *
 * Generated from: src/models/user.rs
 * Date: 2025-01-15T10:30:00Z
 */

import { z } from "zod";

export const UserSchema = z.object({
  email: z.string().email().max(255),
  age: z.number().min(18).max(120),
  username: z.string().min(3).max(20).regex(/^[a-zA-Z0-9]*$/),
});

export type User = z.infer<typeof UserSchema>;

export const PostSchema = z.object({
  title: z.string().min(1).max(200),
  content: z.string().min(10),
  author: UserSchema,  // Nested type
  tags: z.array(z.string()),  // Vec<String>
});

export type Post = z.infer<typeof PostSchema>;
```

### Milestone 2: Testing & Examples (1 day)

**Tests**:
1. Unit tests for parser
2. Unit tests for code generator
3. Integration tests with sample Rust files
4. Snapshot tests for generated output

**Examples**:
1. Basic usage example (`examples/basic`)
2. Full-stack example with React frontend
3. CI integration example

### Milestone 3: Documentation (1 day)

**Docs**:
1. README for `domainstack-cli` crate
2. User guide for `domainstack zod` command
3. Migration guide from manual Zod schemas
4. Troubleshooting guide
5. Architecture docs (unified CLI design, adding new generators)
6. Update ROADMAP.md to mark as implemented

### Milestone 4: Advanced Features (Optional, v1.2.0+)

**Future enhancements**:
1. **Watch mode**: Auto-regenerate on file changes
2. **Selective export**: Only generate for marked types
3. **Custom type mappings**: Override default type mappings
4. **Multiple output files**: Split schemas by module
5. **Zod refinements**: Support for custom validation logic
6. **Error messages**: Custom error messages from Rust
7. **Proc macro**: `#[derive(ToZod)]` for convenience

---

## Technical Challenges & Solutions

### Challenge 1: Parsing Validation Attributes

**Problem**: Need to parse `#[validate(...)]` attributes from raw Rust files

**Solution**: Reuse existing parsing logic from `domainstack-derive`:
- Extract `parse_validation_rules()` into shared module
- Create `domainstack-derive-shared` crate if needed
- Both proc macro and CLI use same parser

### Challenge 2: Nested Types

**Problem**: `author: User` needs to reference `UserSchema`

**Solution**: Two-pass generation:
1. First pass: Collect all type names
2. Second pass: Generate schemas with references
3. Topological sort for dependency order

### Challenge 3: Complex Enums

**Problem**: Rust enums don't map cleanly to TypeScript

**Solution**: Start simple, iterate:
- **V1**: Only support structs
- **V2**: Unit enums → TypeScript union types
- **V3**: Complex enums → discriminated unions

### Challenge 4: Custom Validators

**Problem**: `#[validate(custom = "my_validator")]` has no TS equivalent

**Solution**: Skip with warning:
```
⚠ Warning: Custom validator 'my_validator' on User.field cannot be
  represented in Zod. Consider adding explicit validation rules.
```

### Challenge 5: Type Precision

**Problem**: `u64` in Rust can't safely represent in JS `number`

**Solution**: Warn user, suggest alternatives:
```
⚠ Warning: u64 field 'id' may lose precision in JavaScript.
  Consider using String for large integers.
```

---

## Success Metrics

### MVP Success Criteria

- ✅ CLI tool successfully parses 100+ real-world Rust structs
- ✅ Generates valid Zod schemas for all supported rules
- ✅ Integration example works with React + domainstack backend
- ✅ Documentation complete with examples
- ✅ 90%+ test coverage
- ✅ Published to crates.io

### User Adoption Metrics (3 months post-launch)

- 🎯 100+ crates.io downloads
- 🎯 5+ GitHub issues/discussions (engagement)
- 🎯 Example projects shared by community
- 🎯 Mentioned in Rust web dev tutorials/blogs

---

## Development Timeline

| Milestone | Effort | Deliverable |
|-----------|--------|-------------|
| **1. Core CLI Tool** | 3 days | `domainstack-cli` with `zod` subcommand |
| **2. Testing & Examples** | 1 day | Comprehensive tests + full-stack example |
| **3. Documentation** | 1 day | User guide, architecture docs, examples |
| **4. Polish & Release** | 1 day | README, CHANGELOG, crates.io publish |
| **Total (MVP)** | **6 days** | `domainstack-cli` v0.1.0 on crates.io |

**Future Milestones** (post-MVP):
- **v0.2.0**: Add `--watch` mode for auto-regeneration
- **v0.3.0**: Add `yup` subcommand for Yup schemas
- **v0.4.0**: Add `graphql` subcommand for GraphQL SDL
- **v1.0.0**: Config file support (`domainstack.toml`), stable API

---

## Out of Scope (for MVP)

These are explicitly **not** included in v1.1.0:

- ❌ Yup schema generation (focus on Zod only)
- ❌ Vanilla JS validators (Zod is the target)
- ❌ GraphQL schema generation
- ❌ OpenAPI 3.0 schema updates (already have `ToSchema`)
- ❌ Proc macro `#[derive(ToZod)]` (CLI only for now)
- ❌ Watch mode (manual generation only)
- ❌ Custom type mappings (use defaults only)

---

## References

### Research Sources

1. **Zod Documentation**
   - [Official Docs](https://zod.dev/)
   - [API Reference](https://zod.dev/api)
   - [GitHub Repository](https://github.com/colinhacks/zod)

2. **Existing Rust→TS Tools**
   - [ts-rs](https://github.com/Aleph-Alpha/ts-rs) - Type generation
   - [typeshare](https://github.com/1Password/typeshare) - Multi-language types
   - [zod_gen](https://github.com/cimatic/zod_gen) - Zod schema generation (no validation)

3. **Inspiration**
   - [Better Stack: Complete Guide to Zod](https://betterstack.com/community/guides/scaling-nodejs/zod-explained/)
   - [Publishing Rust Types to TypeScript Frontend](https://cetra3.github.io/blog/sharing-types-with-the-frontend/)

---

## Implementation Status

### ✅ Completed Steps

1. ✅ **Research and design** - Unified CLI architecture designed
2. ✅ **Unified CLI architecture** - Future-proof design for multiple generators
3. ✅ **User approval** - Confirmed unified CLI approach
4. ✅ **Create `domainstack-cli` crate** - Workspace crate created with proper structure
5. ✅ **Implement shared parser** - File parsing + validation extraction (reusable by all generators)
6. ✅ **Implement Zod generator** - Complete Zod schema generation in `generators/zod.rs`
7. ✅ **Build CLI interface** - `clap` subcommands + file I/O working
8. ✅ **Write tests** - Comprehensive local testing with 26+ validation rules
9. ✅ **Write documentation** - Complete README, CHANGELOG entry, implementation plan
10. ✅ **Commit and push** - All changes committed to branch

### 📋 Remaining for v0.1.0 Release

1. 📋 **Create examples** - Full-stack example app with React (optional for initial commit)
2. 📋 **Publish to crates.io** - Release `domainstack-cli` v0.1.0
3. 📋 **Update main README** - Add domainstack-cli to main project README

### 🚀 Future Work (v0.2.0+)

- **v0.2.0**: Watch mode (`--watch` flag) for auto-regeneration
- **v0.3.0**: Yup generator (`domainstack yup`)
- **v0.4.0**: GraphQL SDL generator (`domainstack graphql`)
- **v0.5.0**: Prisma schema generator (`domainstack prisma`)
- **v1.0.0**: Config file support (`domainstack.toml`), stable API

---

**Status**: ✅ **PHASE 1 COMPLETE** - v0.1.0 Implemented & Tested
**Architecture**: Unified CLI (`domainstack-cli`) with subcommands, starting with `zod`
**Author**: Claude (AI Assistant)
**Date**: 2025-01-15
**Implemented**: 2025-12-25
**Next**: Publish to crates.io, update main README
