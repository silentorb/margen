use margen_generation::{CellGrid, Sector, SectorId};
use mythic_math::Vector3i;
use std::collections::{HashMap, HashSet};

pub fn validate_sector_map(grid: &CellGrid, sectors: &HashMap<SectorId, Sector>) -> HashSet<Vector3i> {
	let mut errors = HashSet::new();

	for (sector_id, sector) in sectors {
		for &location in &sector.cells {
			match grid.get(location) {
				Some(cell) if cell.sector == *sector_id => {}
				_ => {
					errors.insert(location);
				}
			}
		}
	}

	errors
}
