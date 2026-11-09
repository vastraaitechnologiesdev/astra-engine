//! Stable identifier for nodes stored within the DOM arena.

use core::fmt;

/// A lightweight, copyable reference to a [`Node`](crate::Node) in a [`Document`](crate::Document).
///
/// Nodes within the Astra DOM tree are managed inside an arena to ensure cache-locality,
/// fast traversal, and avoid reference-counting cycles. A `NodeId` uniquely identifies
/// a node in its containing document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    /// Returns the internal numerical index of the node in the arena.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }

    /// Creates a `NodeId` from a raw index.
    #[inline]
    #[must_use]
    pub(crate) const fn from_raw(index: usize) -> Self {
        Self(index)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}
