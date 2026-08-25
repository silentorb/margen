use crate::analysis::PathDepths;
use crate::core::{LevelSwitchEntry, Sector, SectorData, SectorId};
use crate::dependency::DagNodeId;
use crate::graphing::{sectors_to_dag, SimpleDagGraph};
use mythic::{select_many_ordered, shuffle_vec, Dice, Vector3i};
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

fn get_sector_cells_with_least_depth(depths: &PathDepths, sector: &Sector) -> Vec<Vector3i> {
	let mut result = Vec::new();
	let mut lowest_depth = 100_000;

	for &location in &sector.cells {
		let depth = depths.get(&location).copied().unwrap_or(i32::MAX);
		if depth < lowest_depth {
			lowest_depth = depth;
			result.clear();
			result.push(location);
		} else if depth == lowest_depth {
			result.push(location);
		}
	}

	result
}

fn select_switch_cell_location(dice: &mut Dice, depths: &PathDepths, sector: &Sector) -> Vector3i {
	if sector.cells.is_empty() {
		return Vector3i::none();
	}

	let earliest_cells = get_sector_cells_with_least_depth(depths, sector);
	dice.from_slice(&earliest_cells)
}

pub fn select_switch_cell_locations(
	dice: &mut Dice,
	sectors: &HashMap<SectorId, Sector>,
	depths: &PathDepths,
	switches: &mut [LevelSwitchEntry],
) {
	for entry in switches {
		if let Some(sector) = sectors.get(&entry.source_sector) {
			entry.source_cell_location = select_switch_cell_location(dice, depths, sector);
		}
	}
}

pub fn generate_goals(
	dice: &mut Dice,
	sector_data: &SectorData,
	path_depths: &PathDepths,
	gate_ratio: f32,
) -> (SimpleDagGraph, Vec<DagNodeId>, Vec<LevelSwitchEntry>) {
	let mut graph = SimpleDagGraph {
		root: sector_data.origin,
		..Default::default()
	};
	sectors_to_dag(sector_data, &mut graph);

	let waypoints = create_gated_waypoints(dice, &graph, gate_ratio);
	let mut switches = waypoints_to_switches(&waypoints);
	select_switch_cell_locations(dice, &sector_data.sectors, path_depths, &mut switches);

	(graph, waypoints, switches)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::core::Sector;
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

	#[test]
	fn select_switch_cell_locations_picks_min_depth_cell() {
		let mut depths = PathDepths::new();
		depths.insert(Vector3i::new(0, 0, 0), 2);
		depths.insert(Vector3i::new(1, 0, 0), 0);

		let sector = Sector {
			id: 1,
			cells: vec![Vector3i::new(0, 0, 0), Vector3i::new(1, 0, 0)],
			..Default::default()
		};
		let sectors = HashMap::from([(1_i16, sector)]);
		let mut switches = vec![LevelSwitchEntry::new(1, 2)];

		let mut dice = Dice::new(99);
		select_switch_cell_locations(&mut dice, &sectors, &depths, &mut switches);
		assert_eq!(switches[0].source_cell_location, Vector3i::new(1, 0, 0));
	}
}
