//! Active formatting elements list supporting markers and reconstruction.

use astra_dom::NodeId;

/// An entry in the list of active formatting elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormattingEntry {
    /// An active formatting element node with its tag name.
    Element {
        /// Node ID in the document arena.
        node_id: NodeId,
        /// Element tag name.
        tag_name: String,
    },
    /// A scope boundary marker (e.g. entering a table cell or caption).
    Marker,
}

/// The list of active formatting elements.
#[derive(Debug, Clone, Default)]
pub struct ActiveFormattingElements {
    entries: Vec<FormattingEntry>,
}

impl ActiveFormattingElements {
    /// Creates a new empty list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Pushes a formatting element onto the list.
    pub fn push_element(&mut self, node_id: NodeId, tag_name: impl Into<String>) {
        self.entries.push(FormattingEntry::Element {
            node_id,
            tag_name: tag_name.into(),
        });
    }

    /// Pushes a marker onto the list.
    pub fn push_marker(&mut self) {
        self.entries.push(FormattingEntry::Marker);
    }

    /// Clears all entries back to and including the last marker.
    pub fn clear_to_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            if matches!(entry, FormattingEntry::Marker) {
                break;
            }
        }
    }

    /// Removes an element by its [`NodeId`].
    pub fn remove(&mut self, node_id: NodeId) {
        if let Some(pos) = self.entries.iter().position(|e| match e {
            FormattingEntry::Element { node_id: id, .. } => *id == node_id,
            FormattingEntry::Marker => false,
        }) {
            self.entries.remove(pos);
        }
    }

    /// Finds the last formatting element with the given tag name after the last marker.
    #[must_use]
    pub fn find_last_after_marker(&self, tag_name: &str) -> Option<NodeId> {
        for entry in self.entries.iter().rev() {
            match entry {
                FormattingEntry::Marker => break,
                FormattingEntry::Element {
                    node_id,
                    tag_name: name,
                } => {
                    if name.eq_ignore_ascii_case(tag_name) {
                        return Some(*node_id);
                    }
                }
            }
        }
        None
    }

    /// Returns the entries slice.
    #[must_use]
    pub fn entries(&self) -> &[FormattingEntry] {
        &self.entries
    }
}
