//! The DOM Document arena holding the entire node tree.

use core::fmt::Write;

use crate::data::{Comment, DocumentType, Element, NodeData, Text};
use crate::error::DomError;
use crate::id::NodeId;
use crate::iter::{AncestorsIter, ChildrenIter, DescendantsIter, ElementChildrenIter};
use crate::node::Node;

/// The central DOM tree structure managing nodes in an arena.
///
/// An instance of `Document` owns all nodes created within it, referenced by stable
/// [`NodeId`] keys. The root node of every `Document` is automatically created with
/// [`NodeData::Document`] at index 0.
#[derive(Debug, Clone)]
pub struct Document {
    nodes: Vec<Node>,
    root: NodeId,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Creates a new, empty DOM document with a root `Document` node.
    #[must_use]
    pub fn new() -> Self {
        let root_id = NodeId::from_raw(0);
        let root_node = Node::new(root_id, NodeData::Document);
        Self {
            nodes: vec![root_node],
            root: root_id,
        }
    }

    /// Returns the root `NodeId` of the document.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Returns the total number of allocated nodes in the arena.
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the node ID exists within this document.
    #[inline]
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        id.index() < self.nodes.len()
    }

    /// Retrieves an immutable reference to a node by its ID.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    /// Retrieves a mutable reference to a node by its ID.
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index())
    }

    // --- Node Allocation ---

    fn allocate_node(&mut self, data: NodeData) -> NodeId {
        let id = NodeId::from_raw(self.nodes.len());
        self.nodes.push(Node::new(id, data));
        id
    }

    /// Creates an [`Element`] node with the given tag name.
    pub fn create_element(&mut self, tag_name: impl Into<String>) -> NodeId {
        self.allocate_node(NodeData::Element(Element::new(tag_name)))
    }

    /// Creates an [`Element`] node with the given tag name and attributes.
    pub fn create_element_with_attributes(
        &mut self,
        tag_name: impl Into<String>,
        attributes: Vec<(String, String)>,
    ) -> NodeId {
        self.allocate_node(NodeData::Element(Element::with_attributes(
            tag_name, attributes,
        )))
    }

    /// Creates a [`Text`] node with the given content.
    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        self.allocate_node(NodeData::Text(Text::new(content)))
    }

    /// Creates a [`Comment`] node with the given text.
    pub fn create_comment(&mut self, content: impl Into<String>) -> NodeId {
        self.allocate_node(NodeData::Comment(Comment::new(content)))
    }

    /// Creates a [`DocumentType`] node (e.g. `<!DOCTYPE html>`).
    pub fn create_doctype(
        &mut self,
        name: impl Into<String>,
        public_id: Option<String>,
        system_id: Option<String>,
    ) -> NodeId {
        self.allocate_node(NodeData::DocumentType(DocumentType::new(
            name, public_id, system_id,
        )))
    }

    // --- Tree Mutation ---

    /// Appends `child` as the last child of `parent`.
    ///
    /// If `child` was previously attached elsewhere in this document, it is first detached.
    ///
    /// # Errors
    /// Returns [`DomError::NodeNotFound`] if either ID is invalid.
    /// Returns [`DomError::RootNodeImmutable`] if attempting to append the root document node.
    /// Returns [`DomError::CycleDetected`] if `child` is an ancestor of `parent` or equal to `parent`.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), DomError> {
        self.validate_mutation(parent, child)?;

        self.detach(child)?;

        let old_last = self.nodes[parent.index()].last_child;
        self.nodes[child.index()].parent = Some(parent);
        self.nodes[child.index()].previous_sibling = old_last;
        self.nodes[child.index()].next_sibling = None;

        if let Some(prev) = old_last {
            self.nodes[prev.index()].next_sibling = Some(child);
        } else {
            self.nodes[parent.index()].first_child = Some(child);
        }

        self.nodes[parent.index()].last_child = Some(child);
        Ok(())
    }

    /// Inserts `child` as a child of `parent` immediately before `reference_child`.
    ///
    /// If `child` was previously attached elsewhere, it is first detached.
    ///
    /// # Errors
    /// Returns [`DomError::NodeNotFound`] if any specified node does not exist.
    /// Returns [`DomError::NotAChild`] if `reference_child` is not a child of `parent`.
    /// Returns [`DomError::CycleDetected`] if `child` is an ancestor of `parent` or equal to `parent`.
    pub fn insert_before(
        &mut self,
        parent: NodeId,
        child: NodeId,
        reference_child: NodeId,
    ) -> Result<(), DomError> {
        self.validate_mutation(parent, child)?;

        if !self.contains(reference_child) {
            return Err(DomError::NodeNotFound(reference_child));
        }

        if self.nodes[reference_child.index()].parent != Some(parent) {
            return Err(DomError::NotAChild {
                parent,
                child: reference_child,
            });
        }

        if child == reference_child {
            return Ok(());
        }

        self.detach(child)?;

        let prev = self.nodes[reference_child.index()].previous_sibling;

        self.nodes[child.index()].parent = Some(parent);
        self.nodes[child.index()].previous_sibling = prev;
        self.nodes[child.index()].next_sibling = Some(reference_child);

        self.nodes[reference_child.index()].previous_sibling = Some(child);

        if let Some(p) = prev {
            self.nodes[p.index()].next_sibling = Some(child);
        } else {
            self.nodes[parent.index()].first_child = Some(child);
        }

        Ok(())
    }

    /// Removes `child` from `parent`.
    ///
    /// # Errors
    /// Returns [`DomError::NotAChild`] if `child` is not a child of `parent`.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), DomError> {
        if !self.contains(parent) {
            return Err(DomError::NodeNotFound(parent));
        }
        if !self.contains(child) {
            return Err(DomError::NodeNotFound(child));
        }

        if self.nodes[child.index()].parent != Some(parent) {
            return Err(DomError::NotAChild { parent, child });
        }

        self.detach(child)
    }

    /// Detaches `node` from its parent and siblings.
    ///
    /// # Errors
    /// Returns [`DomError::RootNodeImmutable`] if attempting to detach the root node.
    /// Returns [`DomError::NodeNotFound`] if `node` does not exist.
    pub fn detach(&mut self, node: NodeId) -> Result<(), DomError> {
        if node == self.root {
            return Err(DomError::RootNodeImmutable);
        }
        if !self.contains(node) {
            return Err(DomError::NodeNotFound(node));
        }

        let Some(parent_id) = self.nodes[node.index()].parent else {
            return Ok(());
        };

        let prev = self.nodes[node.index()].previous_sibling;
        let next = self.nodes[node.index()].next_sibling;

        if let Some(p) = prev {
            self.nodes[p.index()].next_sibling = next;
        } else {
            self.nodes[parent_id.index()].first_child = next;
        }

        if let Some(n) = next {
            self.nodes[n.index()].previous_sibling = prev;
        } else {
            self.nodes[parent_id.index()].last_child = prev;
        }

        self.nodes[node.index()].parent = None;
        self.nodes[node.index()].previous_sibling = None;
        self.nodes[node.index()].next_sibling = None;

        Ok(())
    }

    fn validate_mutation(&self, parent: NodeId, child: NodeId) -> Result<(), DomError> {
        if !self.contains(parent) {
            return Err(DomError::NodeNotFound(parent));
        }
        if !self.contains(child) {
            return Err(DomError::NodeNotFound(child));
        }
        if child == self.root {
            return Err(DomError::RootNodeImmutable);
        }
        if parent == child {
            return Err(DomError::CycleDetected { parent, child });
        }

        // Check if child is an ancestor of parent
        for ancestor in self.ancestors(parent) {
            if ancestor == child {
                return Err(DomError::CycleDetected { parent, child });
            }
        }

        Ok(())
    }

    // --- Traversal Methods ---

    /// Returns the parent of `node`, if any.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).and_then(|n| n.parent())
    }

    /// Returns the first child of `node`, if any.
    #[must_use]
    pub fn first_child(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).and_then(|n| n.first_child())
    }

    /// Returns the last child of `node`, if any.
    #[must_use]
    pub fn last_child(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).and_then(|n| n.last_child())
    }

    /// Returns the previous sibling of `node`, if any.
    #[must_use]
    pub fn previous_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).and_then(|n| n.previous_sibling())
    }

    /// Returns the next sibling of `node`, if any.
    #[must_use]
    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).and_then(|n| n.next_sibling())
    }

    /// Returns an iterator over all direct children of `parent`.
    #[must_use]
    pub fn children(&self, parent: NodeId) -> ChildrenIter<'_> {
        ChildrenIter::new(self, parent)
    }

    /// Returns an iterator over direct element children of `parent`.
    #[must_use]
    pub fn element_children(&self, parent: NodeId) -> ElementChildrenIter<'_> {
        ElementChildrenIter::new(self, parent)
    }

    /// Returns an iterator over ancestors of `start`, moving up to the root.
    #[must_use]
    pub fn ancestors(&self, start: NodeId) -> AncestorsIter<'_> {
        AncestorsIter::new(self, start)
    }

    /// Returns a pre-order depth-first iterator over all descendants of `root`.
    #[must_use]
    pub fn descendants(&self, root: NodeId) -> DescendantsIter<'_> {
        DescendantsIter::new(self, root)
    }

    /// Returns the first direct child of `parent` that is an element.
    #[must_use]
    pub fn first_element_child(&self, parent: NodeId) -> Option<NodeId> {
        self.element_children(parent).next()
    }

    /// Returns the last direct child of `parent` that is an element.
    #[must_use]
    pub fn last_element_child(&self, parent: NodeId) -> Option<NodeId> {
        let mut curr = self.last_child(parent);
        while let Some(id) = curr {
            if let Some(node) = self.get(id) {
                if node.is_element() {
                    return Some(id);
                }
            }
            curr = self.previous_sibling(id);
        }
        None
    }

    /// Returns the nearest previous sibling that is an element.
    #[must_use]
    pub fn previous_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let mut curr = self.previous_sibling(node);
        while let Some(id) = curr {
            if let Some(n) = self.get(id) {
                if n.is_element() {
                    return Some(id);
                }
            }
            curr = self.previous_sibling(id);
        }
        None
    }

    /// Returns the nearest next sibling that is an element.
    #[must_use]
    pub fn next_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let mut curr = self.next_sibling(node);
        while let Some(id) = curr {
            if let Some(n) = self.get(id) {
                if n.is_element() {
                    return Some(id);
                }
            }
            curr = self.next_sibling(id);
        }
        None
    }

    /// Finds an element node by its `id` attribute.
    #[must_use]
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.descendants(self.root).find(|&node_id| {
            self.get(node_id)
                .and_then(|n| n.as_element())
                .is_some_and(|elem| elem.id() == Some(id))
        })
    }

    /// Finds all element nodes with the matching tag name (case-insensitive).
    #[must_use]
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        self.descendants(self.root)
            .filter(|&node_id| {
                self.get(node_id)
                    .and_then(|n| n.as_element())
                    .is_some_and(|elem| elem.tag_name().eq_ignore_ascii_case(tag_name))
            })
            .collect()
    }

    /// Extracts the concatenated textual content of `node` and all its descendants.
    #[must_use]
    pub fn text_content(&self, node: NodeId) -> String {
        let mut result = String::new();
        self.collect_text_content(node, &mut result);
        result
    }

    fn collect_text_content(&self, node: NodeId, out: &mut String) {
        if let Some(node_ref) = self.get(node) {
            match node_ref.data() {
                NodeData::Text(t) => out.push_str(t.content()),
                NodeData::Element(_) | NodeData::Document => {
                    for child in self.children(node) {
                        self.collect_text_content(child, out);
                    }
                }
                _ => {}
            }
        }
    }

    /// Dumps the hierarchical DOM tree as a formatted string for debugging and inspection.
    #[must_use]
    pub fn dump_tree(&self) -> String {
        let mut output = String::new();
        self.dump_node(self.root, 0, &mut output);
        output
    }

    fn dump_node(&self, id: NodeId, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        if let Some(node) = self.get(id) {
            match node.data() {
                NodeData::Document => {
                    let _ = writeln!(out, "{indent}#document");
                }
                NodeData::DocumentType(d) => {
                    let _ = writeln!(out, "{indent}<!DOCTYPE {}>", d.name());
                }
                NodeData::Element(elem) => {
                    let mut attr_str = String::new();
                    for (k, v) in elem.attributes() {
                        let _ = write!(attr_str, " {k}=\"{v}\"");
                    }
                    let _ = writeln!(out, "{indent}<{}{attr_str}>", elem.tag_name());
                }
                NodeData::Text(t) => {
                    let _ = writeln!(out, "{indent}#text {:?}", t.content());
                }
                NodeData::Comment(c) => {
                    let _ = writeln!(out, "{indent}<!-- {:?} -->", c.content());
                }
            }

            for child in self.children(id) {
                self.dump_node(child, depth + 1, out);
            }
        }
    }
}
