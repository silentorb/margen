use crate::winding_path::congruence::is_path_location;
use crate::winding_path::types::{CellPriority, SolvingState};
use margen_generation::biomes::Biome;
use margen_generation::core::directions::{
	get_grid_direction_vector, get_surrounding_offsets, DirectionIndex, HORIZONTAL_DIRECTION_COUNT,
};
use margen_generation::core::pathing::CellPaths;
use margen_generation::core::world_cell::{cell_is_path, CellGrid};
use mythic_math::Vector3i;
use std::sync::Arc;

/// Mirrors Unreal DotEnv `LIMIT_CELL_PATHS` (default off).
pub const LIMIT_CELL_PATHS: bool = false;

pub fn get_path_count(paths: CellPaths) -> i32 {
	paths.count_ones() as i32
}

pub fn get_surrounding_path_count(grid: &CellGrid, origin: Vector3i) -> i32 {
	let mut result = 0;
	for offset in get_surrounding_offsets() {
		for h in -1..=1 {
			if is_path_location(grid, origin + offset + Vector3i::new(0, 0, h)) {
				result += 1;
				break;
			}
		}
	}
	result
}

fn cell_is_connected_to_different_biomes(
	grid: &CellGrid,
	origin: Vector3i,
	biome: &Biome,
) -> bool {
	for direction in 0..HORIZONTAL_DIRECTION_COUNT {
		let location = origin + get_grid_direction_vector(direction as DirectionIndex);
		if let Some(cell) = grid.get(location) {
			if cell_is_path(cell) && cell.biome_id() != biome.id {
				return true;
			}
		}
	}
	false
}

/// Returns the count of neighbors that are connected to different biomes.
pub fn get_max_neighbor_biome_connection_count(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	grid: &CellGrid,
	origin: Vector3i,
) -> i32 {
	let origin_biome = get_biome(origin);
	let mut max_count = 0;
	let mut neighbor_count = if cell_is_connected_to_different_biomes(grid, origin, &origin_biome)
	{
		1
	} else {
		0
	};

	for direction in 0..HORIZONTAL_DIRECTION_COUNT {
		let location = origin + get_grid_direction_vector(direction as DirectionIndex);
		if let Some(cell) = grid.get(location) {
			if cell_is_path(cell) {
				if let Some(neighbor_biome) = cell.biome.as_ref() {
					if cell_is_connected_to_different_biomes(grid, location, neighbor_biome) {
						neighbor_count += 1;
						let possible_count = get_path_count(cell.paths);
						if possible_count > max_count {
							max_count = possible_count;
						}
					}
				}
			}
		}
	}

	max_count.max(neighbor_count)
}

/// Biome-junction priority only; RoomDistribution is unused in current Unreal.
pub fn get_cell_priority(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	solving_state: SolvingState<'_>,
	location: Vector3i,
) -> CellPriority {
	let grid = solving_state.grid;

	// Avoid collisions of more than two biomes.
	if get_max_neighbor_biome_connection_count(get_biome, grid, location) > 1 {
		return CellPriority::Fallback;
	}

	if LIMIT_CELL_PATHS && get_surrounding_path_count(grid, location) > 2 {
		return CellPriority::Fallback;
	}

	CellPriority::Primary
}
