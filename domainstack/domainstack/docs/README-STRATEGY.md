# README Strategy

Documentation hierarchy for domainstack project.

## Current Structure

```
domainstack/
├── README.md                         ← GitHub landing page (PRIMARY)
├── docs/
│   ├── api-guide.md                  ← Complete API documentation
│   ├── rules.md                      ← Rules reference
│   ├── CONCEPT.md                    ← Design philosophy
│   ├── REVIEW.md                     ← Architecture review
│   └── V0.*.md                       ← Implementation plans
└── domainstack/
    ├── README.md                     ← Workspace overview (for crates.io)
    └── domainstack/
        └── README.md                 ← Core crate docs (for crates.io)
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
- Workspace structure (4 crates)
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

## Duplication Strategy

### What to Duplicate

**Installation section** - Different in each README:
- Root: Shows all 3 crates together
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

- Architecture diagrams (only in root)
- Planning docs (only in docs/)
- Detailed API guide (only in docs/api-guide.md)
- Rules reference (only in docs/rules.md)

## Links Strategy

Each README links to others:

**Root README** links to:
- Workspace README for technical docs
- docs/api-guide.md for complete API
- docs/rules.md for rules reference
- docs/CONCEPT.md for philosophy

**Workspace README** links to:
- Root README (back to home)
- Core crate README for deep dive
- docs/ for guides

**Core README** links to:
- Workspace README (up one level)
- docs.rs for generated API docs

## When to Update

| Change | Root | Workspace | Core |
|--------|------|-----------|------|
| New feature | Yes | Yes | Maybe |
| New crate | Yes | Yes | [x] No |
| API change | [x] No | Yes | Yes |
| New rule | [x] No | Yes | Yes |
| Example | Maybe | Yes | Maybe |

## Target Audiences

1. **GitHub visitors** → Root README → "Wow, this is cool!"
2. **Cargo users** → Workspace README → "How do I use this?"
3. **API explorers** → Core README + docs.rs → "How does this work internally?"
4. **Deep learners** → docs/api-guide.md, rules.md → "I want to master this"

## Maintenance

Keep root README **concise and compelling** - it's the first impression.  
Keep workspace README **complete but organized** - it's the main docs.  
Keep core README **focused on primitives** - it's the foundation.

The strategy accepts some duplication for the benefit of having appropriate content at each level.
