# Astra Engine Architecture (Core v0.0.1)

Astra Engine is an independent, Android-first browser rendering engine built from scratch in Rust by V Astra AI Technologies.

## Design Philosophy

- **Zero Third-Party Engine Code**: Completely independent from Chromium, Gecko, WebKit, or Android System WebView.
- **Custom HTML & DOM Stacks**: Built from foundational standards without third-party HTML parsing or DOM crates.
- **Cache-Friendly & Cycle-Free**: Arena-based DOM storage using indices (`NodeId`) rather than reference-counted pointer webs (`Rc<RefCell<Node>>`).
- **Memory Safety & Performance**: Pure Rust core with strict `#![forbid(unsafe_code)]` guarantees.

---

## Workspace Structure

```
astra-engine/
├── Cargo.toml                  # Workspace manifest (Rust 2024, resolver 3)
├── LICENSE                     # Apache License 2.0
├── README.md                   # Project overview and quick start
├── docs/
│   └── architecture.md         # Detailed architectural documentation
├── crates/
│   ├── astra-dom/              # Arena-based DOM tree and node management
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── id.rs           # Stable NodeId representation
│   │   │   ├── node.rs         # Node representation and metadata
│   │   │   ├── data.rs         # Element, Text, Comment, DocumentType, NodeData
│   │   │   ├── document.rs     # Document arena and hierarchy operations
│   │   │   ├── iter.rs         # Traversal iterators (Children, Elements, Ancestors, Descendants)
│   │   │   └── error.rs        # DomError validation and hierarchy errors
│   │   └── tests/
│   │       └── dom_tests.rs
│   │
│   └── astra-html/             # Custom HTML tokenizer and parser
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── token.rs        # Token definitions (StartTag, EndTag, Doctype, etc.)
│       │   ├── tokenizer.rs    # Streaming HTML tokenizer
│       │   ├── parser.rs       # DOM-constructing parser with recovery
│       │   └── entity.rs       # Named and numeric entity decoder
│
└── tests/                      # End-to-end and integration tests
    ├── Cargo.toml
    ├── milestone.rs            # Core v0.0.1 milestone verification test
    └── html_tests.rs           # Automated test suite
```

---

## Component Deep Dive

### 1. `astra-dom`

`astra-dom` provides the foundational DOM data structures:

- **Arena Storage (`Document`)**:
  All nodes are stored in a contiguous `Vec<Node>` inside `Document`. Nodes are identified by a 64-bit copyable `NodeId`.
- **Node Linkages**:
  Nodes maintain structural indices:
  - `parent: Option<NodeId>`
  - `first_child: Option<NodeId>`
  - `last_child: Option<NodeId>`
  - `previous_sibling: Option<NodeId>`
  - `next_sibling: Option<NodeId>`
- **Cycle Prevention**:
  `append_child` and `insert_before` inspect the ancestor chain before modifying structural links. Any attempt to introduce an ancestor as a descendant returns `DomError::CycleDetected`.
- **Traversals**:
  - `children(parent)`: Iterates direct children.
  - `element_children(parent)`: Iterates only element children.
  - `ancestors(node)`: Walks up to the document root.
  - `descendants(node)`: Pre-order depth-first traversal of all descendants.

### 2. `astra-html`

`astra-html` implements our proprietary HTML tokenizer and tree constructor:

- **Tokenizer (`Tokenizer`)**:
  - Streams characters and identifies DOCTYPEs, start tags, end tags, comments, text, and self-closing tags.
  - Parses attributes: double-quoted (`attr="val"`), single-quoted (`attr='val'`), unquoted (`attr=val`), and boolean (`disabled`).
  - Decodes character references: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`, `&copy;`, `&#NNN;`, `&#xHHH;`.
- **Parser (`Parser`)**:
  - Maintains an open element stack `stack: Vec<NodeId>`.
  - Dispatches tokens to create and attach DOM nodes.
  - Treats standard HTML void elements (`<img>`, `<br>`, `<meta>`, `<input>`, `<hr>`, etc.) as self-closing without stacking them.
  - Coalesces adjacent text tokens to avoid DOM node fragmentation.
  - Recovers gracefully from malformed HTML:
    - Auto-closes unclosed tags at parent tag closure or at EOF.
    - Gracefully discards stray, unattached closing tags without panicking.
