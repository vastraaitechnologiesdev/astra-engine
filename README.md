# astra-engine

> **Independent Open Source Web Rendering Engine**  
> An independent, Android-first browser rendering engine developed by **V Astra AI Technologies**.

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](Cargo.toml)

---

## Overview

**Astra Engine** is a modern, standalone browser engine engineered from the ground up for high-performance and lightweight mobile rendering, targeting **Android First**.

Unlike conventional solutions, Astra Engine is completely independent:
- **No Chromium or Blink**
- **No Gecko or Servo**
- **No Android System WebView**
- **No third-party HTML parsers or DOM libraries**

All core rendering engine subsystems—HTML tokenizer, parser, DOM tree, CSS engine, layout calculation, and GPU rendering—are designed and implemented independently.

---

## Project Architecture

Astra Engine is organized as a modular Rust workspace:

```
astra-engine/
├── Cargo.toml                  # Workspace manifest (Rust 2024 edition, resolver 3)
├── crates/
│   ├── astra-dom/              # Arena-based DOM tree and node storage
│   └── astra-html/             # Custom HTML tokenizer and DOM tree constructor
├── tests/                      # Workspace integration and milestone test suite
├── docs/                       # Architectural specifications and design notes
├── README.md                   # Project documentation
└── LICENSE                     # Apache License 2.0
```

### Core Crates

- **`astra-dom`**:
  - Memory-efficient arena allocation using stable `NodeId` references.
  - Zero reference-counting memory leaks or cyclic graphs.
  - Complete DOM node abstractions: `Document`, `Element`, `Text`, `Comment`, and `DocumentType`.
  - Comprehensive tree mutation and traversal APIs (`children`, `element_children`, `ancestors`, `descendants`).
- **`astra-html`**:
  - Custom streaming tokenizer for HTML markup, attributes (quoted, unquoted, boolean), comments, and DOCTYPE declarations.
  - Resilient HTML parser building DOM trees from token streams.
  - Native support for standard HTML void elements (`img`, `br`, `meta`, `input`, `hr`, etc.).
  - Robust recovery for malformed markup (unclosed tags, mismatched tags, stray end tags).
  - Built-in HTML character reference and entity decoding (`&amp;`, `&lt;`, `&gt;`, `&#...;`).

---

## Current Development Status

- **Version**: `0.0.1 (Milestone 1)`
- **Completed**:
  - [x] Shared Rust 2024 workspace initialization with resolver 3.
  - [x] Arena-based DOM implementation (`astra-dom`).
  - [x] Custom HTML tokenizer supporting tags, attributes, comments, and DOCTYPEs (`astra-html`).
  - [x] Custom HTML tree construction parser with void elements and recovery (`astra-html`).
  - [x] Milestone verification test parsing the core HTML5 document.
  - [x] Full automated test suite (19 tests covering DOM operations, tokenization, tree generation, and error recovery).
- **Upcoming Milestones**:
  - CSS tokenizer, parser, and CSSOM.
  - Style resolution and cascade engine.
  - Layout tree computation (flow layout, box model).
  - Android SDK JNI bindings and native surface integration.
  - QuickJS JavaScript runtime integration.

---

## Build Instructions

### Prerequisites

- Rust 1.85+ (Rust 2024 edition support)
- Cargo

### Compilation

Check all crates in the workspace:

```bash
cargo check --workspace
```

Build debug binaries and libraries:

```bash
cargo build --workspace
```

Build optimized release artifacts:

```bash
cargo build --workspace --release
```

---

## Testing Instructions

Run all unit and integration tests across the workspace:

```bash
cargo test --workspace
```

Run code formatting checks:

```bash
cargo fmt --all -- --check
```

### Running the Milestone Test

To execute the Core v0.0.1 milestone verification test specifically:

```bash
cargo test --test milestone -- --nocapture
```

The milestone test verifies parsing of:
```html
<!DOCTYPE html>
<html>
    <head>
        <title>Astra Engine</title>
    </head>
    <body>
        <h1>Hello Astra!</h1>
        <p>Our own browser engine.</p>
    </body>
</html>
```
and validates the complete hierarchical DOM tree and text contents.

---

## License

Astra Engine is licensed under the **Apache License, Version 2.0**.
See the [LICENSE](LICENSE) file for complete terms and copyright notices.
