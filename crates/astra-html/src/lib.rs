//! # Astra HTML
//!
//! `astra-html` provides an independent, custom HTML tokenizer and parser for the
//! Astra Engine.
//!
//! ## Architecture
//!
//! - [`tokenizer`]: Converts character streams into structured tokens ([`Token`]).
//! - [`parser`]: Consumes tokens and constructs an arena-based DOM tree ([`astra_dom::Document`]).
//! - [`entity`]: Standard entity resolution and numeric character reference decoding.
//!
//! ## Features
//!
//! - Custom streaming tokenizer supporting tags, attributes (quoted/unquoted/boolean), comments, and DOCTYPEs.
//! - Tree construction with nested element support, text coalescing, and comment preservation.
//! - Built-in handling of HTML void elements (`<img>`, `<br>`, `<meta>`, `<input>`, etc.).
//! - Basic malformed markup recovery (mismatched closing tags, unclosed tags at EOF, stray closing tags).
//! - Clean, zero-external-dependency Rust implementation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entity;
pub mod parser;
pub mod token;
pub mod tokenizer;

pub use parser::{Parser, ParserOptions, is_void_element, parse_html, parse_html_with_options};
pub use token::{DoctypeToken, EndTagToken, StartTagToken, Token};
pub use tokenizer::Tokenizer;
