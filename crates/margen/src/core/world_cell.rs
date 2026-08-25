use crate::core::directions::DirectionIndex;
use crate::core::pathing::CellPaths;
use crate::core::sector::SectorId;
use mythic::Vector3i;
use std::collections::HashMap;

pub type CellAttributes = i64;

pub mod cell_attribute {
	use super::CellAttributes;

	pub const NONE: CellAttributes = 0;
	pub const SOLID: CellAttributes = 1 << 0;
	pub const PATH: CellAttributes = 1 << 1;
	pub const SLOTS: CellAttributes = 1 << 2;
	pub const HEADROOM: CellAttributes = 1 << 3;
	pub const FLAT: CellAttributes = 1 << 4;
	pub const NARROW_WALLS: CellAttributes = 1 << 5;
	pub const CUSTOM_MESH: CellAttributes = 1 << 6;
	pub const NON_ESSENTIAL_PATH: CellAttributes = 1 << 7;
	pub const CUSTOM_WALL_SHIFTING: CellAttributes = 1 << 8;
	pub const INDEPENDENT_CELL_MESH: CellAttributes = 1 << 9;
}

pub const CELL_MATERIAL_COUNT: i32 = 8;

#[derive(Clone, Debug, Default)]
pub struct WorldCell {
	pub step: u32,
	pub attributes: CellAttributes,
	pub paths: CellPaths,
	pub biome_id: u32,
	pub sector: SectorId,
}

pub fn cell_has_attribute(cell: &WorldCell, attribute: CellAttributes) -> bool {
	(cell.attributes & attribute) != 0
}

pub fn cell_add_attributes(cell: &mut WorldCell, new_attributes: CellAttributes) {
	cell.attributes |= new_attributes;
}

pub fn cell_remove_attributes(cell: &mut WorldCell, new_attributes: CellAttributes) {
	cell.attributes &= !new_attributes;
}

pub fn cell_is_path(cell: &WorldCell) -> bool {
	cell_has_attribute(cell, cell_attribute::PATH)
}

pub fn cell_is_solid(cell: &WorldCell) -> bool {
	cell_has_attribute(cell, cell_attribute::SOLID)
}

pub fn cell_has_path(cell: &WorldCell, direction: i32) -> bool {
	(cell.paths & (1u32 << direction)) != 0
}

pub fn cell_add_paths(cell: &mut WorldCell, new_paths: CellPaths) {
	cell.paths |= new_paths;
}

pub fn cell_add_path(cell: &mut WorldCell, direction: DirectionIndex) {
	cell_add_paths(cell, 1u32 << direction);
}

pub fn cell_remove_path(cell: &mut WorldCell, direction: DirectionIndex) {
	cell.paths &= !(1u32 << direction);
}

/// Spatial cell map keyed by integer grid coordinates.
#[derive(Clone, Debug, Default)]
pub struct CellGrid {
	cells: HashMap<Vector3i, WorldCell>,
}

impl CellGrid {
	pub fn new() -> Self {
		Self {
			cells: HashMap::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.cells.len()
	}

	pub fn is_empty(&self) -> bool {
		self.cells.is_empty()
	}

	pub fn contains(&self, location: Vector3i) -> bool {
		self.cells.contains_key(&location)
	}

	pub fn get(&self, location: Vector3i) -> Option<&WorldCell> {
		self.cells.get(&location)
	}

	pub fn get_mut(&mut self, location: Vector3i) -> Option<&mut WorldCell> {
		self.cells.get_mut(&location)
	}

	pub fn entry_or_default(&mut self, location: Vector3i) -> &mut WorldCell {
		self.cells.entry(location).or_default()
	}

	pub fn iter(&self) -> impl Iterator<Item = (&Vector3i, &WorldCell)> {
		self.cells.iter()
	}
}
