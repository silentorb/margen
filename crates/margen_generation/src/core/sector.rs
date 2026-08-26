use mythic_math::Vector3i;
use std::collections::HashMap;

pub type SectorId = i16;
pub type SectorAttributes = i32;

pub mod sector_attribute {
	use super::SectorAttributes;

	pub const NONE: SectorAttributes = 0;
	pub const SUPPORTS_UP_OFFSET: SectorAttributes = 1 << 0;
}

#[derive(Clone, Debug, Default)]
pub struct Sector {
	pub id: SectorId,
	pub cells: Vec<Vector3i>,
	pub neighbors: Vec<SectorId>,
	pub attributes: SectorAttributes,
}

pub fn sector_has_attributes(sector: &Sector, attributes: SectorAttributes) -> bool {
	(sector.attributes & attributes) != 0
}

pub fn sector_add_attributes(sector: &mut Sector, attributes: SectorAttributes) {
	sector.attributes |= attributes;
}

pub fn sector_remove_attributes(sector: &mut Sector, attributes: SectorAttributes) {
	sector.attributes &= !attributes;
}

#[derive(Clone, Debug, Default)]
pub struct SectorData {
	pub sectors: HashMap<SectorId, Sector>,
	pub cell_map: HashMap<Vector3i, SectorId>,
	pub origin: SectorId,
}

#[derive(Clone, Debug, Default)]
pub struct LevelSwitchEntry {
	pub source_sector: SectorId,
	pub target_sector: SectorId,
	pub source_cell_location: Vector3i,
}

impl LevelSwitchEntry {
	pub fn new(source_sector: SectorId, target_sector: SectorId) -> Self {
		Self {
			source_sector,
			target_sector,
			source_cell_location: Vector3i::none(),
		}
	}
}
