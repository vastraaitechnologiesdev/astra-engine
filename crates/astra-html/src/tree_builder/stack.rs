//! Open element stack for tracking HTML5 tree nesting and element scopes.

use astra_dom::{Document, NodeId};

/// The stack of open elements maintained during HTML tree construction.
#[derive(Debug, Clone, Default)]
pub struct OpenElementStack {
    elements: Vec<NodeId>,
}

impl OpenElementStack {
    /// Creates a new open element stack initialized with the document root.
    #[must_use]
    pub fn new(root: NodeId) -> Self {
        Self {
            elements: vec![root],
        }
    }

    /// Returns the element slice.
    #[must_use]
    pub fn elements(&self) -> &[NodeId] {
        &self.elements
    }

    /// Pushes a node onto the top of the stack.
    pub fn push(&mut self, id: NodeId) {
        self.elements.push(id);
    }

    /// Pops the topmost node from the stack.
    pub fn pop(&mut self) -> Option<NodeId> {
        if self.elements.len() > 1 {
            self.elements.pop()
        } else {
            None
        }
    }

    /// Returns the topmost node on the stack.
    #[must_use]
    pub fn current_node(&self) -> NodeId {
        *self.elements.last().expect("Stack must never be empty")
    }

    /// Returns the number of nodes currently on the stack.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the stack contains only the document root.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.len() <= 1
    }

    /// Checks if a node is currently in the stack.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.elements.contains(&id)
    }

    /// Finds the index of a node in the stack.
    #[must_use]
    pub fn find_position(&self, id: NodeId) -> Option<usize> {
        self.elements.iter().position(|&x| x == id)
    }

    /// Finds the last node in the stack matching the given tag name.
    #[must_use]
    pub fn find_last_tag(&self, tag_name: &str, document: &Document) -> Option<NodeId> {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name().eq_ignore_ascii_case(tag_name) {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    /// Checks if a tag is present in the stack.
    #[must_use]
    pub fn contains_tag(&self, tag_name: &str, document: &Document) -> bool {
        self.find_last_tag(tag_name, document).is_some()
    }

    /// Pops elements from the stack until an element with `tag_name` is popped.
    pub fn pop_until(&mut self, tag_name: &str, document: &Document) -> Option<NodeId> {
        while self.elements.len() > 1 {
            let top_id = self.elements.pop()?;
            if let Some(node) = document.get(top_id) {
                if let Some(elem) = node.as_element() {
                    if elem.tag_name().eq_ignore_ascii_case(tag_name) {
                        return Some(top_id);
                    }
                }
            }
        }
        None
    }

    /// Checks if an element with `tag_name` is in standard scope.
    #[must_use]
    pub fn has_element_in_scope(&self, tag_name: &str, document: &Document) -> bool {
        self.has_element_in_specific_scope(
            tag_name,
            document,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
            ],
        )
    }

    /// Checks if an element with `tag_name` is in button scope.
    #[must_use]
    pub fn has_element_in_button_scope(&self, tag_name: &str, document: &Document) -> bool {
        self.has_element_in_specific_scope(
            tag_name,
            document,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
                "button",
            ],
        )
    }

    /// Checks if a `<p>` element is in button scope.
    #[must_use]
    pub fn has_p_in_button_scope(&self, document: &Document) -> bool {
        self.has_element_in_button_scope("p", document)
    }

    /// Checks if an element is in table scope.
    #[must_use]
    pub fn has_element_in_table_scope(&self, tag_name: &str, document: &Document) -> bool {
        self.has_element_in_specific_scope(tag_name, document, &["html", "table", "template"])
    }

    /// Checks if a heading (`h1` through `h6`) is currently in scope.
    #[must_use]
    pub fn has_heading_in_scope(&self, document: &Document) -> bool {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                        return true;
                    }
                    if matches!(
                        name,
                        "applet"
                            | "caption"
                            | "html"
                            | "table"
                            | "td"
                            | "th"
                            | "marquee"
                            | "object"
                            | "template"
                    ) {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Pops the topmost heading (`h1` through `h6`) in scope, if any.
    pub fn pop_heading_in_scope(&mut self, document: &Document) -> bool {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                        let name_owned = name.to_string();
                        self.pop_until(&name_owned, document);
                        return true;
                    }
                    if matches!(
                        name,
                        "applet"
                            | "caption"
                            | "html"
                            | "table"
                            | "td"
                            | "th"
                            | "marquee"
                            | "object"
                            | "template"
                    ) {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Checks if an `li` element is in list item scope.
    #[must_use]
    pub fn has_li_in_list_scope(&self, document: &Document) -> bool {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if name.eq_ignore_ascii_case("li") {
                        return true;
                    }
                    if matches!(
                        name,
                        "ol" | "ul"
                            | "applet"
                            | "caption"
                            | "html"
                            | "table"
                            | "td"
                            | "th"
                            | "marquee"
                            | "object"
                            | "template"
                    ) {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Checks if a `dt` or `dd` element is in scope.
    #[must_use]
    pub fn has_dt_or_dd_in_scope(&self, document: &Document) -> bool {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if name.eq_ignore_ascii_case("dt") || name.eq_ignore_ascii_case("dd") {
                        return true;
                    }
                    if matches!(
                        name,
                        "dl" | "applet"
                            | "caption"
                            | "html"
                            | "table"
                            | "td"
                            | "th"
                            | "marquee"
                            | "object"
                            | "template"
                    ) {
                        return false;
                    }
                }
            }
        }
        false
    }

    fn has_element_in_specific_scope(
        &self,
        tag_name: &str,
        document: &Document,
        scopers: &[&str],
    ) -> bool {
        for &id in self.elements.iter().rev() {
            if let Some(node) = document.get(id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if name.eq_ignore_ascii_case(tag_name) {
                        return true;
                    }
                    for &scoper in scopers {
                        if name.eq_ignore_ascii_case(scoper) {
                            return false;
                        }
                    }
                }
            }
        }
        false
    }

    /// Clears stack back to table context (pops until top is `table`, `template`, or `html`).
    pub fn clear_back_to_table_context(&mut self, document: &Document) {
        while self.elements.len() > 1 {
            let top_id = self.current_node();
            if let Some(node) = document.get(top_id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if matches!(name, "table" | "template" | "html") {
                        break;
                    }
                }
            }
            self.elements.pop();
        }
    }

    /// Clears stack back to table body context (pops until top is `tbody`, `thead`, `tfoot`, `template`, or `html`).
    pub fn clear_back_to_table_body_context(&mut self, document: &Document) {
        while self.elements.len() > 1 {
            let top_id = self.current_node();
            if let Some(node) = document.get(top_id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if matches!(name, "tbody" | "thead" | "tfoot" | "template" | "html") {
                        break;
                    }
                }
            }
            self.elements.pop();
        }
    }

    /// Clears stack back to table row context (pops until top is `tr`, `template`, or `html`).
    pub fn clear_back_to_table_row_context(&mut self, document: &Document) {
        while self.elements.len() > 1 {
            let top_id = self.current_node();
            if let Some(node) = document.get(top_id) {
                if let Some(elem) = node.as_element() {
                    let name = elem.tag_name();
                    if matches!(name, "tr" | "template" | "html") {
                        break;
                    }
                }
            }
            self.elements.pop();
        }
    }
}
