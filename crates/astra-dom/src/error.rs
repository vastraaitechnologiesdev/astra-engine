//! Error types encountered during DOM operations.

use core::fmt;

use crate::NodeId;

/// Errors that can occur when manipulating the DOM tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomError {
    /// The specified node was not found within the document arena.
    NodeNotFound(NodeId),
    /// Attempted to insert a node as a descendant of itself, creating a cyclic graph.
    CycleDetected {
        /// The target parent node.
        parent: NodeId,
        /// The child node causing a cycle.
        child: NodeId,
    },
    /// The root document node cannot be attached, inserted, or detached.
    RootNodeImmutable,
    /// The given reference child is not a child of the specified parent.
    NotAChild {
        /// The expected parent node.
        parent: NodeId,
        /// The node that is not a child of `parent`.
        child: NodeId,
    },
    /// General invalid hierarchy or node placement error.
    InvalidHierarchy(String),
}

impl fmt::Display for DomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node {id} not found in document arena"),
            Self::CycleDetected { parent, child } => {
                write!(
                    f,
                    "Cannot insert node {child} into {parent}: cycle detected"
                )
            }
            Self::RootNodeImmutable => {
                write!(f, "The root document node cannot be modified in hierarchy")
            }
            Self::NotAChild { parent, child } => {
                write!(f, "Node {child} is not a child of node {parent}")
            }
            Self::InvalidHierarchy(msg) => write!(f, "Invalid DOM hierarchy: {msg}"),
        }
    }
}

impl std::error::Error for DomError {}
