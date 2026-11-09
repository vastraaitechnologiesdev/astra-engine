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
│   └── astra-html/             # Custom HTML tokenizer and tree builder
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
  - Document compatibility modes: `QuirksMode` (`NoQuirks`, `Quirks`, `LimitedQuirks`).
  - Comprehensive tree mutation and traversal APIs (`children`, `element_children`, `ancestors`, `descendants`).
- **`astra-html`**:
  - Custom streaming tokenizer supporting tags, attributes, comments, DOCTYPEs, and RCDATA/RAWTEXT state switches.
  - Full HTML5 tree construction state machine (`TreeBuilder`) supporting 16 insertion modes:
    - Document structure: `Initial`, `BeforeHtml`, `BeforeHead`, `InHead`, `AfterHead`, `InBody`, `Text`, `AfterBody`, `AfterAfterBody`.
    - Table foundation: `InTable`, `InTableText`, `InCaption`, `InColumnGroup`, `InTableBody`, `InRow`, `InCell`.
  - Automatic structure insertion for missing `<html>`, `<head>`, and `<body>`.
  - Basic HTML5 error recovery:
    - Paragraph auto-closing upon encountering block-level elements or nested paragraphs.
    - List item (`<li>`) and definition list (`<dt>`, `<dd>`) auto-closing.
    - Heading recovery (`<h1>`-`<h6>`).
    - Foster parenting for misnested table text and elements.
  - Built-in HTML character reference and entity decoding (`&amp;`, `&lt;`, `&gt;`, `&#...;`).

---

## Current Development Status

- **Version**: `0.0.1`
- **Completed**:
  - [x] Shared Rust 2024 workspace initialization with resolver 3.
  - [x] Arena-based DOM implementation (`astra-dom`).
  - [x] Custom HTML tokenizer supporting tags, attributes, comments, and DOCTYPEs (`astra-html`).
  - [x] **Milestone 2: HTML5 Tree Construction Foundation**:
    - [x] 16-mode insertion state machine (`TreeBuilder`).
    - [x] Automatic document structure insertion (`<html>`, `<head>`, `<body>`).
    - [x] Paragraph auto-closing, list item auto-closing, and heading recovery.
    - [x] Table parsing modes and foster parenting algorithm.
    - [x] Tokenizer RCDATA (`<title>`, `<textarea>`) and RAWTEXT (`<style>`, `<script>`) states.
    - [x] DOCTYPE quirks mode determination (`NoQuirks`, `Quirks`, `LimitedQuirks`).
    - [x] 36/36 automated unit and integration tests passing.
- **Upcoming Milestones**:
  - CSS tokenizer, parser, and CSSOM (Milestone 3).
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

### Running Milestone Tests

To execute the Milestone 1 verification test specifically:

```bash
cargo test --test milestone -- --nocapture
```

To execute the Milestone 2 tree construction test suite:

```bash
cargo test --test tree_construction -- --nocapture
```

---

## License

Astra Engine is licensed under the **Apache License, Version 2.0**.
See the [LICENSE](LICENSE) file for complete terms and copyright notices.
