//! Core Node structure stored within the DOM arena.

use crate::data::{Comment, DocumentType, Element, NodeData, Text};
use crate::id::NodeId;

/// A node within the DOM arena.
///
/// Contains the payload [`NodeData`] as well as structural link pointers
/// (`parent`, `first_child`, `last_child`, `previous_sibling`, `next_sibling`)
/// which form the hierarchical DOM tree without reference-counting cycles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub(crate) id: NodeId,
    pub(crate) data: NodeData,
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
}

impl Node {
    /// Constructs a new node with the given ID and data, detached from the tree.
    #[must_use]
    pub(crate) fn new(id: NodeId, data: NodeData) -> Self {
        Self {
            id,
            data,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
        }
    }

    /// The stable ID of this node.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    /// Reference to the underlying node data.
    #[inline]
    #[must_use]
    pub const fn data(&self) -> &NodeData {
        &self.data
    }

    /// Mutable reference to the underlying node data.
    #[inline]
    #[must_use]
    pub fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    /// The parent node's ID, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// The first child's ID, if any.
    #[inline]
    #[must_use]
    pub const fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    /// The last child's ID, if any.
    #[inline]
    #[must_use]
    pub const fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    /// The previous sibling's ID, if any.
    #[inline]
    #[must_use]
    pub const fn previous_sibling(&self) -> Option<NodeId> {
        self.previous_sibling
    }

    /// The next sibling's ID, if any.
    #[inline]
    #[must_use]
    pub const fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    /// Returns `true` if this is the root document node.
    #[inline]
    #[must_use]
    pub const fn is_document(&self) -> bool {
        matches!(self.data, NodeData::Document)
    }

    /// Returns `true` if this is a DOCTYPE node.
    #[inline]
    #[must_use]
    pub const fn is_doctype(&self) -> bool {
        matches!(self.data, NodeData::DocumentType(_))
    }

    /// Returns `true` if this is an element node.
    #[inline]
    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    /// Returns `true` if this is a text node.
    #[inline]
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self.data, NodeData::Text(_))
    }

    /// Returns `true` if this is a comment node.
    #[inline]
    #[must_use]
    pub const fn is_comment(&self) -> bool {
        matches!(self.data, NodeData::Comment(_))
    }

    /// Returns a reference to the `Element` payload, if this node is an element.
    #[inline]
    #[must_use]
    pub fn as_element(&self) -> Option<&Element> {
        self.data.as_element()
    }

    /// Returns a mutable reference to the `Element` payload, if this node is an element.
    #[inline]
    #[must_use]
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        self.data.as_element_mut()
    }

    /// Returns a reference to the `Text` payload, if this node is a text node.
    #[inline]
    #[must_use]
    pub fn as_text(&self) -> Option<&Text> {
        self.data.as_text()
    }

    /// Returns a mutable reference to the `Text` payload, if this node is a text node.
    #[inline]
    #[must_use]
    pub fn as_text_mut(&mut self) -> Option<&mut Text> {
        self.data.as_text_mut()
    }

    /// Returns a reference to the `Comment` payload, if this node is a comment.
    #[inline]
    #[must_use]
    pub fn as_comment(&self) -> Option<&Comment> {
        self.data.as_comment()
    }

    /// Returns a reference to the `DocumentType` payload, if this node is a DOCTYPE.
    #[inline]
    #[must_use]
    pub fn as_doctype(&self) -> Option<&DocumentType> {
        self.data.as_doctype()
    }
}
