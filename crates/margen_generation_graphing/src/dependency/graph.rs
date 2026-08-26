use std::collections::{BTreeMap, BTreeSet};

pub type DagNodeId = i16;
pub type DagConnectionAttributes = u32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DagConnection {
	pub attributes: DagConnectionAttributes,
}

impl DagConnection {
	pub fn new(attributes: DagConnectionAttributes) -> Self {
		Self { attributes }
	}

	pub fn has_attribute(self, attribute: DagConnectionAttributes) -> bool {
		(self.attributes & attribute) != 0
	}
}

/// Directed edge identity: (from, to).
pub type DagConnectionId = (DagNodeId, DagNodeId);

/// Ordered map so pipeline steps that iterate connections are deterministic.
pub type DagConnectionMap = BTreeMap<DagConnectionId, DagConnection>;

#[derive(Clone, Debug, Default)]
pub struct DagGraph {
	pub nodes: BTreeSet<DagNodeId>,
	pub connections: DagConnectionMap,
	pub root: DagNodeId,
}
