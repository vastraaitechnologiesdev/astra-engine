//! Astra HTML Parser constructing a DOM tree from tokenizer tokens.

use astra_dom::Document;

use crate::token::Token;
use crate::tokenizer::Tokenizer;

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
        let mut document = Document::new();
        let mut tokenizer = Tokenizer::new(html);
        let mut stack = vec![document.root()];

        loop {
            let token = tokenizer.next_token();
            if token == Token::Eof {
                break;
            }

            match token {
                Token::Doctype(doctype) => {
                    let doctype_id =
                        document.create_doctype(doctype.name, doctype.public_id, doctype.system_id);
                    // Doctype is attached directly to the document root
                    let _ = document.append_child(document.root(), doctype_id);
                }
                Token::StartTag(start_tag) => {
                    let parent = *stack.last().unwrap_or(&document.root());
                    let elem_id = document.create_element_with_attributes(
                        start_tag.name.clone(),
                        start_tag.attributes,
                    );
                    let _ = document.append_child(parent, elem_id);

                    let is_void = is_void_element(&start_tag.name);
                    if !is_void && !start_tag.self_closing {
                        stack.push(elem_id);
                    }
                }
                Token::EndTag(end_tag) => {
                    if is_void_element(&end_tag.name) {
                        // Void elements cannot be explicitly closed via end tag; ignore
                        continue;
                    }

                    // Search stack from top to bottom (excluding root at index 0)
                    let mut match_idx = None;
                    for i in (1..stack.len()).rev() {
                        let node_id = stack[i];
                        if let Some(node) = document.get(node_id) {
                            if let Some(elem) = node.as_element() {
                                if elem.tag_name().eq_ignore_ascii_case(&end_tag.name) {
                                    match_idx = Some(i);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(idx) = match_idx {
                        // Pop elements off stack down to and including the matched element
                        stack.truncate(idx);
                    }
                    // If not found in stack, this is a stray closing tag; ignore (error recovery)
                }
                Token::Text(text) => {
                    let is_whitespace_only = text.chars().all(char::is_whitespace);
                    if is_whitespace_only && !self.options.preserve_whitespace_only_text {
                        continue;
                    }

                    let parent = *stack.last().unwrap_or(&document.root());

                    // Coalesce with existing preceding text child if present
                    let mut coalesced = false;
                    if let Some(last_child) = document.last_child(parent) {
                        if let Some(node) = document.get_mut(last_child) {
                            if let Some(text_node) = node.as_text_mut() {
                                text_node.append(&text);
                                coalesced = true;
                            }
                        }
                    }

                    if !coalesced {
                        let text_id = document.create_text(text);
                        let _ = document.append_child(parent, text_id);
                    }
                }
                Token::Comment(comment) => {
                    let parent = *stack.last().unwrap_or(&document.root());
                    let comment_id = document.create_comment(comment);
                    let _ = document.append_child(parent, comment_id);
                }
                Token::Eof => break,
            }
        }

        document
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
