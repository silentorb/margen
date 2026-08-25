use crate::core::world_cell::{cell_is_solid, CellGrid};
use crate::core::{get_surrounding_offsets, SectorId};
use mythic::Vector3i;

fn try_add_sector_assigned_neighbor(
	grid: &CellGrid,
	location: Vector3i,
	neighbor_cells: &mut Vec<(Vector3i, SectorId, Option<u32>)>,
) {
	if let Some(neighbor_cell) = grid.get(location) {
		if neighbor_cell.sector != 0 {
			neighbor_cells.push((
				location,
				neighbor_cell.sector,
				neighbor_cell.biome.as_ref().map(|b| b.id),
			));
		}
	}
}

fn get_sector_assigned_neighbor_cells(
	grid: &CellGrid,
	location: Vector3i,
) -> Vec<(Vector3i, SectorId, Option<u32>)> {
	let mut neighbor_cells = Vec::new();
	for offset in get_surrounding_offsets() {
		try_add_sector_assigned_neighbor(grid, location + offset, &mut neighbor_cells);
	}
	try_add_sector_assigned_neighbor(grid, location + Vector3i::new(0, 0, 1), &mut neighbor_cells);
	try_add_sector_assigned_neighbor(grid, location + Vector3i::new(0, 0, -1), &mut neighbor_cells);
	neighbor_cells
}

fn get_strongest_neighbor_sector(sector_weights: &[(SectorId, i32)]) -> SectorId {
	let mut strongest_sector_id = 0;
	let mut strongest_sector_weight = 0;

	for &(sector_id, weight) in sector_weights {
		if weight > strongest_sector_weight {
			strongest_sector_id = sector_id;
			strongest_sector_weight = weight;
		}
	}

	strongest_sector_id
}

fn resolve_cell_sector(
	neighbors: &[(Vector3i, SectorId, Option<u32>)],
	preferred_biome_id: Option<u32>,
) -> SectorId {
	if neighbors.is_empty() {
		return 0;
	}
	if neighbors.len() == 1 {
		return neighbors[0].1;
	}

	let mut sector_weights: Vec<(SectorId, i32)> = Vec::new();
	for &(_, sector, biome_id) in neighbors {
		let biome_weight = if biome_id == preferred_biome_id { 1 } else { 0 };
		if let Some((_, weight)) = sector_weights.iter_mut().find(|(id, _)| *id == sector) {
			*weight += 1 + biome_weight;
		} else {
			sector_weights.push((sector, 1 + biome_weight));
		}
	}

	get_strongest_neighbor_sector(&sector_weights)
}

fn propagate_sector_cells_along_vector(
	offset: Vector3i,
	origin: Vector3i,
	grid: &mut CellGrid,
) {
	let sector = grid.get(origin).expect("origin cell").sector;
	let mut location = origin;

	for _ in 0..1000 {
		location = location + offset;
		match grid.get_mut(location) {
			Some(cell) if cell.sector == 0 => {
				cell.sector = sector;
			}
			Some(_) | None => break,
		}
	}
}

fn propagate_sector_cells_along_column(origin: Vector3i, grid: &mut CellGrid) {
	propagate_sector_cells_along_vector(Vector3i::new(0, 0, 1), origin, grid);
	propagate_sector_cells_along_vector(Vector3i::new(0, 0, -1), origin, grid);
}

fn propagate_sector_cells_along_columns(grid: &mut CellGrid, path_locations: &[Vector3i]) {
	for &path_location in path_locations {
		propagate_sector_cells_along_column(path_location, grid);
	}
}

fn gather_solid_unassigned_sector_cells(grid: &CellGrid) -> Vec<Vector3i> {
	grid
		.iter()
		.filter_map(|(location, cell)| {
			(cell.sector == 0 && cell_is_solid(cell)).then_some(*location)
		})
		.collect()
}

fn gather_unassigned_sector_cells(grid: &CellGrid) -> Vec<Vector3i> {
	grid
		.iter()
		.filter_map(|(location, cell)| (cell.sector == 0).then_some(*location))
		.collect()
}

fn propagate_sector_cells_anywhere(unassigned: &mut Vec<Vector3i>, grid: &mut CellGrid) {
	while !unassigned.is_empty() {
		let mut changed_count = 0;

		for i in (0..unassigned.len()).rev() {
			let location = unassigned[i];
			let cell = grid.get(location).expect("unassigned cell present");
			if cell.sector > 0 {
				changed_count += 1;
				unassigned.remove(i);
				continue;
			}

			let preferred_biome_id = cell.biome.as_ref().map(|b| b.id);
			let neighbors = get_sector_assigned_neighbor_cells(grid, location);
			let resolved_sector = resolve_cell_sector(&neighbors, preferred_biome_id);
			if resolved_sector > 0 {
				grid.get_mut(location).expect("cell present").sector = resolved_sector;
				changed_count += 1;
				unassigned.remove(i);
				propagate_sector_cells_along_column(location, grid);
			}
		}

		if changed_count == 0 {
			break;
		}
	}
}

pub fn propagate_sector_cells(grid: &mut CellGrid, path_locations: &[Vector3i]) {
	propagate_sector_cells_along_columns(grid, path_locations);

	let mut unassigned = gather_solid_unassigned_sector_cells(grid);
	propagate_sector_cells_anywhere(&mut unassigned, grid);

	unassigned = gather_unassigned_sector_cells(grid);
	propagate_sector_cells_anywhere(&mut unassigned, grid);
}
