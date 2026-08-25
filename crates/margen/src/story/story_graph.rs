use crate::dependency::DagGraph;
use crate::dependency::DagNodeId;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default)]
pub struct StoryGraph {
	pub graph: DagGraph,
	pub endpoints: BTreeSet<DagNodeId>,
}
