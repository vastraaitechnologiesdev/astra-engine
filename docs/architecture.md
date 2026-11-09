# Astra Engine Architecture (Core v0.0.1 — Milestone 2)

Astra Engine is an independent, Android-first browser rendering engine built from scratch in Rust by V Astra AI Technologies.

## Design Philosophy

- **Zero Third-Party Engine Code**: Completely independent from Chromium, Blink, Gecko, WebKit, or Android System WebView.
- **Custom HTML & DOM Stacks**: Built from foundational standards without third-party HTML parsing or DOM crates (e.g. no html5ever).
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
│   │   │   ├── data.rs         # Element, Text, Comment, DocumentType, QuirksMode, NodeData
│   │   │   ├── document.rs     # Document arena and hierarchy operations
│   │   │   ├── iter.rs         # Traversal iterators (Children, Elements, Ancestors, Descendants)
│   │   │   └── error.rs        # DomError validation and hierarchy errors
│   │   └── tests/
│   │       └── dom_tests.rs
│   │
│   └── astra-html/             # Custom HTML tokenizer and tree builder
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── token.rs        # Token definitions (StartTag, EndTag, Doctype, etc.)
│       │   ├── tokenizer.rs    # Streaming HTML tokenizer with RCDATA/RAWTEXT state support
│       │   ├── entity.rs       # Named and numeric entity decoder
│       │   ├── parser.rs       # High-level Parser facade
│       │   └── tree_builder/   # HTML5 tree construction engine
│       │       ├── mod.rs      # TreeBuilder coordinator and mode dispatch
│       │       ├── mode.rs     # InsertionMode state machine enum
│       │       ├── stack.rs    # OpenElementStack with scope checking
│       │       ├── formatting.rs # ActiveFormattingElements list with markers
│       │       ├── location.rs # Target InsertionLocation and foster parenting
│       │       └── quirks.rs   # DOCTYPE quirks mode determination
│
└── tests/                      # End-to-end and integration tests
    ├── Cargo.toml
    ├── milestone.rs            # Core v0.0.1 milestone verification test
    ├── html_tests.rs           # Tokenizer, attributes, comments, and void element tests
    └── tree_construction_tests.rs # Milestone 2 HTML5 tree construction test suite
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
- **Quirks Mode Representation**:
  `Document::quirks_mode(&self) -> QuirksMode` and `Document::set_quirks_mode(&mut self, QuirksMode)` track `NoQuirks`, `Quirks`, and `LimitedQuirks` compatibility states.
- **Traversals**:
  - `children(parent)`: Iterates direct children.
  - `element_children(parent)`: Iterates only element children.
  - `ancestors(node)`: Walks up to the document root.
  - `descendants(node)`: Pre-order depth-first traversal of all descendants.

---

### 2. `astra-html` Tree Construction Engine

`astra-html` implements our proprietary HTML tokenizer and HTML5 tree construction state machine:

#### Tokenizer States
- **`Data`**: Standard HTML tokenization of elements, attributes, entities, comments, and DOCTYPEs.
- **`Rcdata(tag)`**: Used for `<title>` and `<textarea>`. Entities are resolved, but only the matching `</tag>` end tag exits the mode.
- **`Rawtext(tag)`**: Used for `<style>`. Entities remain unescaped; only the matching `</tag>` end tag exits the mode.
- **`ScriptData(tag)`**: Used for `<script>`. Code is preserved verbatim until matching `</script>`.

#### Tree Builder (`TreeBuilder`)
The tree builder consumes tokens emitted by `Tokenizer` and operates an explicit insertion-mode state machine:

1. **`Initial`**: Processes DOCTYPE tokens, infers quirks mode via `determine_quirks_mode`, or defaults to quirks mode on early content.
2. **`BeforeHtml`**: Handles root comments and whitespace; auto-inserts `<html>` if missing.
3. **`BeforeHead`**: Creates or auto-inserts `<head>`.
4. **`InHead`**: Inserts metadata, scripts, styles, titles, and dispatches tokenizer state transitions for RCDATA and RAWTEXT.
5. **`AfterHead`**: Creates or auto-inserts `<body>`.
6. **`InBody`**: Core body processing with block paragraph auto-closing, list item auto-closing, heading recovery, formatting elements, and void tags.
7. **`Text`**: Inserts character data into RCDATA or RAWTEXT nodes and returns to `original_insertion_mode` on end tag.
8. **`AfterBody`**: Handles comments and transitions to `AfterAfterBody` on `</html>`.
9. **`AfterAfterBody`**: Trailing comments and EOF termination.

#### Table Insertion Modes & Foster Parenting
- **`InTable`**: Direct table children (`caption`, `colgroup`, `col`, `tbody`, `thead`, `tfoot`, `tr`).
- **`InTableText` / Foster Parenting**: Non-whitespace text and non-table elements (e.g. `<b>`, `<p>`) encountered in table contexts trigger **foster parenting**, placing them into the table's parent *before* the `<table>` element in the DOM tree.
- **`InCaption`**: Table caption context with marker boundaries.
- **`InColumnGroup`**: Column group elements and column void elements.
- **`InTableBody`**: Automatically wraps `<tr>` elements in `<tbody>` when omitted.
- **`InRow`**: Automatically wraps cells in `<tr>`.
- **`InCell`**: Cells (`<td>`, `<th>`) act as normal body contexts with formatting markers.

#### Open Element Stack & Active Formatting Elements
- **`OpenElementStack`**: Implements standard scope checks:
  - `has_p_in_button_scope`: Used to auto-close paragraphs when block elements or new paragraphs appear.
  - `has_heading_in_scope` / `pop_heading_in_scope`: Closes previous headings when new headings or block elements appear.
  - `has_li_in_list_scope`: Closes preceding list items (`<li>`) in list context.
  - `has_dt_or_dd_in_scope`: Closes preceding definition terms and descriptions (`<dt>`, `<dd>`).
  - `has_element_in_table_scope`: Table boundary scope.
- **`ActiveFormattingElements`**: Tracks inline styling (`<b>`, `<i>`, `<a>`, etc.) across block boundaries using scope markers.

---

## Implemented vs. Planned Features

### Implemented in Milestone 2
- [x] Explicit 16-mode insertion state machine.
- [x] Automatic insertion of missing `<html>`, `<head>`, and `<body>` elements.
- [x] Prevention of duplicate structural element insertions.
- [x] Block-level paragraph auto-closing (`<p>First<p>Second` -> siblings).
- [x] List item auto-closing (`<li>Alpha<li>Beta` -> siblings under `<ul>`).
- [x] Definition list auto-closing (`<dt>` / `<dd>`).
- [x] Heading auto-closing (`<h1>` closed by `<h2>` or block elements).
- [x] Full table insertion modes (`caption`, `colgroup`, `tbody`, `tr`, `td`, `th`).
- [x] Foster parenting for misnested table text and elements.
- [x] Tokenizer RCDATA (`<title>`, `<textarea>`) and RAWTEXT (`<style>`, `<script>`) states.
- [x] DOCTYPE quirks mode determination (`NoQuirks`, `Quirks`, `LimitedQuirks`).
- [x] 36/36 automated unit and integration tests passing.

### Planned for Future Milestones
- [ ] Complex Adoption Agency Algorithm (AAA) full edge cases for deeply nested formatting elements.
- [ ] Foreign content namespaces (SVG and MathML integration).
- [ ] CSS tokenizer, parser, and CSSOM (Milestone 3).
- [ ] Style resolution and layout engine (Milestone 4).
- [ ] QuickJS JavaScript runtime integration (Milestone 5).
