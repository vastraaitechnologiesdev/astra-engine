//! DOM tree traversal iterators.

use crate::document::Document;
use crate::id::NodeId;

/// An iterator over the direct children of a node.
#[derive(Debug, Clone)]
pub struct ChildrenIter<'a> {
    document: &'a Document,
    current: Option<NodeId>,
}

impl<'a> ChildrenIter<'a> {
    pub(crate) fn new(document: &'a Document, parent: NodeId) -> Self {
        let first_child = document.get(parent).and_then(|n| n.first_child());
        Self {
            document,
            current: first_child,
        }
    }
}

impl Iterator for ChildrenIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.current?;
        self.current = self.document.get(id).and_then(|n| n.next_sibling());
        Some(id)
    }
}

/// An iterator over the direct element children of a node.
#[derive(Debug, Clone)]
pub struct ElementChildrenIter<'a> {
    inner: ChildrenIter<'a>,
}

impl<'a> ElementChildrenIter<'a> {
    pub(crate) fn new(document: &'a Document, parent: NodeId) -> Self {
        Self {
            inner: ChildrenIter::new(document, parent),
        }
    }
}

impl Iterator for ElementChildrenIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(child_id) = self.inner.next() {
            if let Some(node) = self.inner.document.get(child_id) {
                if node.is_element() {
                    return Some(child_id);
                }
            }
        }
        None
    }
}

/// An iterator over the ancestors of a node, starting from its parent up to the root.
#[derive(Debug, Clone)]
pub struct AncestorsIter<'a> {
    document: &'a Document,
    current: Option<NodeId>,
}

impl<'a> AncestorsIter<'a> {
    pub(crate) fn new(document: &'a Document, start: NodeId) -> Self {
        let parent = document.get(start).and_then(|n| n.parent());
        Self {
            document,
            current: parent,
        }
    }
}

impl Iterator for AncestorsIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.current?;
        self.current = self.document.get(id).and_then(|n| n.parent());
        Some(id)
    }
}

/// A pre-order depth-first traversal iterator over all descendants of a node.
#[derive(Debug, Clone)]
pub struct DescendantsIter<'a> {
    document: &'a Document,
    stack: Vec<NodeId>,
}

impl<'a> DescendantsIter<'a> {
    pub(crate) fn new(document: &'a Document, root: NodeId) -> Self {
        // Collect children in reverse order so we pop left-most first
        let mut initial_children = Vec::new();
        let mut curr = document.get(root).and_then(|n| n.first_child());
        while let Some(child_id) = curr {
            initial_children.push(child_id);
            curr = document.get(child_id).and_then(|n| n.next_sibling());
        }
        initial_children.reverse();

        Self {
            document,
            stack: initial_children,
        }
    }
}

impl Iterator for DescendantsIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let next_id = self.stack.pop()?;

        // Push children of this node in reverse order
        let mut children = Vec::new();
        let mut curr = self.document.get(next_id).and_then(|n| n.first_child());
        while let Some(child_id) = curr {
            children.push(child_id);
            curr = self.document.get(child_id).and_then(|n| n.next_sibling());
        }
        children.reverse();
        self.stack.extend(children);

        Some(next_id)
    }
}
