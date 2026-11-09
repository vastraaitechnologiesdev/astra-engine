//! Astra HTML Tokenizer converting raw markup into a stream of tokens.

use crate::entity::decode_entities;
use crate::token::{DoctypeToken, EndTagToken, StartTagToken, Token};

/// Tokenizer that scans HTML character data and produces [`Token`] instances.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    /// Creates a new `Tokenizer` from an input string slice.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    /// Returns `true` if the tokenizer has reached the end of the input.
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// Peeks at the next character without advancing the cursor.
    #[must_use]
    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Peeks at a character at an offset from current cursor position.
    #[must_use]
    pub fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    /// Advances and returns the next character.
    pub fn next_char(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    /// Consumes all consecutive ASCII whitespace characters.
    pub fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Checks if the remaining stream starts with the given string (case-sensitive).
    #[must_use]
    pub fn starts_with(&self, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        if self.pos + pattern_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &pch) in pattern_chars.iter().enumerate() {
            if self.chars[self.pos + i] != pch {
                return false;
            }
        }
        true
    }

    /// Checks if the remaining stream starts with the given string (ASCII case-insensitive).
    #[must_use]
    pub fn starts_with_case_insensitive(&self, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        if self.pos + pattern_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &pch) in pattern_chars.iter().enumerate() {
            if !self.chars[self.pos + i].eq_ignore_ascii_case(&pch) {
                return false;
            }
        }
        true
    }

    /// Advances the cursor by `count` characters.
    pub fn advance(&mut self, count: usize) {
        self.pos = (self.pos + count).min(self.chars.len());
    }

    /// Reads and returns the next token from the input.
    pub fn next_token(&mut self) -> Token {
        if self.is_eof() {
            return Token::Eof;
        }

        if self.peek() == Some('<') {
            if self.starts_with("<!--") {
                return self.tokenize_comment();
            }

            if self.starts_with_case_insensitive("<!doctype") {
                return self.tokenize_doctype();
            }

            if self.starts_with("<!") || self.starts_with("<?") {
                return self.tokenize_bogus_comment();
            }

            if self.starts_with("</") {
                return self.tokenize_end_tag();
            }

            // Check if it's a start tag (< followed by ASCII alphabetic)
            if let Some(c) = self.peek_at(1) {
                if c.is_ascii_alphabetic() {
                    return self.tokenize_start_tag();
                }
            }

            // Otherwise, stray '<' followed by non-tag characters (e.g. "a < b")
            return self.tokenize_text();
        }

        self.tokenize_text()
    }

    /// Tokenizes the entire input into a vector of tokens up to [`Token::Eof`].
    pub fn tokenize_all(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token == Token::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn tokenize_comment(&mut self) -> Token {
        // Skip "<!--"
        self.advance(4);

        let mut content = String::new();
        while !self.is_eof() {
            if self.starts_with("-->") {
                self.advance(3);
                break;
            }
            if let Some(ch) = self.next_char() {
                content.push(ch);
            }
        }

        Token::Comment(content)
    }

    fn tokenize_bogus_comment(&mut self) -> Token {
        // Skip "<!" or "<?"
        self.advance(2);

        let mut content = String::new();
        while let Some(ch) = self.next_char() {
            if ch == '>' {
                break;
            }
            content.push(ch);
        }

        Token::Comment(content)
    }

    fn tokenize_doctype(&mut self) -> Token {
        // Skip "<!doctype" (9 chars)
        self.advance(9);
        self.consume_whitespace();

        // Read DOCTYPE name
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() || ch == '>' {
                break;
            }
            name.push(ch);
            self.next_char();
        }

        self.consume_whitespace();

        let mut public_id = None;
        let mut system_id = None;

        if self.starts_with_case_insensitive("public") {
            self.advance(6);
            self.consume_whitespace();
            public_id = self.read_quoted_string();
            self.consume_whitespace();
            system_id = self.read_quoted_string();
        } else if self.starts_with_case_insensitive("system") {
            self.advance(6);
            self.consume_whitespace();
            system_id = self.read_quoted_string();
        }

        // Consume until closing '>'
        while let Some(ch) = self.next_char() {
            if ch == '>' {
                break;
            }
        }

        Token::Doctype(DoctypeToken {
            name: name.to_ascii_lowercase(),
            public_id,
            system_id,
            force_quirks: false,
        })
    }

    fn read_quoted_string(&mut self) -> Option<String> {
        let quote = self.peek()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        self.next_char(); // consume opening quote

        let mut string = String::new();
        while let Some(ch) = self.next_char() {
            if ch == quote {
                break;
            }
            string.push(ch);
        }
        Some(string)
    }

    fn tokenize_end_tag(&mut self) -> Token {
        // Skip "</"
        self.advance(2);
        self.consume_whitespace();

        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                name.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        // Consume anything until closing '>'
        while let Some(ch) = self.next_char() {
            if ch == '>' {
                break;
            }
        }

        Token::EndTag(EndTagToken {
            name: name.to_ascii_lowercase(),
        })
    }

    fn tokenize_start_tag(&mut self) -> Token {
        // Skip '<'
        self.next_char();

        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                name.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        let mut attributes = Vec::new();
        let mut self_closing = false;

        loop {
            self.consume_whitespace();
            if self.is_eof() {
                break;
            }

            if self.peek() == Some('>') {
                self.next_char();
                break;
            }

            if self.starts_with("/>") {
                self.advance(2);
                self_closing = true;
                break;
            }

            if self.peek() == Some('/') {
                self.next_char();
                if self.peek() == Some('>') {
                    self.next_char();
                    self_closing = true;
                    break;
                }
                continue;
            }

            // Read attribute name
            let attr_name = self.read_attribute_name();
            if attr_name.is_empty() {
                // Unexpected character; advance to avoid infinite loop
                self.next_char();
                continue;
            }

            self.consume_whitespace();

            let attr_val = if self.peek() == Some('=') {
                self.next_char(); // consume '='
                self.consume_whitespace();
                self.read_attribute_value()
            } else {
                String::new()
            };

            attributes.push((attr_name.to_ascii_lowercase(), decode_entities(&attr_val)));
        }

        Token::StartTag(StartTagToken {
            name: name.to_ascii_lowercase(),
            attributes,
            self_closing,
        })
    }

    fn read_attribute_name(&mut self) -> String {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace()
                || ch == '='
                || ch == '/'
                || ch == '>'
                || ch == '"'
                || ch == '\''
            {
                break;
            }
            name.push(ch);
            self.next_char();
        }
        name
    }

    fn read_attribute_value(&mut self) -> String {
        let mut value = String::new();
        if let Some(quote) = self.peek() {
            if quote == '"' || quote == '\'' {
                self.next_char(); // consume opening quote
                while let Some(ch) = self.next_char() {
                    if ch == quote {
                        break;
                    }
                    value.push(ch);
                }
                return value;
            }
        }

        // Unquoted attribute value
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() || ch == '>' || ch == '/' {
                break;
            }
            value.push(ch);
            self.next_char();
        }
        value
    }

    fn tokenize_text(&mut self) -> Token {
        let mut text = String::new();

        if self.peek() == Some('<') {
            // Stray '<'
            text.push(self.next_char().unwrap());
        }

        while let Some(ch) = self.peek() {
            if ch == '<' {
                break;
            }
            text.push(ch);
            self.next_char();
        }

        Token::Text(decode_entities(&text))
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token::Eof => None,
            token => Some(token),
        }
    }
}
