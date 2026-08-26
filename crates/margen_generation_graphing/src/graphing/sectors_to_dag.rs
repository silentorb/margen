use margen_generation::SectorData;
use crate::dependency::DagNodeId;
use crate::graphing::simple_dag::{SimpleDagConnections, SimpleDagGraph, SimpleDagNode};
use std::collections::VecDeque;

fn contains_connection(container: &SimpleDagConnections, id: DagNodeId) -> bool {
	container.contains(&id)
}

fn try_add_neighbor_node(
	node_id: DagNodeId,
	node: &mut SimpleDagNode,
	neighbor_id: DagNodeId,
	neighbor: &mut SimpleDagNode,
	container_index: usize,
) {
	let container = &mut node.connections[container_index];
	if !contains_connection(container, neighbor_id) {
		let neighbor_container = &mut neighbor.connections[1 - container_index];
		container.push(neighbor_id);
		neighbor_container.push(node_id);
	}
}

pub fn sectors_to_dag(sector_data: &SectorData, graph: &mut SimpleDagGraph) {
	if sector_data.sectors.is_empty() {
		return;
	}

	let mut queue = VecDeque::new();
	queue.push_back(sector_data.origin);
	graph
		.nodes
		.insert(sector_data.origin, SimpleDagNode::default());

	while let Some(sector_id) = queue.pop_front() {
		let neighbors = sector_data.sectors[&sector_id].neighbors.clone();
		let node_depth = graph.nodes[&sector_id].depth;

		for neighbor_id in neighbors {
			if !graph.nodes.contains_key(&neighbor_id) {
				graph.nodes.insert(
					neighbor_id,
					SimpleDagNode {
						depth: node_depth + 1,
						..Default::default()
					},
				);
				queue.push_back(neighbor_id);
			} else {
				let neighbor_depth = graph.nodes[&neighbor_id].depth;
				if neighbor_depth != node_depth {
					let container_index = if neighbor_depth > node_depth {
						SimpleDagNode::OUTPUT_INDEX
					} else {
						SimpleDagNode::INPUT_INDEX
					};

					let mut node = graph.nodes.remove(&sector_id).expect("sector present");
					let mut neighbor = graph.nodes.remove(&neighbor_id).expect("neighbor present");
					try_add_neighbor_node(
						sector_id,
						&mut node,
						neighbor_id,
						&mut neighbor,
						container_index,
					);
					graph.nodes.insert(sector_id, node);
					graph.nodes.insert(neighbor_id, neighbor);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_generation::Sector;

	#[test]
	fn builds_layered_connections() {
		let mut sector_data = SectorData::default();
		sector_data.origin = 1;
		sector_data.sectors.insert(
			1,
			Sector {
				id: 1,
				neighbors: vec![2],
				..Default::default()
			},
		);
		sector_data.sectors.insert(
			2,
			Sector {
				id: 2,
				neighbors: vec![1, 3],
				..Default::default()
			},
		);
		sector_data.sectors.insert(
			3,
			Sector {
				id: 3,
				neighbors: vec![2],
				..Default::default()
			},
		);

		let mut graph = SimpleDagGraph::default();
		sectors_to_dag(&sector_data, &mut graph);

		assert_eq!(graph.nodes[&1].depth, 0);
		assert_eq!(graph.nodes[&2].depth, 1);
		assert_eq!(graph.nodes[&3].depth, 2);
		assert_eq!(graph.nodes[&1].get_outputs(), &vec![2]);
		assert_eq!(graph.nodes[&2].get_inputs(), &vec![1]);
	}
}
