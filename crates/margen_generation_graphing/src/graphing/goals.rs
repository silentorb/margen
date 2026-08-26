use crate::dependency::DagNodeId;
use crate::graphing::SimpleDagGraph;
use margen_generation::LevelSwitchEntry;
use mythic_dice::Dice;
use mythic_distribution::{select_many_ordered, shuffle_vec};
use std::collections::HashMap;

fn get_max_depth(depths: &HashMap<DagNodeId, i32>) -> i32 {
	depths.values().copied().max().unwrap_or(-1)
}

fn order_nodes_by_depth(depths: &HashMap<DagNodeId, i32>) -> Vec<Vec<DagNodeId>> {
	let max_depth = get_max_depth(depths);
	let mut layers = vec![Vec::new(); (max_depth + 1) as usize];
	for (&node_id, &depth) in depths {
		layers[depth as usize].push(node_id);
	}
	layers
}

fn get_endpoints(graph: &SimpleDagGraph) -> HashMap<DagNodeId, i32> {
	let mut output = HashMap::new();
	for (&node_id, node) in &graph.nodes {
		if node_id != graph.root && node.get_outputs().is_empty() {
			output.insert(node_id, node.depth);
		}
	}
	output
}

fn flatten_shuffled_node_layers(dice: &mut Dice, layers: &[Vec<DagNodeId>]) -> Vec<DagNodeId> {
	let mut output = Vec::new();
	for layer in layers {
		output.extend(shuffle_vec(dice, layer));
	}
	output
}

pub fn create_gated_waypoints(
	dice: &mut Dice,
	graph: &SimpleDagGraph,
	gate_ratio: f32,
) -> Vec<DagNodeId> {
	let endpoint_depths = get_endpoints(graph);
	let endpoint_layers = order_nodes_by_depth(&endpoint_depths);
	let mut waypoints = flatten_shuffled_node_layers(dice, &endpoint_layers);
	let max_waypoints = (gate_ratio * waypoints.len() as f32).floor() as i32;

	if max_waypoints < 2 {
		Vec::new()
	} else {
		select_many_ordered(dice, max_waypoints as usize, &mut waypoints);
		waypoints
	}
}

pub fn waypoints_to_switches(waypoints: &[DagNodeId]) -> Vec<LevelSwitchEntry> {
	if waypoints.len() < 2 {
		return Vec::new();
	}

	(0..waypoints.len() - 1)
		.map(|i| LevelSwitchEntry::new(waypoints[i], waypoints[i + 1]))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::graphing::SimpleDagNode;

	#[test]
	fn waypoints_to_switches_pairs_consecutive_nodes() {
		let switches = waypoints_to_switches(&[1, 2, 4]);
		assert_eq!(switches.len(), 2);
		assert_eq!(switches[0].source_sector, 1);
		assert_eq!(switches[0].target_sector, 2);
		assert_eq!(switches[1].source_sector, 2);
		assert_eq!(switches[1].target_sector, 4);
	}

	#[test]
	fn create_gated_waypoints_empty_when_ratio_too_low() {
		let mut graph = SimpleDagGraph {
			root: 1,
			nodes: HashMap::from([
				(1, SimpleDagNode { depth: 0, ..Default::default() }),
				(2, SimpleDagNode { depth: 1, ..Default::default() }),
			]),
			..Default::default()
		};
		graph.nodes.get_mut(&2).unwrap().connections[SimpleDagNode::INPUT_INDEX].push(1);

		let mut dice = Dice::new(3);
		let waypoints = create_gated_waypoints(&mut dice, &graph, 0.1);
		assert!(waypoints.is_empty());
	}
}
