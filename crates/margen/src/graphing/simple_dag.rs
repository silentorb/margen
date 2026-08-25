use crate::dependency::DagNodeId;
use std::collections::HashMap;

pub type SimpleDagConnections = Vec<DagNodeId>;

#[derive(Clone, Debug, Default)]
pub struct SimpleDagNode {
	pub depth: i32,
	pub connections: [SimpleDagConnections; 2],
	pub type_id: i32,
}

impl SimpleDagNode {
	pub const INPUT_INDEX: usize = 0;
	pub const OUTPUT_INDEX: usize = 1;

	pub fn get_inputs(&self) -> &SimpleDagConnections {
		&self.connections[Self::INPUT_INDEX]
	}

	pub fn get_outputs(&self) -> &SimpleDagConnections {
		&self.connections[Self::OUTPUT_INDEX]
	}
}

#[derive(Clone, Debug, Default)]
pub struct SimpleDagGraph {
	pub nodes: HashMap<DagNodeId, SimpleDagNode>,
	pub root: DagNodeId,
}
