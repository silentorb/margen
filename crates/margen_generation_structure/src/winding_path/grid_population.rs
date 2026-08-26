use margen_generation::core::world_cell::{
	cell_add_attributes, cell_add_path, cell_attribute, CellAttributes, CellGrid, WorldCell,
};
use crate::types::NewCellContext;
use crate::winding_path::congruence::{
	can_cells_be_connected, gather_neighbor_cell_paths_array, new_solving_cell_from_world,
	CongruenceError,
};
use margen_generation::core::directions::{
	get_direction_index_from_vector, get_grid_direction_vector, get_opposite_direction_index,
	is_downward_direction, is_upward_direction, DirectionIndex,
};
use mythic_dice::psuedorandom;
use mythic_math::Vector3i;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct BlockLayer {
	pub attributes: CellAttributes,
	pub count: i32,
}

impl BlockLayer {
	pub fn new(attributes: CellAttributes) -> Self {
		Self {
			attributes,
			count: 1,
		}
	}

	pub fn with_count(attributes: CellAttributes, count: i32) -> Self {
		Self { attributes, count }
	}
}

#[derive(Clone, Debug, Default)]
pub struct Block {
	pub layers: Vec<BlockLayer>,
	pub z_offset: i16,
}

impl Block {
	pub fn new(layers: impl IntoIterator<Item = BlockLayer>, z_offset: i16) -> Self {
		Self {
			layers: layers.into_iter().collect(),
			z_offset,
		}
	}
}

pub fn create_new_cell(context: &NewCellContext, attributes: CellAttributes) -> WorldCell {
	let mut cell = WorldCell {
		attributes,
		biome: Some(Arc::clone(&context.biome)),
		step: context.step,
		sector: context.sector,
		..Default::default()
	};
	if (attributes & cell_attribute::PATH) != 0 && context.biome.is_flat() {
		cell.attributes |= cell_attribute::FLAT;
	}
	cell
}

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

pub fn add_block(grid: &mut CellGrid, origin: Vector3i, context: &NewCellContext, block: &Block) {
	let mut step = i32::from(block.z_offset);
	for layer in &block.layers {
		for _ in 0..layer.count {
			let location = origin + Vector3i::new(0, 0, step);
			step += 1;
			grid.insert(location, create_new_cell(context, layer.attributes));
		}
	}
}

pub fn block_has_available_space(grid: &CellGrid, origin: Vector3i, block: &Block) -> bool {
	let mut step = i32::from(block.z_offset);
	for layer in &block.layers {
		for _ in 0..layer.count {
			let location = origin + Vector3i::new(0, 0, step);
			step += 1;
			if grid.contains(location) {
				return false;
			}
		}
	}
	true
}

pub fn try_add_block(grid: &mut CellGrid, origin: Vector3i, context: &NewCellContext, block: &Block) {
	if block_has_available_space(grid, origin, block) {
		add_block(grid, origin, context, block);
	}
}

fn connect_ready_cell_paths(grid: &mut CellGrid, paths: &[DirectionIndex], origin: Vector3i) {
	if paths.is_empty() || !grid.contains(origin) {
		return;
	}
	let start = psuedorandom(
		origin
			.x
			.wrapping_add(origin.y)
			.wrapping_add(origin.z)
			.wrapping_add(paths.len() as i32),
		paths.len() as i32,
	) as usize;
	for i in 0..paths.len() {
		let direction = paths[(start + i) % paths.len()];
		let neighbor_loc = origin + get_grid_direction_vector(direction);
		let Some(neighbor) = grid.get(neighbor_loc).cloned() else {
			continue;
		};
		let Some(cell) = grid.get(origin).cloned() else {
			continue;
		};
		let solving_cell = new_solving_cell_from_world(&cell);
		let solving_neighbor = new_solving_cell_from_world(&neighbor);
		if can_cells_be_connected(&solving_cell, &solving_neighbor, direction) == CongruenceError::None
		{
			connect_cells_with_path(grid, origin, neighbor_loc);
		}
	}
}

pub fn connect_cell_neighbors(grid: &mut CellGrid, location: Vector3i, cell: &WorldCell) {
	let paths = gather_neighbor_cell_paths_array(&new_solving_cell_from_world(cell), grid, location);
	connect_ready_cell_paths(grid, &paths, location);
}

pub fn add_connected_block(
	grid: &mut CellGrid,
	location: Vector3i,
	context: &NewCellContext,
	block: &Block,
) {
	add_block(grid, location, context, block);
	if let Some(cell) = grid.get(location).cloned() {
		connect_cell_neighbors(grid, location, &cell);
	}
}
