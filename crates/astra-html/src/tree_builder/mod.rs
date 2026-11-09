//! HTML5 Tree Builder coordinating state machine transitions and DOM construction.

pub mod formatting;
pub mod location;
pub mod mode;
pub mod quirks;
pub mod stack;

use astra_dom::{Document, NodeId, QuirksMode};

use crate::parser::ParserOptions;
use crate::token::{EndTagToken, StartTagToken, Token};
use crate::tokenizer::{Tokenizer, TokenizerState};
use formatting::ActiveFormattingElements;
use location::InsertionLocation;
pub use mode::InsertionMode;
use quirks::determine_quirks_mode;
pub use stack::OpenElementStack;

/// The core HTML tree construction engine.
#[derive(Debug)]
pub struct TreeBuilder {
    document: Document,
    mode: InsertionMode,
    original_insertion_mode: Option<InsertionMode>,
    stack: OpenElementStack,
    formatting_elements: ActiveFormattingElements,
    head_element: Option<NodeId>,
    options: ParserOptions,
    foster_parenting: bool,
    reprocess_depth: usize,
}

impl TreeBuilder {
    /// Creates a new `TreeBuilder` with the specified options.
    #[must_use]
    pub fn new(options: ParserOptions) -> Self {
        let document = Document::new();
        let root = document.root();
        Self {
            document,
            mode: InsertionMode::Initial,
            original_insertion_mode: None,
            stack: OpenElementStack::new(root),
            formatting_elements: ActiveFormattingElements::new(),
            head_element: None,
            options,
            foster_parenting: false,
            reprocess_depth: 0,
        }
    }

    /// Returns the current insertion mode.
    #[must_use]
    pub const fn mode(&self) -> InsertionMode {
        self.mode
    }

    /// Returns a reference to the open element stack.
    #[must_use]
    pub const fn stack(&self) -> &OpenElementStack {
        &self.stack
    }

    /// Returns a reference to the active formatting elements list.
    #[must_use]
    pub const fn formatting_elements(&self) -> &ActiveFormattingElements {
        &self.formatting_elements
    }

    /// Completes tree construction and consumes the builder, returning the resulting DOM [`Document`].
    #[must_use]
    pub fn finish(mut self) -> Document {
        // If still in Initial mode (e.g. empty document), set quirks mode
        if self.mode == InsertionMode::Initial {
            self.document.set_quirks_mode(QuirksMode::Quirks);
        }
        self.document
    }

    /// Returns the current target insertion location for new nodes.
    #[must_use]
    pub fn current_insertion_location(&self) -> InsertionLocation {
        if self.foster_parenting {
            InsertionLocation::calculate_foster_parenting(&self.stack, &self.document)
        } else {
            InsertionLocation::Append(self.stack.current_node())
        }
    }

    /// Inserts an element at the current insertion location and returns its [`NodeId`].
    pub fn insert_element(
        &mut self,
        tag_name: String,
        attributes: Vec<(String, String)>,
    ) -> NodeId {
        let location = self.current_insertion_location();
        let elem_id = self
            .document
            .create_element_with_attributes(tag_name, attributes);
        let _ = location.insert(&mut self.document, elem_id);
        elem_id
    }

    /// Inserts text into the current insertion location, coalescing with adjacent text if possible.
    pub fn insert_text(&mut self, text: String) {
        if text.chars().all(char::is_whitespace) && !self.options.preserve_whitespace_only_text {
            return;
        }

        let location = self.current_insertion_location();
        match location {
            InsertionLocation::Append(parent) => {
                if let Some(last_child) = self.document.last_child(parent) {
                    if let Some(node) = self.document.get_mut(last_child) {
                        if let Some(text_node) = node.as_text_mut() {
                            text_node.append(&text);
                            return;
                        }
                    }
                }
                let text_id = self.document.create_text(text);
                let _ = self.document.append_child(parent, text_id);
            }
            InsertionLocation::InsertBefore {
                parent,
                reference_child,
            } => {
                if let Some(prev) = self.document.previous_sibling(reference_child) {
                    if let Some(node) = self.document.get_mut(prev) {
                        if let Some(text_node) = node.as_text_mut() {
                            text_node.append(&text);
                            return;
                        }
                    }
                }
                let text_id = self.document.create_text(text);
                let _ = self
                    .document
                    .insert_before(parent, text_id, reference_child);
            }
        }
    }

    /// Inserts a comment node into the current insertion location.
    pub fn insert_comment(&mut self, content: String) -> NodeId {
        let location = self.current_insertion_location();
        let comment_id = self.document.create_comment(content);
        let _ = location.insert(&mut self.document, comment_id);
        comment_id
    }

    /// Dispatches a token to the active insertion mode.
    pub fn process_token(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        if self.reprocess_depth > 16 {
            // Guard against unbounded reprocessing cycles on malformed inputs
            return;
        }

        match self.mode {
            InsertionMode::Initial => self.handle_initial(token, tokenizer),
            InsertionMode::BeforeHtml => self.handle_before_html(token, tokenizer),
            InsertionMode::BeforeHead => self.handle_before_head(token, tokenizer),
            InsertionMode::InHead => self.handle_in_head(token, tokenizer),
            InsertionMode::AfterHead => self.handle_after_head(token, tokenizer),
            InsertionMode::InBody => self.handle_in_body(token, tokenizer),
            InsertionMode::Text => self.handle_text(token, tokenizer),
            InsertionMode::AfterBody => self.handle_after_body(token, tokenizer),
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token, tokenizer),

            // Table modes
            InsertionMode::InTable => self.handle_in_table(token, tokenizer),
            InsertionMode::InTableText => self.handle_in_table(token, tokenizer),
            InsertionMode::InCaption => self.handle_in_caption(token, tokenizer),
            InsertionMode::InColumnGroup => self.handle_in_column_group(token, tokenizer),
            InsertionMode::InTableBody => self.handle_in_table_body(token, tokenizer),
            InsertionMode::InRow => self.handle_in_row(token, tokenizer),
            InsertionMode::InCell => self.handle_in_cell(token, tokenizer),
        }
    }

    fn reprocess(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        self.reprocess_depth += 1;
        self.process_token(token, tokenizer);
        self.reprocess_depth -= 1;
    }

    // --- Mode Implementations ---

    fn handle_initial(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Doctype(ref doctype) => {
                let dt_id = self.document.create_doctype(
                    &doctype.name,
                    doctype.public_id.clone(),
                    doctype.system_id.clone(),
                );
                let _ = self.document.append_child(self.document.root(), dt_id);
                let quirks = determine_quirks_mode(Some(doctype));
                self.document.set_quirks_mode(quirks);
                self.mode = InsertionMode::BeforeHtml;
            }
            Token::Comment(comment) => {
                let comment_id = self.document.create_comment(comment);
                let _ = self.document.append_child(self.document.root(), comment_id);
            }
            Token::Text(ref text) if text.chars().all(char::is_whitespace) => {
                // Ignore whitespace in initial mode
            }
            other => {
                self.document.set_quirks_mode(QuirksMode::Quirks);
                self.mode = InsertionMode::BeforeHtml;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_before_html(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Doctype(_) => {
                // Ignore stray DOCTYPE
            }
            Token::Comment(comment) => {
                let comment_id = self.document.create_comment(comment);
                let _ = self.document.append_child(self.document.root(), comment_id);
            }
            Token::Text(ref text) if text.chars().all(char::is_whitespace) => {
                // Ignore whitespace before html
            }
            Token::StartTag(ref tag) if tag.name == "html" => {
                let html_id = self
                    .document
                    .create_element_with_attributes(&tag.name, tag.attributes.clone());
                let _ = self.document.append_child(self.document.root(), html_id);
                self.stack.push(html_id);
                self.mode = InsertionMode::BeforeHead;
            }
            Token::EndTag(ref tag)
                if !matches!(tag.name.as_str(), "head" | "body" | "html" | "br") =>
            {
                // Ignore stray end tags
            }
            other => {
                // Automatically insert <html>
                let html_id = self.document.create_element("html");
                let _ = self.document.append_child(self.document.root(), html_id);
                self.stack.push(html_id);
                self.mode = InsertionMode::BeforeHead;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_before_head(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(ref text) if text.chars().all(char::is_whitespace) => {
                // Ignore whitespace before head
            }
            Token::Comment(comment) => {
                let current = self.stack.current_node();
                let comment_id = self.document.create_comment(comment);
                let _ = self.document.append_child(current, comment_id);
            }
            Token::Doctype(_) => {}
            Token::StartTag(ref tag) if tag.name == "html" => {
                self.handle_in_body(token, tokenizer);
            }
            Token::StartTag(ref tag) if tag.name == "head" => {
                let head_id = self
                    .document
                    .create_element_with_attributes(&tag.name, tag.attributes.clone());
                let current = self.stack.current_node();
                let _ = self.document.append_child(current, head_id);
                self.stack.push(head_id);
                self.head_element = Some(head_id);
                self.mode = InsertionMode::InHead;
            }
            Token::EndTag(ref tag)
                if !matches!(tag.name.as_str(), "head" | "body" | "html" | "br") =>
            {
                // Ignore stray end tag
            }
            other => {
                // Automatically insert <head>
                let head_id = self.document.create_element("head");
                let current = self.stack.current_node();
                let _ = self.document.append_child(current, head_id);
                self.stack.push(head_id);
                self.head_element = Some(head_id);
                self.mode = InsertionMode::InHead;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_in_head(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) if text.chars().all(char::is_whitespace) => {
                self.insert_text(text);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {}
            Token::StartTag(tag) => match tag.name.as_str() {
                "html" => self.handle_in_body(Token::StartTag(tag), tokenizer),
                "base" | "basefont" | "bgsound" | "link" | "meta" => {
                    let elem_id = self
                        .document
                        .create_element_with_attributes(tag.name, tag.attributes);
                    let current = self.stack.current_node();
                    let _ = self.document.append_child(current, elem_id);
                }
                "title" => {
                    let elem_id = self
                        .document
                        .create_element_with_attributes(&tag.name, tag.attributes);
                    let current = self.stack.current_node();
                    let _ = self.document.append_child(current, elem_id);
                    self.stack.push(elem_id);
                    tokenizer.set_state(TokenizerState::Rcdata("title".to_string()));
                    self.original_insertion_mode = Some(InsertionMode::InHead);
                    self.mode = InsertionMode::Text;
                }
                "style" => {
                    let elem_id = self
                        .document
                        .create_element_with_attributes(&tag.name, tag.attributes);
                    let current = self.stack.current_node();
                    let _ = self.document.append_child(current, elem_id);
                    self.stack.push(elem_id);
                    tokenizer.set_state(TokenizerState::Rawtext("style".to_string()));
                    self.original_insertion_mode = Some(InsertionMode::InHead);
                    self.mode = InsertionMode::Text;
                }
                "script" => {
                    let elem_id = self
                        .document
                        .create_element_with_attributes(&tag.name, tag.attributes);
                    let current = self.stack.current_node();
                    let _ = self.document.append_child(current, elem_id);
                    self.stack.push(elem_id);
                    tokenizer.set_state(TokenizerState::ScriptData("script".to_string()));
                    self.original_insertion_mode = Some(InsertionMode::InHead);
                    self.mode = InsertionMode::Text;
                }
                "head" => {
                    // Ignore duplicate head
                }
                _ => {
                    self.stack.pop();
                    self.mode = InsertionMode::AfterHead;
                    self.reprocess(Token::StartTag(tag), tokenizer);
                }
            },
            Token::EndTag(ref tag) if tag.name == "head" => {
                self.stack.pop();
                self.mode = InsertionMode::AfterHead;
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                // Ignore stray end tag
            }
            other => {
                self.stack.pop();
                self.mode = InsertionMode::AfterHead;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_after_head(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) if text.chars().all(char::is_whitespace) => {
                self.insert_text(text);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {}
            Token::StartTag(ref tag) if tag.name == "html" => {
                self.handle_in_body(token, tokenizer);
            }
            Token::StartTag(tag) if tag.name == "body" => {
                let body_id = self
                    .document
                    .create_element_with_attributes(tag.name, tag.attributes);
                let current = self.stack.current_node();
                let _ = self.document.append_child(current, body_id);
                self.stack.push(body_id);
                self.mode = InsertionMode::InBody;
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                // Ignore stray end tag
            }
            other => {
                // Automatically insert <body>
                let body_id = self.document.create_element("body");
                let current = self.stack.current_node();
                let _ = self.document.append_child(current, body_id);
                self.stack.push(body_id);
                self.mode = InsertionMode::InBody;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_in_body(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) => {
                self.insert_text(text);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {}
            Token::StartTag(tag) => self.handle_in_body_start_tag(tag, tokenizer),
            Token::EndTag(tag) => self.handle_in_body_end_tag(tag, tokenizer),
            Token::Eof => {
                // Parsing completed
            }
        }
    }

    fn handle_in_body_start_tag(&mut self, tag: StartTagToken, tokenizer: &mut Tokenizer) {
        let name = tag.name.as_str();
        match name {
            "html" => {
                // Ignore in body
            }
            "body" => {
                // Ignore duplicate body tag
            }
            "title" | "textarea" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                let elem_id = self.insert_element(tag.name.clone(), tag.attributes);
                self.stack.push(elem_id);
                tokenizer.set_state(TokenizerState::Rcdata(tag.name));
                self.original_insertion_mode = Some(InsertionMode::InBody);
                self.mode = InsertionMode::Text;
            }
            "style" => {
                let elem_id = self.insert_element(tag.name.clone(), tag.attributes);
                self.stack.push(elem_id);
                tokenizer.set_state(TokenizerState::Rawtext("style".to_string()));
                self.original_insertion_mode = Some(InsertionMode::InBody);
                self.mode = InsertionMode::Text;
            }
            "script" => {
                let elem_id = self.insert_element(tag.name.clone(), tag.attributes);
                self.stack.push(elem_id);
                tokenizer.set_state(TokenizerState::ScriptData("script".to_string()));
                self.original_insertion_mode = Some(InsertionMode::InBody);
                self.mode = InsertionMode::Text;
            }
            // Block-level elements that close <p> in button scope and auto-close headings
            "address" | "article" | "aside" | "blockquote" | "details" | "dialog" | "dir"
            | "div" | "dl" | "fieldset" | "figcaption" | "figure" | "footer" | "header"
            | "hgroup" | "main" | "menu" | "nav" | "ol" | "p" | "section" | "ul" | "pre" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                if self.stack.has_heading_in_scope(&self.document) {
                    self.stack.pop_heading_in_scope(&self.document);
                }
                let elem_id = self.insert_element(tag.name, tag.attributes);
                self.stack.push(elem_id);
            }
            "hr" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                if self.stack.has_heading_in_scope(&self.document) {
                    self.stack.pop_heading_in_scope(&self.document);
                }
                // Void element, do not push to stack
                self.insert_element(tag.name, tag.attributes);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                if self.stack.has_heading_in_scope(&self.document) {
                    self.stack.pop_heading_in_scope(&self.document);
                }
                let elem_id = self.insert_element(tag.name, tag.attributes);
                self.stack.push(elem_id);
            }
            "li" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                if self.stack.has_li_in_list_scope(&self.document) {
                    self.stack.pop_until("li", &self.document);
                }
                let elem_id = self.insert_element(tag.name, tag.attributes);
                self.stack.push(elem_id);
            }
            "dt" | "dd" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                if self.stack.has_dt_or_dd_in_scope(&self.document) {
                    for tag_str in ["dt", "dd"] {
                        if self.stack.find_last_tag(tag_str, &self.document).is_some() {
                            self.stack.pop_until(tag_str, &self.document);
                            break;
                        }
                    }
                }
                let elem_id = self.insert_element(tag.name, tag.attributes);
                self.stack.push(elem_id);
            }
            // Formatting elements
            "b" | "big" | "code" | "em" | "font" | "i" | "s" | "small" | "strike" | "strong"
            | "tt" | "u" => {
                let elem_id = self.insert_element(tag.name.clone(), tag.attributes);
                self.stack.push(elem_id);
                self.formatting_elements.push_element(elem_id, tag.name);
            }
            "a" => {
                if let Some(existing_a) = self.formatting_elements.find_last_after_marker("a") {
                    self.stack.pop_until("a", &self.document);
                    self.formatting_elements.remove(existing_a);
                }
                let elem_id = self.insert_element(tag.name.clone(), tag.attributes);
                self.stack.push(elem_id);
                self.formatting_elements.push_element(elem_id, tag.name);
            }
            "table" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                }
                let elem_id = self.insert_element(tag.name, tag.attributes);
                self.stack.push(elem_id);
                self.mode = InsertionMode::InTable;
            }
            // Void elements
            "area" | "br" | "embed" | "img" | "input" | "wbr" | "link" | "meta" => {
                self.insert_element(tag.name, tag.attributes);
            }
            _ => {
                let is_self_closing = tag.self_closing;
                let elem_id = self.insert_element(tag.name, tag.attributes);
                if !is_self_closing {
                    self.stack.push(elem_id);
                }
            }
        }
    }

    fn handle_in_body_end_tag(&mut self, tag: EndTagToken, tokenizer: &mut Tokenizer) {
        let name = tag.name.as_str();
        match name {
            "body" => {
                if self.stack.contains_tag("body", &self.document) {
                    self.mode = InsertionMode::AfterBody;
                }
            }
            "html" => {
                if self.stack.contains_tag("body", &self.document) {
                    self.mode = InsertionMode::AfterBody;
                    self.reprocess(Token::EndTag(tag), tokenizer);
                }
            }
            "p" => {
                if self.stack.has_p_in_button_scope(&self.document) {
                    self.stack.pop_until("p", &self.document);
                } else {
                    let p_id = self.insert_element("p".to_string(), Vec::new());
                    self.stack.push(p_id);
                    self.stack.pop();
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if self.stack.has_heading_in_scope(&self.document) {
                    self.stack.pop_until(name, &self.document);
                }
            }
            "li" => {
                if self.stack.has_li_in_list_scope(&self.document) {
                    self.stack.pop_until("li", &self.document);
                }
            }
            "dt" | "dd" => {
                if self.stack.has_dt_or_dd_in_scope(&self.document) {
                    self.stack.pop_until(name, &self.document);
                }
            }
            "a" | "b" | "big" | "code" | "em" | "font" | "i" | "s" | "small" | "strike"
            | "strong" | "tt" | "u" => {
                if let Some(id) = self.formatting_elements.find_last_after_marker(name) {
                    self.stack.pop_until(name, &self.document);
                    self.formatting_elements.remove(id);
                }
            }
            _ => {
                if self.stack.contains_tag(name, &self.document) {
                    self.stack.pop_until(name, &self.document);
                }
            }
        }
    }

    fn handle_text(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) => {
                self.insert_text(text);
            }
            Token::EndTag(_) => {
                self.stack.pop();
                self.mode = self
                    .original_insertion_mode
                    .take()
                    .unwrap_or(InsertionMode::InBody);
            }
            Token::Eof => {
                self.stack.pop();
                self.mode = self
                    .original_insertion_mode
                    .take()
                    .unwrap_or(InsertionMode::InBody);
                self.reprocess(Token::Eof, tokenizer);
            }
            _ => {}
        }
    }

    fn handle_after_body(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(ref text) if text.chars().all(char::is_whitespace) => {
                self.handle_in_body(token, tokenizer);
            }
            Token::Comment(comment) => {
                if let Some(html_id) = self.stack.find_last_tag("html", &self.document) {
                    let comment_id = self.document.create_comment(comment);
                    let _ = self.document.append_child(html_id, comment_id);
                }
            }
            Token::Doctype(_) => {}
            Token::EndTag(ref tag) if tag.name == "html" => {
                self.mode = InsertionMode::AfterAfterBody;
            }
            Token::Eof => {}
            other => {
                self.mode = InsertionMode::InBody;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_after_after_body(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Comment(comment) => {
                let comment_id = self.document.create_comment(comment);
                let _ = self.document.append_child(self.document.root(), comment_id);
            }
            Token::Doctype(_) => {}
            Token::Text(ref text) if text.chars().all(char::is_whitespace) => {
                self.handle_in_body(token, tokenizer);
            }
            Token::Eof => {}
            other => {
                self.mode = InsertionMode::InBody;
                self.reprocess(other, tokenizer);
            }
        }
    }

    // --- Table Handling Methods ---

    fn handle_in_table(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) => {
                if text.chars().all(char::is_whitespace) {
                    self.insert_text(text);
                } else {
                    // Foster parenting for non-whitespace text in table context
                    let location =
                        InsertionLocation::calculate_foster_parenting(&self.stack, &self.document);
                    let text_id = self.document.create_text(text);
                    let _ = location.insert(&mut self.document, text_id);
                }
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {}
            Token::StartTag(tag) => match tag.name.as_str() {
                "caption" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    self.formatting_elements.push_marker();
                    let cap_id = self.insert_element(tag.name, tag.attributes);
                    self.stack.push(cap_id);
                    self.mode = InsertionMode::InCaption;
                }
                "colgroup" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    let cg_id = self.insert_element(tag.name, tag.attributes);
                    self.stack.push(cg_id);
                    self.mode = InsertionMode::InColumnGroup;
                }
                "col" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    let cg_id = self.insert_element("colgroup".to_string(), Vec::new());
                    self.stack.push(cg_id);
                    self.mode = InsertionMode::InColumnGroup;
                    self.reprocess(Token::StartTag(tag), tokenizer);
                }
                "tbody" | "thead" | "tfoot" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    let sec_id = self.insert_element(tag.name, tag.attributes);
                    self.stack.push(sec_id);
                    self.mode = InsertionMode::InTableBody;
                }
                "tr" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    let tb_id = self.insert_element("tbody".to_string(), Vec::new());
                    self.stack.push(tb_id);
                    self.mode = InsertionMode::InTableBody;
                    self.reprocess(Token::StartTag(tag), tokenizer);
                }
                "td" | "th" => {
                    self.stack.clear_back_to_table_context(&self.document);
                    let tb_id = self.insert_element("tbody".to_string(), Vec::new());
                    self.stack.push(tb_id);
                    self.mode = InsertionMode::InTableBody;
                    self.reprocess(Token::StartTag(tag), tokenizer);
                }
                "table" => {
                    if self
                        .stack
                        .has_element_in_table_scope("table", &self.document)
                    {
                        self.stack.pop_until("table", &self.document);
                        self.reset_insertion_mode();
                        self.reprocess(Token::StartTag(tag), tokenizer);
                    }
                }
                _ => {
                    // Foster parenting for non-table elements
                    self.foster_parenting = true;
                    self.handle_in_body(Token::StartTag(tag), tokenizer);
                    self.foster_parenting = false;
                }
            },
            Token::EndTag(tag) => match tag.name.as_str() {
                "table" => {
                    if self
                        .stack
                        .has_element_in_table_scope("table", &self.document)
                    {
                        self.stack.pop_until("table", &self.document);
                        self.reset_insertion_mode();
                    }
                }
                "body" | "html" => {}
                _ => {
                    self.foster_parenting = true;
                    self.handle_in_body(Token::EndTag(tag), tokenizer);
                    self.foster_parenting = false;
                }
            },
            Token::Eof => {
                self.handle_in_body(Token::Eof, tokenizer);
            }
        }
    }

    fn handle_in_caption(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::EndTag(ref tag) if tag.name == "caption" => {
                self.stack.pop_until("caption", &self.document);
                self.formatting_elements.clear_to_last_marker();
                self.mode = InsertionMode::InTable;
            }
            Token::EndTag(ref tag) if tag.name == "table" => {
                self.stack.pop_until("caption", &self.document);
                self.formatting_elements.clear_to_last_marker();
                self.mode = InsertionMode::InTable;
                self.reprocess(token, tokenizer);
            }
            other => {
                self.handle_in_body(other, tokenizer);
            }
        }
    }

    fn handle_in_column_group(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::Text(text) if text.chars().all(char::is_whitespace) => {
                self.insert_text(text);
            }
            Token::Comment(comment) => {
                self.insert_comment(comment);
            }
            Token::Doctype(_) => {}
            Token::StartTag(tag) if tag.name == "col" => {
                self.insert_element(tag.name, tag.attributes);
            }
            Token::EndTag(ref tag) if tag.name == "colgroup" => {
                self.stack.pop_until("colgroup", &self.document);
                self.mode = InsertionMode::InTable;
            }
            other => {
                self.stack.pop_until("colgroup", &self.document);
                self.mode = InsertionMode::InTable;
                self.reprocess(other, tokenizer);
            }
        }
    }

    fn handle_in_table_body(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::StartTag(tag) => match tag.name.as_str() {
                "tr" => {
                    self.stack.clear_back_to_table_body_context(&self.document);
                    let tr_id = self.insert_element(tag.name, tag.attributes);
                    self.stack.push(tr_id);
                    self.mode = InsertionMode::InRow;
                }
                "th" | "td" => {
                    self.stack.clear_back_to_table_body_context(&self.document);
                    let tr_id = self.insert_element("tr".to_string(), Vec::new());
                    self.stack.push(tr_id);
                    self.mode = InsertionMode::InRow;
                    self.reprocess(Token::StartTag(tag), tokenizer);
                }
                "tbody" | "thead" | "tfoot" => {
                    if self
                        .stack
                        .has_element_in_table_scope(&tag.name, &self.document)
                    {
                        self.stack.clear_back_to_table_body_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTable;
                        self.reprocess(Token::StartTag(tag), tokenizer);
                    }
                }
                _ => {
                    self.handle_in_table(Token::StartTag(tag), tokenizer);
                }
            },
            Token::EndTag(tag) => match tag.name.as_str() {
                "tbody" | "thead" | "tfoot" => {
                    if self
                        .stack
                        .has_element_in_table_scope(&tag.name, &self.document)
                    {
                        self.stack.clear_back_to_table_body_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTable;
                    }
                }
                "table" => {
                    if self
                        .stack
                        .has_element_in_table_scope("table", &self.document)
                    {
                        self.stack.clear_back_to_table_body_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTable;
                        self.reprocess(Token::EndTag(tag), tokenizer);
                    }
                }
                _ => {
                    self.handle_in_table(Token::EndTag(tag), tokenizer);
                }
            },
            other => {
                self.handle_in_table(other, tokenizer);
            }
        }
    }

    fn handle_in_row(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::StartTag(tag) => match tag.name.as_str() {
                "th" | "td" => {
                    self.stack.clear_back_to_table_row_context(&self.document);
                    let cell_id = self.insert_element(tag.name, tag.attributes);
                    self.stack.push(cell_id);
                    self.formatting_elements.push_marker();
                    self.mode = InsertionMode::InCell;
                }
                "tr" => {
                    if self.stack.has_element_in_table_scope("tr", &self.document) {
                        self.stack.clear_back_to_table_row_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTableBody;
                        self.reprocess(Token::StartTag(tag), tokenizer);
                    }
                }
                "tbody" | "thead" | "tfoot" | "table" => {
                    if self.stack.has_element_in_table_scope("tr", &self.document) {
                        self.stack.clear_back_to_table_row_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTableBody;
                        self.reprocess(Token::StartTag(tag), tokenizer);
                    }
                }
                _ => {
                    self.handle_in_table(Token::StartTag(tag), tokenizer);
                }
            },
            Token::EndTag(tag) => match tag.name.as_str() {
                "tr" => {
                    if self.stack.has_element_in_table_scope("tr", &self.document) {
                        self.stack.clear_back_to_table_row_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTableBody;
                    }
                }
                "table" => {
                    if self.stack.has_element_in_table_scope("tr", &self.document) {
                        self.stack.clear_back_to_table_row_context(&self.document);
                        self.stack.pop();
                        self.mode = InsertionMode::InTableBody;
                        self.reprocess(Token::EndTag(tag), tokenizer);
                    }
                }
                "tbody" | "thead" | "tfoot" => {
                    if self
                        .stack
                        .has_element_in_table_scope(&tag.name, &self.document)
                    {
                        if self.stack.has_element_in_table_scope("tr", &self.document) {
                            self.stack.clear_back_to_table_row_context(&self.document);
                            self.stack.pop();
                            self.mode = InsertionMode::InTableBody;
                            self.reprocess(Token::EndTag(tag), tokenizer);
                        }
                    }
                }
                _ => {
                    self.handle_in_table(Token::EndTag(tag), tokenizer);
                }
            },
            other => {
                self.handle_in_table(other, tokenizer);
            }
        }
    }

    fn handle_in_cell(&mut self, token: Token, tokenizer: &mut Tokenizer) {
        match token {
            Token::EndTag(ref tag) if tag.name == "td" || tag.name == "th" => {
                if self
                    .stack
                    .has_element_in_table_scope(&tag.name, &self.document)
                {
                    self.stack.pop_until(&tag.name, &self.document);
                    self.formatting_elements.clear_to_last_marker();
                    self.mode = InsertionMode::InRow;
                }
            }
            Token::EndTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "tr" | "tbody" | "thead" | "tfoot" | "table"
                ) =>
            {
                if self.close_cell() {
                    self.reprocess(token, tokenizer);
                }
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "td" | "th" | "tr" | "tbody" | "thead" | "tfoot" | "table"
                ) =>
            {
                if self.close_cell() {
                    self.reprocess(token, tokenizer);
                }
            }
            other => {
                self.handle_in_body(other, tokenizer);
            }
        }
    }

    fn close_cell(&mut self) -> bool {
        for tag in ["td", "th"] {
            if self.stack.has_element_in_table_scope(tag, &self.document) {
                self.stack.pop_until(tag, &self.document);
                self.formatting_elements.clear_to_last_marker();
                self.mode = InsertionMode::InRow;
                return true;
            }
        }
        false
    }

    fn reset_insertion_mode(&mut self) {
        for &id in self.stack.elements().iter().rev() {
            if let Some(node) = self.document.get(id) {
                if let Some(elem) = node.as_element() {
                    match elem.tag_name() {
                        "th" | "td" => {
                            self.mode = InsertionMode::InCell;
                            return;
                        }
                        "tr" => {
                            self.mode = InsertionMode::InRow;
                            return;
                        }
                        "tbody" | "thead" | "tfoot" => {
                            self.mode = InsertionMode::InTableBody;
                            return;
                        }
                        "caption" => {
                            self.mode = InsertionMode::InCaption;
                            return;
                        }
                        "colgroup" => {
                            self.mode = InsertionMode::InColumnGroup;
                            return;
                        }
                        "table" => {
                            self.mode = InsertionMode::InTable;
                            return;
                        }
                        "head" => {
                            self.mode = InsertionMode::InHead;
                            return;
                        }
                        "body" => {
                            self.mode = InsertionMode::InBody;
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.mode = InsertionMode::InBody;
    }
}
