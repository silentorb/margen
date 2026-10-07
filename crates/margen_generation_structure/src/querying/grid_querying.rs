use margen_generation::core::pathing::CellPaths;
use margen_generation::core::world_cell::{cell_is_path, CellGrid};
use margen_generation::{get_grid_direction_vector, DirectionIndex, HORIZONTAL_DIRECTION_COUNT};
use mythic_math::Vector3i;
use std::collections::HashMap;

pub type NeighborsMap = HashMap<Vector3i, Vec<Vector3i>>;

fn has_path_direction(paths: CellPaths, direction: DirectionIndex) -> bool {
	(paths & (1u32 << direction)) != 0
}

pub fn paths_to_neighbors_array(location: Vector3i, paths: CellPaths) -> Vec<Vector3i> {
	let mut result = Vec::new();
	for direction in 0..HORIZONTAL_DIRECTION_COUNT {
		if has_path_direction(paths, direction) {
			result.push(location + get_grid_direction_vector(direction));
		}
	}
	result
}

pub fn grid_to_neighbors_map(grid: &CellGrid, neighbors_map: &mut NeighborsMap) {
	neighbors_map.clear();
	for (location, cell) in grid.iter() {
		if cell_is_path(cell) {
			neighbors_map.insert(*location, paths_to_neighbors_array(*location, cell.paths));
		}
	}
}

pub fn get_path_locations(grid: &CellGrid) -> Vec<Vector3i> {
	grid
		.iter()
		.filter_map(|(location, cell)| cell_is_path(cell).then_some(*location))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_generation::core::world_cell::{cell_add_attributes, cell_add_path, WorldCell};
	use margen_generation::{cell_attribute, directions::directions};

	fn path_cell_with_links(location: Vector3i, links: &[(DirectionIndex, Vector3i)]) -> (Vector3i, WorldCell) {
		let mut cell = WorldCell::default();
		cell_add_attributes(&mut cell, cell_attribute::PATH);
		for &(direction, _) in links {
			cell_add_path(&mut cell, direction);
		}
		(location, cell)
	}

	#[test]
	fn paths_to_neighbors_resolves_linked_locations() {
		let origin = Vector3i::new(0, 0, 0);
		let east = Vector3i::new(1, 0, 0);
		let south = Vector3i::new(0, -1, 0);

		let mut cell = WorldCell::default();
		cell_add_attributes(&mut cell, cell_attribute::PATH);
		cell_add_path(&mut cell, directions::NORTHEAST);
		cell_add_path(&mut cell, directions::SOUTH);

		let neighbors = paths_to_neighbors_array(origin, cell.paths);
		assert!(neighbors.contains(&east));
		assert!(neighbors.contains(&south));
		assert_eq!(neighbors.len(), 2);
	}

	#[test]
	fn grid_to_neighbors_map_builds_path_graph() {
		let mut grid = CellGrid::new();
		let a = Vector3i::new(0, 0, 0);
		let b = Vector3i::new(1, 0, 0);

		let (loc_a, cell_a) = path_cell_with_links(a, &[(directions::NORTHEAST, b)]);
		let (loc_b, cell_b) = path_cell_with_links(b, &[(directions::SOUTHWEST, a)]);
		grid.insert(loc_a, cell_a);
		grid.insert(loc_b, cell_b);

		let mut map = NeighborsMap::new();
		grid_to_neighbors_map(&grid, &mut map);

		assert_eq!(map.len(), 2);
		assert_eq!(map[&a], vec![b]);
		assert_eq!(map[&b], vec![a]);
	}

	#[test]
	fn get_path_locations_skips_solid_cells() {
		let mut grid = CellGrid::new();
		let path_loc = Vector3i::new(2, 2, 0);
		let solid_loc = Vector3i::new(3, 3, 0);

		let mut path_cell = WorldCell::default();
		cell_add_attributes(&mut path_cell, cell_attribute::PATH);
		let mut solid_cell = WorldCell::default();
		cell_add_attributes(&mut solid_cell, cell_attribute::SOLID);

		grid.insert(path_loc, path_cell);
		grid.insert(solid_loc, solid_cell);

		let paths = get_path_locations(&grid);
		assert_eq!(paths, vec![path_loc]);
	}
}
