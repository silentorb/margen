use crate::core::directions::{
	get_direction_index_from_vector, get_opposite_direction_index, is_downward_direction,
	is_upward_direction, DirectionIndex,
};
use crate::core::world_cell::{cell_add_attributes, cell_add_path, cell_attribute, CellGrid, WorldCell};
use mythic::Vector3i;

pub fn connect_cells_with_path_cells(cell: &mut WorldCell, other: &mut WorldCell, direction: i32) {
	let opposite = get_opposite_direction_index(direction as DirectionIndex);
	cell_add_path(cell, direction as DirectionIndex);
	cell_add_path(other, opposite);

	if is_upward_direction(direction as DirectionIndex) {
		cell_add_attributes(cell, cell_attribute::SOLID);
	} else if is_downward_direction(direction as DirectionIndex) {
		cell_add_attributes(other, cell_attribute::SOLID);
	}
}

pub fn connect_cells_with_path(grid: &mut CellGrid, first: Vector3i, second: Vector3i) {
	let direction = get_direction_index_from_vector(second - first);
	if direction == -1 || !grid.contains(first) || !grid.contains(second) {
		return;
	}
	let mut a = grid.get(first).cloned().expect("first cell");
	let mut b = grid.get(second).cloned().expect("second cell");
	connect_cells_with_path_cells(&mut a, &mut b, direction as i32);
	*grid.entry_or_default(first) = a;
	*grid.entry_or_default(second) = b;
}

pub fn connect_cells_with_paths(grid: &mut CellGrid, locations: &[Vector3i]) {
	if locations.len() < 2 {
		return;
	}
	for window in locations.windows(2) {
		connect_cells_with_path(grid, window[0], window[1]);
	}
}
