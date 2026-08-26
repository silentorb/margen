use margen_generation::core::world_cell::{cell_is_solid, CellGrid};
use margen_generation::{sector_add_attributes, sector_attribute, sector_remove_attributes, Sector, SectorId};
use mythic_math::Vector3i;
use std::collections::HashMap;

fn cell_has_headroom_clearance(
	grid: &CellGrid,
	required_headroom: i32,
	sector: SectorId,
	origin: Vector3i,
) -> bool {
	let mut location = origin;
	let mut space_cell_series_accumulator = 0;

	for _ in 0..1000 {
		location.z += 1;
		match grid.get(location) {
			Some(cell) if cell_is_solid(cell) => {
				if cell.sector != sector {
					return false;
				}
				space_cell_series_accumulator = 0;
			}
			Some(_) | None => {
				space_cell_series_accumulator += 1;
				if space_cell_series_accumulator >= required_headroom {
					return true;
				}
			}
		}
	}

	false
}

pub fn calculate_headroom_clearance_for_sectors(
	grid: &CellGrid,
	sectors: &mut HashMap<SectorId, Sector>,
	required_headroom: i32,
) {
	for sector in sectors.values_mut() {
		sector_add_attributes(sector, sector_attribute::SUPPORTS_UP_OFFSET);
	}

	for (location, cell) in grid.iter() {
		if let Some(sector) = sectors.get_mut(&cell.sector) {
			if (sector.attributes & sector_attribute::SUPPORTS_UP_OFFSET) != 0
				&& !cell_has_headroom_clearance(grid, required_headroom, cell.sector, *location)
			{
				sector_remove_attributes(sector, sector_attribute::SUPPORTS_UP_OFFSET);
			}
		}
	}
}
