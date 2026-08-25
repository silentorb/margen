use crate::analysis::cell_grouping::{
	check_create_fallback_sector, group_cells, sync_sector_cells,
};
use crate::analysis::headroom_clearance::calculate_headroom_clearance_for_sectors;
use crate::analysis::map_path_depths::map_path_depths;
use crate::analysis::sector_propagation::propagate_sector_cells;
use crate::analysis::types::{AnalysisInput, AnalysisResult, PathDepths, StripMap};
use crate::core::SectorData;
use crate::structure::querying::{get_path_locations, grid_to_neighbors_map, NeighborsMap};

pub fn partition_sectors(input: AnalysisInput) -> AnalysisResult {
	let AnalysisInput {
		mut grid,
		player_start,
		moveable_sector_height_offset,
		config,
	} = input;

	let mut neighbors_map = NeighborsMap::new();
	grid_to_neighbors_map(&grid, &mut neighbors_map);

	let mut path_depths = PathDepths::new();
	map_path_depths(&neighbors_map, player_start, &mut path_depths);

	let mut sector_data = SectorData::default();
	let mut strip_map = StripMap::new();
	let mut fork_locations = Vec::new();

	group_cells(
		&neighbors_map,
		&path_depths,
		&mut strip_map,
		&mut fork_locations,
		&mut sector_data,
		config.cell_group_second_pass,
		config.cell_group_third_pass,
	);

	check_create_fallback_sector(&grid, &mut sector_data);
	sync_sector_cells(&sector_data, &mut grid);

	let path_locations = get_path_locations(&grid);
	propagate_sector_cells(&mut grid, &path_locations);

	calculate_headroom_clearance_for_sectors(
		&grid,
		&mut sector_data.sectors,
		moveable_sector_height_offset,
	);

	AnalysisResult {
		sector_data,
		path_depths,
		grid,
	}
}

#[cfg(test)]
mod integration_tests {
	use super::*;
	use crate::generate_goals;
	use crate::generate_story_grid;
	use crate::validate_sector_map;
	use crate::PartitionConfig;
	use mythic::{Dice, Vector3i};

	#[test]
	fn story_grid_partition_and_goals_produce_sectors_and_switches() {
		let mut dice = Dice::new(42);
		let origin = Vector3i::new(0, 0, 0);
		let story = generate_story_grid(&mut dice, 6, origin);
		assert!(story.success);

		let analysis = partition_sectors(AnalysisInput {
			grid: story.grid,
			player_start: origin,
			moveable_sector_height_offset: 3,
			config: PartitionConfig::default(),
		});

		assert!(!analysis.sector_data.sectors.is_empty());
		assert!(!analysis.path_depths.is_empty());
		assert!(validate_sector_map(&analysis.grid, &analysis.sector_data.sectors).is_empty());

		let mut goal_dice = Dice::new(42);
		let (dag, waypoints, switches) = generate_goals(
			&mut goal_dice,
			&analysis.sector_data,
			&analysis.path_depths,
			0.5,
		);

		assert!(!dag.nodes.is_empty());
		assert_eq!(dag.root, analysis.sector_data.origin);
		if waypoints.len() >= 2 {
			assert!(!switches.is_empty());
			for switch in &switches {
				assert_ne!(switch.source_cell_location, Vector3i::none());
			}
		}
	}
}
