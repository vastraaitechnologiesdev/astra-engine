//! # Astra DOM
//!
//! `astra-dom` provides the foundational Document Object Model (DOM) data structures
//! for the Astra Engine.
//!
//! ## Design Highlights
//!
//! - **Arena-based memory layout**: Nodes are owned by a single [`Document`] arena and referenced
//!   by lightweight, stable [`NodeId`] handle types.
//! - **Zero cyclic reference leaks**: Bidirectional tree linkages use arena indices instead of
//!   `Rc<RefCell<...>>`, providing high cache locality and straightforward safety.
//! - **Rich traversals**: Complete tree traversal iterators ([`ChildrenIter`], [`ElementChildrenIter`],
//!   [`AncestorsIter`], [`DescendantsIter`]).
//! - **Zero external dependencies**: Implemented using pure Rust standard library constructs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod data;
pub mod document;
pub mod error;
pub mod id;
pub mod iter;
pub mod node;

pub use data::{Comment, DocumentType, Element, NodeData, QuirksMode, Text};
pub use document::Document;
pub use error::DomError;
pub use id::NodeId;
pub use iter::{AncestorsIter, ChildrenIter, DescendantsIter, ElementChildrenIter};
pub use node::Node;
