//! # Astra HTML
//!
//! `astra-html` provides an independent, custom HTML tokenizer and parser for the
//! Astra Engine.
//!
//! ## Architecture
//!
//! - [`tokenizer`]: Converts character streams into structured tokens ([`Token`]) and manages
//!   state transitions (e.g. RCDATA, RAWTEXT, ScriptData).
//! - [`tree_builder`]: HTML5 tree construction state machine managing insertion modes, open
//!   element scopes, active formatting elements, and foster parenting.
//! - [`parser`]: High-level parser facade constructing an arena-based DOM tree ([`astra_dom::Document`]).
//! - [`entity`]: Standard entity resolution and numeric character reference decoding.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entity;
pub mod parser;
pub mod token;
pub mod tokenizer;
pub mod tree_builder;

pub use parser::{Parser, ParserOptions, is_void_element, parse_html, parse_html_with_options};
pub use token::{DoctypeToken, EndTagToken, StartTagToken, Token};
pub use tokenizer::{Tokenizer, TokenizerState};
pub use tree_builder::TreeBuilder;
pub use tree_builder::formatting::{ActiveFormattingElements, FormattingEntry};
pub use tree_builder::location::InsertionLocation;
pub use tree_builder::mode::InsertionMode;
pub use tree_builder::stack::OpenElementStack;
