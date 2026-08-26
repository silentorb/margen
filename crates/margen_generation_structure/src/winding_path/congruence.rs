use margen_generation::biomes::Biome;
use margen_generation::core::directions::{
	get_grid_direction_vector, get_surrounding_offsets, is_downward_direction, is_upward_direction,
	DirectionIndex, FLAT_DIRECTION_COUNT, HORIZONTAL_DIRECTION_COUNT,
};
use margen_generation::core::pathing::CellPaths;
use margen_generation::core::world_cell::{
	cell_attribute, cell_is_path, cell_is_solid, CellAttributes, CellGrid, WorldCell,
};
use mythic_math::Vector3i;

#[derive(Clone, Copy, Debug, Default)]
pub struct SolvingCell {
	pub attributes: CellAttributes,
	pub paths: CellPaths,
}

pub fn new_solving_cell_from_world(cell: &WorldCell) -> SolvingCell {
	SolvingCell {
		attributes: cell.attributes,
		paths: cell.paths,
	}
}

pub fn new_solving_cell_from_biome(biome: &Biome) -> SolvingCell {
	let flat_attributes = if biome.is_flat() {
		cell_attribute::FLAT
	} else {
		cell_attribute::NONE
	};
	SolvingCell {
		attributes: cell_attribute::PATH | flat_attributes,
		paths: 0,
	}
}

pub fn new_solving_cell_from_grid(grid: &CellGrid, location: Vector3i) -> SolvingCell {
	if let Some(cell) = grid.get(location) {
		new_solving_cell_from_world(cell)
	} else {
		SolvingCell::default()
	}
}

pub fn new_solving_cell_from_biome_or_grid(
	biome: &Biome,
	grid: &CellGrid,
	location: Vector3i,
) -> SolvingCell {
	if let Some(cell) = grid.get(location) {
		new_solving_cell_from_world(cell)
	} else {
		new_solving_cell_from_biome(biome)
	}
}

pub fn has_direction(paths: CellPaths, direction: DirectionIndex) -> bool {
	(paths & (1u32 << direction)) != 0
}

pub fn is_path_location(grid: &CellGrid, location: Vector3i) -> bool {
	grid.get(location).is_some_and(cell_is_path)
}

pub fn get_cell_max_path_directions(attributes: CellAttributes) -> DirectionIndex {
	if (attributes & cell_attribute::FLAT) != 0 {
		FLAT_DIRECTION_COUNT
	} else {
		HORIZONTAL_DIRECTION_COUNT
	}
}

pub fn get_neighbor_cell<'a>(
	grid: &'a CellGrid,
	location: Vector3i,
	direction: DirectionIndex,
) -> Option<&'a WorldCell> {
	grid.get(location + get_grid_direction_vector(direction))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CongruenceError {
	None = 0,
	UpwardSlope,
	NeighboringUpward,
	NoPath,
	Flat,
}

fn supports_new_upward_slope(paths: CellPaths, direction: DirectionIndex) -> CongruenceError {
	use margen_generation::core::directions::directions;
	let relative_base = direction - directions::UP_NORTH;
	let neighbor_mask = (1 << relative_base)
		| (1 << ((relative_base + 5) % 6))
		| (1 << ((relative_base + 1) % 6));
	let non_neighbor_upward_mask = (0x3Fu32 ^ neighbor_mask as u32) << directions::UP_NORTH;
	let downward_mask = (neighbor_mask as u32) << directions::DOWN_NORTH;
	let unsupported_paths_mask = non_neighbor_upward_mask | downward_mask;
	let conflicts = paths & unsupported_paths_mask;
	if conflicts != 0 {
		CongruenceError::UpwardSlope
	} else {
		CongruenceError::None
	}
}

fn are_cell_paths_compatible(paths: CellPaths, direction: DirectionIndex) -> CongruenceError {
	use margen_generation::core::directions::rotate_direction_index;
	if is_upward_direction(direction) {
		supports_new_upward_slope(paths, direction)
	} else if is_downward_direction(direction) {
		let upward = direction - 6;
		if has_direction(paths, rotate_direction_index(upward, 1))
			|| has_direction(paths, rotate_direction_index(upward, -1))
		{
			CongruenceError::NeighboringUpward
		} else {
			CongruenceError::None
		}
	} else {
		CongruenceError::None
	}
}

pub fn can_cell_connect_in_direction(
	cell: &SolvingCell,
	direction: DirectionIndex,
) -> CongruenceError {
	if (cell.attributes & cell_attribute::PATH) == 0 {
		return CongruenceError::NoPath;
	}
	if direction > 5 && (cell.attributes & cell_attribute::FLAT) != 0 {
		return CongruenceError::Flat;
	}
	are_cell_paths_compatible(cell.paths, direction)
}

pub fn can_cells_be_connected(
	first: &SolvingCell,
	second: &SolvingCell,
	direction: DirectionIndex,
) -> CongruenceError {
	use margen_generation::core::directions::get_opposite_direction_index;
	let error = can_cell_connect_in_direction(first, direction);
	if error != CongruenceError::None {
		error
	} else {
		can_cell_connect_in_direction(second, get_opposite_direction_index(direction))
	}
}

pub fn gather_neighbor_cell_paths_array(
	cell: &SolvingCell,
	grid: &CellGrid,
	origin: Vector3i,
) -> Vec<DirectionIndex> {
	let mut paths = Vec::new();
	let direction_count = get_cell_max_path_directions(cell.attributes);
	for direction in 0..direction_count {
		if let Some(other) = get_neighbor_cell(grid, origin, direction) {
			let error =
				can_cells_be_connected(cell, &new_solving_cell_from_world(other), direction);
			if error == CongruenceError::None {
				paths.push(direction);
			}
		}
	}
	paths
}

pub fn get_cell_empty_headroom(grid: &CellGrid, origin: Vector3i, max_steps: i32) -> i32 {
	for h in 0..max_steps {
		if grid.contains(origin + Vector3i::new(0, 0, h)) {
			return h;
		}
	}
	max_steps
}

pub fn block_space_is_available(grid: &CellGrid, origin: Vector3i, headroom: i32) -> bool {
	let below = origin + Vector3i::new(0, 0, -1);
	if let Some(existing) = grid.get(below) {
		if !cell_is_solid(existing) {
			return false;
		}
	}
	get_cell_empty_headroom(grid, origin, headroom) >= headroom
}

pub fn cell_has_viable_paths(cell: &SolvingCell, grid: &CellGrid, origin: Vector3i) -> bool {
	let direction_count = get_cell_max_path_directions(cell.attributes);
	for direction in 0..direction_count {
		if let Some(other) = get_neighbor_cell(grid, origin, direction) {
			let error =
				can_cells_be_connected(cell, &new_solving_cell_from_world(other), direction);
			if error == CongruenceError::None {
				return true;
			}
		}
	}
	false
}

pub fn cell_has_no_flat_slope_conflicts(
	grid: &CellGrid,
	cell: &SolvingCell,
	origin: Vector3i,
) -> bool {
	if (cell.attributes & cell_attribute::FLAT) != 0 {
		for offset in get_surrounding_offsets() {
			let column = origin + offset;
			if is_path_location(grid, column + Vector3i::new(0, 0, -1)) {
				return false;
			}
		}
	}

	for offset in get_surrounding_offsets() {
		let column = origin + offset;
		if let Some(neighbor) = grid.get(column + Vector3i::new(0, 0, -1)) {
			if cell_is_path(neighbor) && (neighbor.attributes & cell_attribute::FLAT) != 0 {
				return false;
			}
		}
	}

	true
}

pub fn can_place_block(
	grid: &CellGrid,
	headroom: i32,
	cell: &SolvingCell,
	origin: Vector3i,
) -> bool {
	if !block_space_is_available(grid, origin, headroom) {
		return false;
	}
	if !cell_has_viable_paths(cell, grid, origin) {
		return false;
	}
	if !cell_has_no_flat_slope_conflicts(grid, cell, origin) {
		return false;
	}
	true
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_generation::biomes::{default_biome, Biome, BIOME_TAG_FLAT};
	use margen_generation::core::directions::directions;
	use margen_generation::core::world_cell::cell_attribute;

	fn path_cell() -> WorldCell {
		WorldCell {
			attributes: cell_attribute::PATH,
			..Default::default()
		}
	}

	fn solid_cell() -> WorldCell {
		WorldCell {
			attributes: cell_attribute::SOLID,
			..Default::default()
		}
	}

	#[test]
	fn biome_solving_cell_marks_flat_from_tag() {
		let mut biome = Biome::new(1);
		biome.tags.insert(BIOME_TAG_FLAT.to_string());
		let cell = new_solving_cell_from_biome(&biome);
		assert_eq!(
			cell.attributes,
			cell_attribute::PATH | cell_attribute::FLAT
		);
		assert_eq!(cell.paths, 0);
	}

	#[test]
	fn biome_solving_cell_default_is_path_only() {
		let biome = default_biome();
		let cell = new_solving_cell_from_biome(&biome);
		assert_eq!(cell.attributes, cell_attribute::PATH);
	}

	#[test]
	fn block_space_rejects_non_solid_below() {
		let mut grid = CellGrid::new();
		let origin = Vector3i::new(0, 0, 0);
		grid.insert(origin + Vector3i::new(0, 0, -1), path_cell());
		assert!(!block_space_is_available(&grid, origin, 3));
	}

	#[test]
	fn block_space_requires_empty_headroom_column() {
		let mut grid = CellGrid::new();
		let origin = Vector3i::new(0, 0, 0);
		grid.insert(origin + Vector3i::new(0, 0, -1), solid_cell());
		assert!(block_space_is_available(&grid, origin, 3));
		grid.insert(origin + Vector3i::new(0, 0, 1), solid_cell());
		assert!(!block_space_is_available(&grid, origin, 3));
	}

	#[test]
	fn can_place_needs_viable_neighbor_path() {
		let mut grid = CellGrid::new();
		let origin = Vector3i::new(0, 0, 0);
		let neighbor = origin + get_grid_direction_vector(directions::NORTH);
		grid.insert(origin + Vector3i::new(0, 0, -1), solid_cell());
		let candidate = new_solving_cell_from_biome(&default_biome());
		assert!(!can_place_block(&grid, 3, &candidate, origin));

		grid.insert(neighbor, path_cell());
		assert!(can_place_block(&grid, 3, &candidate, origin));
	}

	#[test]
	fn flat_candidate_rejects_lower_path_neighbor() {
		let mut grid = CellGrid::new();
		let origin = Vector3i::new(0, 0, 0);
		let neighbor_col = origin + get_grid_direction_vector(directions::NORTH);
		grid.insert(origin + Vector3i::new(0, 0, -1), solid_cell());
		grid.insert(neighbor_col, path_cell());
		grid.insert(
			neighbor_col + Vector3i::new(0, 0, -1),
			path_cell(),
		);

		let mut flat_biome = Biome::new(2);
		flat_biome.tags.insert(BIOME_TAG_FLAT.to_string());
		let candidate = new_solving_cell_from_biome(&flat_biome);
		assert!(!can_place_block(&grid, 3, &candidate, origin));
	}

	#[test]
	fn flat_neighbor_below_blocks_non_flat_candidate() {
		let mut grid = CellGrid::new();
		let origin = Vector3i::new(0, 0, 0);
		let neighbor_col = origin + get_grid_direction_vector(directions::NORTH);
		grid.insert(origin + Vector3i::new(0, 0, -1), solid_cell());
		grid.insert(neighbor_col, path_cell());
		let mut flat_below = path_cell();
		flat_below.attributes |= cell_attribute::FLAT;
		grid.insert(neighbor_col + Vector3i::new(0, 0, -1), flat_below);

		let candidate = new_solving_cell_from_biome(&default_biome());
		assert!(!can_place_block(&grid, 3, &candidate, origin));
	}
}
