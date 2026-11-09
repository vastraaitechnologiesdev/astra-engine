//! Target DOM insertion location supporting standard appending and foster parenting.

use astra_dom::{Document, DomError, NodeId};

use super::stack::OpenElementStack;

/// Specific insertion location for newly created nodes in the DOM tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionLocation {
    /// Appends the node as the last child of `parent`.
    Append(NodeId),
    /// Inserts the node into `parent` immediately preceding `reference_child`.
    InsertBefore {
        /// The parent node receiving the child.
        parent: NodeId,
        /// The existing child of `parent` before which the new node is spliced.
        reference_child: NodeId,
    },
}

impl InsertionLocation {
    /// Inserts `node` into `document` at this location.
    pub fn insert(self, document: &mut Document, node: NodeId) -> Result<(), DomError> {
        match self {
            Self::Append(parent) => document.append_child(parent, node),
            Self::InsertBefore {
                parent,
                reference_child,
            } => document.insert_before(parent, node, reference_child),
        }
    }

    /// Calculates the foster parenting target location.
    ///
    /// In HTML5, foster parenting places content into the parent of the last `table`
    /// element in the open element stack, before that `table` element.
    /// If an element has already been foster-parented and is currently on top of the
    /// stack (such as `<b>`), content is appended directly into it.
    #[must_use]
    pub fn calculate_foster_parenting(stack: &OpenElementStack, document: &Document) -> Self {
        let current = stack.current_node();

        // If the current node on the stack is a foster-parented element (e.g. <b> or <span>),
        // append children into it.
        if let Some(node) = document.get(current) {
            if let Some(elem) = node.as_element() {
                if !matches!(
                    elem.tag_name(),
                    "table" | "tbody" | "thead" | "tfoot" | "tr" | "colgroup" | "caption"
                ) {
                    return Self::Append(current);
                }
            }
        }

        if let Some(table_id) = stack.find_last_tag("table", document) {
            if let Some(table_parent) = document.parent(table_id) {
                return Self::InsertBefore {
                    parent: table_parent,
                    reference_child: table_id,
                };
            }

            // If table has no parent, insert into the stack element preceding the table
            if let Some(pos) = stack.find_position(table_id) {
                if pos > 0 {
                    return Self::Append(stack.elements()[pos - 1]);
                }
            }
        }

        // Fallback to appending to the current node
        Self::Append(current)
    }
}
