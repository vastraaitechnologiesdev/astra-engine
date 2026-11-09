//! Node payloads and concrete DOM node types.

/// Represents a DOCTYPE declaration in the DOM tree (e.g. `<!DOCTYPE html>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentType {
    name: String,
    public_id: Option<String>,
    system_id: Option<String>,
}

impl DocumentType {
    /// Creates a new `DocumentType` instance.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        public_id: Option<String>,
        system_id: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            public_id,
            system_id,
        }
    }

    /// The name of the DOCTYPE (e.g., `"html"`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The optional public identifier.
    #[must_use]
    pub fn public_id(&self) -> Option<&str> {
        self.public_id.as_deref()
    }

    /// The optional system identifier.
    #[must_use]
    pub fn system_id(&self) -> Option<&str> {
        self.system_id.as_deref()
    }
}

/// Represents an HTML element in the DOM tree (e.g. `<div>`, `<span>`, `<body>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    tag_name: String,
    attributes: Vec<(String, String)>,
}

impl Element {
    /// Creates a new `Element` with the specified tag name and no attributes.
    #[must_use]
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: Vec::new(),
        }
    }

    /// Creates a new `Element` with the specified tag name and attributes.
    #[must_use]
    pub fn with_attributes(tag_name: impl Into<String>, attributes: Vec<(String, String)>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes,
        }
    }

    /// Returns the tag name of the element (e.g. `"div"`, `"h1"`).
    #[must_use]
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Sets the tag name of the element.
    pub fn set_tag_name(&mut self, tag_name: impl Into<String>) {
        self.tag_name = tag_name.into();
    }

    /// Returns the attributes slice as key-value pairs.
    #[must_use]
    pub fn attributes(&self) -> &[(String, String)] {
        &self.attributes
    }

    /// Returns a mutable reference to the attributes vector.
    pub fn attributes_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.attributes
    }

    /// Returns the value of the attribute with the given name, if present.
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Sets the attribute `name` to `value`. Overwrites if it already exists.
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name_str = name.into();
        let value_str = value.into();
        if let Some(existing) = self
            .attributes
            .iter_mut()
            .find(|(k, _)| k.eq_ignore_ascii_case(&name_str))
        {
            existing.1 = value_str;
        } else {
            self.attributes.push((name_str, value_str));
        }
    }

    /// Removes an attribute by name, returning its old value if it was present.
    pub fn remove_attribute(&mut self, name: &str) -> Option<String> {
        if let Some(pos) = self
            .attributes
            .iter()
            .position(|(k, _)| k.eq_ignore_ascii_case(name))
        {
            Some(self.attributes.remove(pos).1)
        } else {
            None
        }
    }

    /// Checks whether an attribute exists on the element.
    #[must_use]
    pub fn has_attribute(&self, name: &str) -> bool {
        self.get_attribute(name).is_some()
    }

    /// Returns the `id` attribute of the element, if present.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.get_attribute("id")
    }

    /// Returns the `class` attribute of the element, if present.
    #[must_use]
    pub fn class_name(&self) -> Option<&str> {
        self.get_attribute("class")
    }

    /// Returns an iterator over the CSS class tokens from the `class` attribute.
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.class_name()
            .unwrap_or_default()
            .split_ascii_whitespace()
    }
}

/// Represents a text node containing textual character data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    content: String,
}

impl Text {
    /// Creates a new `Text` node with the specified string content.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    /// Returns the string content of this text node.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns a mutable reference to the string content.
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }

    /// Replaces the text content.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Appends additional text to the current content.
    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }
}

/// Represents an HTML comment node (e.g. `<!-- a comment -->`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    content: String,
}

impl Comment {
    /// Creates a new `Comment` node with the specified content.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    /// Returns the content string of this comment.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns a mutable reference to the comment string content.
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }

    /// Replaces the comment content.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
}

/// The specific data associated with a DOM [`Node`](crate::Node).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
    /// Root document node.
    Document,
    /// A DOCTYPE node declaration.
    DocumentType(DocumentType),
    /// An element node with tag and attributes.
    Element(Element),
    /// A text node with raw text content.
    Text(Text),
    /// A comment node.
    Comment(Comment),
}

impl NodeData {
    /// Returns the element reference if this node is an `Element`.
    #[must_use]
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            Self::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns a mutable element reference if this node is an `Element`.
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self {
            Self::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the text reference if this node is a `Text` node.
    #[must_use]
    pub fn as_text(&self) -> Option<&Text> {
        match self {
            Self::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Returns a mutable text reference if this node is a `Text` node.
    pub fn as_text_mut(&mut self) -> Option<&mut Text> {
        match self {
            Self::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Returns the comment reference if this node is a `Comment` node.
    #[must_use]
    pub fn as_comment(&self) -> Option<&Comment> {
        match self {
            Self::Comment(c) => Some(c),
            _ => None,
        }
    }

    /// Returns the doctype reference if this node is a `DocumentType` node.
    #[must_use]
    pub fn as_doctype(&self) -> Option<&DocumentType> {
        match self {
            Self::DocumentType(d) => Some(d),
            _ => None,
        }
    }
}
