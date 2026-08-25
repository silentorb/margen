use crate::core::directions::{
	get_grid_direction_vector, is_downward_direction, is_upward_direction, DirectionIndex,
	FLAT_DIRECTION_COUNT, HORIZONTAL_DIRECTION_COUNT,
};
use crate::core::pathing::CellPaths;
use crate::core::world_cell::{cell_attribute, cell_is_path, CellAttributes, CellGrid, WorldCell};
use mythic::Vector3i;

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

pub fn new_solving_cell_from_grid(grid: &CellGrid, location: Vector3i) -> SolvingCell {
	if let Some(cell) = grid.get(location) {
		new_solving_cell_from_world(cell)
	} else {
		SolvingCell::default()
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
	use crate::core::directions::directions;
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
	use crate::core::directions::rotate_direction_index;
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
	use crate::core::directions::get_opposite_direction_index;
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
