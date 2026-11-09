//! Token representations emitted by the Astra HTML tokenizer.

/// Represents a DOCTYPE token (e.g. `<!DOCTYPE html>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctypeToken {
    /// The name declared in the DOCTYPE (lowercased, e.g. `"html"`).
    pub name: String,
    /// The optional public identifier.
    pub public_id: Option<String>,
    /// The optional system identifier.
    pub system_id: Option<String>,
    /// Whether quirks mode is forced.
    pub force_quirks: bool,
}

/// Represents an HTML start tag token (e.g. `<div id="app">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartTagToken {
    /// The tag name (lowercased, e.g. `"div"`, `"h1"`).
    pub name: String,
    /// List of attributes as (name, value) pairs.
    pub attributes: Vec<(String, String)>,
    /// Indicates if the tag used self-closing syntax (e.g. `<br />`).
    pub self_closing: bool,
}

impl StartTagToken {
    /// Retrieves an attribute value by name (case-insensitive).
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// Represents an HTML end tag token (e.g. `</div>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndTagToken {
    /// The tag name (lowercased, e.g. `"div"`).
    pub name: String,
}

/// Fundamental token types produced during HTML tokenization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// DOCTYPE declaration.
    Doctype(DoctypeToken),
    /// Start tag with optional attributes and self-closing flag.
    StartTag(StartTagToken),
    /// End tag.
    EndTag(EndTagToken),
    /// Comment token containing the comment payload.
    Comment(String),
    /// Text token containing character data.
    Text(String),
    /// End-of-file token signaling end of input.
    Eof,
}
