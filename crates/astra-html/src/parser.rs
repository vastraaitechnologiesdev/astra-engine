//! Astra HTML Parser constructing a DOM tree from tokenizer tokens.

use astra_dom::Document;

use crate::token::Token;
use crate::tokenizer::Tokenizer;
use crate::tree_builder::TreeBuilder;

/// Configuration options for the HTML parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserOptions {
    /// Whether to preserve text nodes consisting purely of whitespace.
    ///
    /// Defaults to `true` (standard DOM behavior).
    pub preserve_whitespace_only_text: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            preserve_whitespace_only_text: true,
        }
    }
}

/// Checks whether a given HTML tag name is a standard void element.
///
/// Void elements cannot have child nodes and are self-closing in HTML.
#[must_use]
pub fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Astra HTML Parser.
///
/// Consumes a token stream and constructs an arena-based [`Document`] DOM tree.
#[derive(Debug)]
pub struct Parser {
    options: ParserOptions,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Creates a new `Parser` with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: ParserOptions::default(),
        }
    }

    /// Creates a new `Parser` with custom options.
    #[must_use]
    pub fn with_options(options: ParserOptions) -> Self {
        Self { options }
    }

    /// Parses an HTML input string into a DOM [`Document`].
    #[must_use]
    pub fn parse(&self, html: &str) -> Document {
        let mut builder = TreeBuilder::new(self.options);
        let mut tokenizer = Tokenizer::new(html);

        loop {
            let token = tokenizer.next_token();
            let is_eof = token == Token::Eof;
            builder.process_token(token, &mut tokenizer);
            if is_eof {
                break;
            }
        }

        builder.finish()
    }
}

/// Convenience function to parse an HTML string with default options.
#[must_use]
pub fn parse_html(html: &str) -> Document {
    Parser::new().parse(html)
}

/// Convenience function to parse an HTML string with custom options.
#[must_use]
pub fn parse_html_with_options(html: &str, options: ParserOptions) -> Document {
    Parser::with_options(options).parse(html)
}
