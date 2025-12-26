# README Strategy

Documentation hierarchy for domainstack project.

## Current Structure

```
domainstack/
├── README.md                         ← GitHub landing page (PRIMARY)
├── CHANGELOG.md                      ← Release history
├── CONTRIBUTING.md                   ← Contribution guidelines
├── ROADMAP.md                        ← Future plans
├── README-STRATEGY.md                ← This file (docs organization)
└── domainstack/
    ├── README.md                     ← Workspace overview (for crates.io)
    ├── domainstack/
    │   ├── README.md                 ← Core crate docs (for crates.io)
    │   └── docs/
    │       ├── CORE_CONCEPTS.md      ← Foundation principles
    │       ├── DERIVE_MACRO.md       ← #[derive(Validate)] guide
    │       ├── MANUAL_VALIDATION.md  ← Custom validation
    │       ├── ERROR_HANDLING.md     ← ValidationError patterns
    │       ├── RULES.md              ← All 37 validation rules
    │       ├── HTTP_INTEGRATION.md   ← Axum/Actix/Rocket
    │       ├── OPENAPI_SCHEMA.md     ← Schema generation
    │       ├── WASM_VALIDATION.md    ← Browser validation
    │       ├── CLI_GUIDE.md          ← Code generation CLI
    │       ├── ADVANCED_PATTERNS.md  ← Complex patterns
    │       ├── architecture.md       ← Technical architecture
    │       └── INSTALLATION.md       ← All installation options
    ├── domainstack-derive/README.md  ← Derive crate docs
    ├── domainstack-schema/README.md  ← Schema crate docs
    ├── domainstack-wasm/README.md    ← WASM crate docs
    ├── domainstack-cli/README.md     ← CLI crate docs
    └── domainstack-*/README.md       ← Other crate docs
```

## Purpose of Each README

### 1. Root README (domainstack/README.md)
**Audience**: GitHub visitors, potential users
**Purpose**: Marketing and quick start
**Content**:
- Badges and branding
- Architecture diagram
- Feature highlights
- Compelling Quick Start with all features
- HTTP integration example (boilerplate reduction)
- Installation for all crates
- Links to detailed docs

**Strategy**: Make this the **best first impression** - show value immediately

### 2. Workspace README (domainstack/domainstack/README.md)
**Audience**: Rust developers, crates.io visitors
**Purpose**: Technical documentation for the workspace
**Content**:
- Workspace structure (9 crates)
- Detailed feature documentation
- All derive attributes
- Rule composition
- Error handling patterns
- Running examples and tests

**Strategy**: **Comprehensive technical reference** for developers using the crates

### 3. Core Crate README (domainstack/domainstack/domainstack/README.md)
**Audience**: docs.rs visitors, deep-dive developers
**Purpose**: Core library API documentation
**Content**:
- Core concepts (Validate trait, Rule<T>, ValidationError)
- Manual validation patterns
- Built-in rules
- Examples focused on core library only (no derive)

**Strategy**: **Detailed API docs** for the core validation primitives

## Detailed Guides (docs/ folder)

Each guide focuses on one topic in depth:

| Guide | Topic | Audience |
|-------|-------|----------|
| CORE_CONCEPTS.md | Foundation principles | All developers |
| DERIVE_MACRO.md | #[derive(Validate)] | Most users |
| RULES.md | All 37 validation rules | Reference |
| ERROR_HANDLING.md | ValidationError patterns | Error handling |
| HTTP_INTEGRATION.md | Axum/Actix/Rocket | Web developers |
| OPENAPI_SCHEMA.md | Schema generation | API developers |
| WASM_VALIDATION.md | Browser validation | Full-stack |
| CLI_GUIDE.md | TypeScript/Zod generation | Full-stack |
| ADVANCED_PATTERNS.md | Async, type-state, etc. | Advanced users |

## Duplication Strategy

### What to Duplicate

**Installation section** - Different in each README:
- Root: Shows all crates together
- Workspace: Shows feature combinations
- Core: Shows just the core crate

**Quick Start** - Different examples:
- Root: Impressive full-featured example
- Workspace: Multiple examples (derive, manual, HTTP)
- Core: Simple manual validation only

**Features list** - Progressively detailed:
- Root: High-level bullet points
- Workspace: Detailed with code examples
- Core: Technical implementation details

### What NOT to Duplicate

- Architecture diagrams (only in root and architecture.md)
- Detailed guides (only in docs/)
- Rules reference (only in docs/RULES.md)
- Roadmap items (only in ROADMAP.md)

## Links Strategy

Each README links to others:

**Root README** links to:
- Workspace README for technical docs
- docs/CORE_CONCEPTS.md for foundation
- docs/RULES.md for rules reference

**Workspace README** links to:
- Root README (back to home)
- Core crate README for deep dive
- docs/ for guides

**Core README** links to:
- Workspace README (up one level)
- docs.rs for generated API docs

## When to Update

| Change | Root | Workspace | Core | Guides |
|--------|------|-----------|------|--------|
| New feature | Yes | Yes | Maybe | Yes |
| New crate | Yes | Yes | No | Maybe |
| API change | No | Yes | Yes | Yes |
| New rule | No | Yes | Yes | RULES.md |
| Example | Maybe | Yes | Maybe | Relevant guide |

## Target Audiences

1. **GitHub visitors** → Root README → "Wow, this is cool!"
2. **Cargo users** → Workspace README → "How do I use this?"
3. **API explorers** → Core README + docs.rs → "How does this work internally?"
4. **Deep learners** → docs/ guides → "I want to master this"

## Maintenance

Keep root README **concise and compelling** - it's the first impression.
Keep workspace README **complete but organized** - it's the main docs.
Keep core README **focused on primitives** - it's the foundation.
Keep guides **focused and current** - one topic per file.

The strategy accepts some duplication for the benefit of having appropriate content at each level.
