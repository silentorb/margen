use crate::types::PathDepths;
use margen_generation::{LevelSwitchEntry, Sector, SectorData, SectorId};
use margen_generation_graphing::{
	create_gated_waypoints, sectors_to_dag, waypoints_to_switches, DagNodeId, SimpleDagGraph,
};
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::collections::HashMap;

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
	use margen_generation::Sector;

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
