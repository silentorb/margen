use crate::analysis::types::{PathDepths, Strips, StripMap};
use crate::core::{Sector, SectorData, SectorId};
use crate::structure::querying::NeighborsMap;
use mythic::Vector3i;
use std::collections::{HashSet, VecDeque};

struct NeighborEntry {
	location: Vector3i,
	neighbors: Vec<Vector3i>,
}

fn get_neighbor_count(neighbors: &[NeighborEntry], origin: &NeighborEntry) -> i32 {
	let mut result = 0;
	for neighbor_location in &origin.neighbors {
		if neighbors.iter().any(|n| n.location == *neighbor_location) {
			result += 1;
		}
	}
	result
}

fn are_neighbors_contiguous(neighbors: &[NeighborEntry]) -> bool {
	if neighbors.len() < 2 {
		return true;
	}

	let mut end_count = 0;
	for neighbor in neighbors {
		let neighbor_count = get_neighbor_count(neighbors, neighbor);
		if neighbor_count == 0 {
			return false;
		}
		if neighbor_count == 1 {
			end_count += 1;
		}
	}

	end_count == 2
}

fn gather_neighbor_strips(
	neighbors_map: &NeighborsMap,
	depths: &PathDepths,
	strips: &mut Strips,
	strip_map: &mut StripMap,
) {
	let mut queue = VecDeque::new();

	for (origin, neighbors) in neighbors_map {
		if strip_map.contains_key(origin) {
			continue;
		}

		let strip_index = strips.len();
		strips.push(Vec::new());
		queue.push_back(*origin);

		while let Some(location) = queue.pop_front() {
			strip_map.insert(location, strip_index);
			strips[strip_index].push(location);

			if let Some(depth) = depths.get(&location) {
				for neighbor in neighbors {
					if depths.get(neighbor) == Some(depth) && !strip_map.contains_key(neighbor) {
						queue.push_back(*neighbor);
					}
				}
			}
		}
	}
}

fn gather_fork_strips(
	neighbors_map: &NeighborsMap,
	depths: &PathDepths,
	strips: &Strips,
	fork_strips: &mut Vec<usize>,
) {
	for (strip_index, strip) in strips.iter().enumerate() {
		let mut further_neighbors: Vec<NeighborEntry> = Vec::new();

		for location in strip {
			if let Some(depth) = depths.get(location) {
				if let Some(neighbors) = neighbors_map.get(location) {
					for neighbor in neighbors {
						if depths.get(neighbor).copied().unwrap_or(-1) > *depth
							&& !further_neighbors.iter().any(|n| n.location == *neighbor)
						{
							if let Some(neighbor_path) = neighbors_map.get(neighbor) {
								further_neighbors.push(NeighborEntry {
									location: *neighbor,
									neighbors: neighbor_path.clone(),
								});
							}
						}
					}
				}
			}
		}

		if !are_neighbors_contiguous(&further_neighbors) {
			fork_strips.push(strip_index);
		}
	}
}

fn next_sector_id(sector_data: &SectorData) -> SectorId {
	(sector_data.sectors.len() + 1) as SectorId
}

fn propagate_sectors_first_pass(
	neighbors_map: &NeighborsMap,
	depths: &PathDepths,
	fork_strips: &[usize],
	strips: &Strips,
	fork_locations: &HashSet<Vector3i>,
	sector_data: &mut SectorData,
) {
	let mut queue = VecDeque::new();

	for &strip_index in fork_strips {
		let strip = &strips[strip_index];
		if strip.is_empty() || sector_data.cell_map.contains_key(&strip[0]) {
			continue;
		}

		let sector_id = next_sector_id(sector_data);
		let sector = sector_data
			.sectors
			.entry(sector_id)
			.or_insert_with(|| Sector {
				id: sector_id,
				..Default::default()
			});

		for &fork_location in strip {
			sector.cells.push(fork_location);
			sector_data.cell_map.insert(fork_location, sector_id);
			queue.push_back(fork_location);
		}

		while let Some(location) = queue.pop_front() {
			let Some(path_directions) = neighbors_map.get(&location) else {
				continue;
			};
			let depth = depths[&location];

			for &neighbor in path_directions {
				if sector_data.cell_map.contains_key(&neighbor)
					|| fork_locations.contains(&neighbor)
				{
					continue;
				}

				let neighbor_depth = depths[&neighbor];
				if neighbor_depth < depth {
					sector_data
						.sectors
						.get_mut(&sector_id)
						.expect("sector present")
						.cells
						.push(neighbor);
					sector_data.cell_map.insert(neighbor, sector_id);
					queue.push_back(neighbor);
				}
			}
		}
	}
}

fn propagate_sectors_second_pass(
	neighbors_map: &NeighborsMap,
	fork_locations: &HashSet<Vector3i>,
	sector_data: &mut SectorData,
) {
	let mut queue = VecDeque::new();

	for &fork_location in fork_locations {
		let Some(&sector_id) = sector_data.cell_map.get(&fork_location) else {
			continue;
		};

		queue.push_back(fork_location);
		while let Some(location) = queue.pop_front() {
			if let Some(path_directions) = neighbors_map.get(&location) {
				for &neighbor in path_directions {
					if sector_data.cell_map.contains_key(&neighbor)
						|| fork_locations.contains(&neighbor)
					{
						continue;
					}

					sector_data
						.sectors
						.get_mut(&sector_id)
						.expect("sector present")
						.cells
						.push(neighbor);
					sector_data.cell_map.insert(neighbor, sector_id);
					queue.push_back(neighbor);
				}
			}
		}
	}
}

fn propagate_sectors_third_pass(neighbors_map: &NeighborsMap, sector_data: &mut SectorData) {
	let mut queue: Vec<Vector3i> = neighbors_map
		.keys()
		.filter(|path_location| !sector_data.cell_map.contains_key(path_location))
		.copied()
		.collect();

	while !queue.is_empty() {
		let mut changed_count = 0;

		for i in (0..queue.len()).rev() {
			let location = queue[i];
			let Some(path_directions) = neighbors_map.get(&location) else {
				continue;
			};

			for &neighbor_location in path_directions {
				if neighbors_map.contains_key(&neighbor_location) {
					if let Some(&sector_id) = sector_data.cell_map.get(&neighbor_location) {
						sector_data.cell_map.insert(location, sector_id);
						sector_data
							.sectors
							.get_mut(&sector_id)
							.expect("sector present")
							.cells
							.push(location);
						queue.remove(i);
						changed_count += 1;
						break;
					}
				}
			}
		}

		if changed_count == 0 {
			break;
		}
	}
}

pub fn connect_neighbor_sectors(neighbors_map: &NeighborsMap, sector_data: &mut SectorData) {
	let sector_ids: Vec<SectorId> = sector_data.sectors.keys().copied().collect();
	for sector_id in sector_ids {
		let cells = sector_data.sectors[&sector_id].cells.clone();
		for origin in cells {
			let Some(neighbors) = neighbors_map.get(&origin) else {
				continue;
			};
			for &neighbor in neighbors {
				if let Some(&neighbor_sector_id) = sector_data.cell_map.get(&neighbor) {
					if neighbor_sector_id != sector_id {
						let sector = sector_data.sectors.get_mut(&sector_id).expect("sector");
						if !sector.neighbors.contains(&neighbor_sector_id) {
							sector.neighbors.push(neighbor_sector_id);
							sector_data
								.sectors
								.get_mut(&neighbor_sector_id)
								.expect("neighbor sector")
								.neighbors
								.push(sector_id);
						}
					}
				}
			}
		}
	}
}

pub fn check_create_fallback_sector(grid: &crate::core::CellGrid, sector_data: &mut SectorData) {
	if !sector_data.sectors.is_empty() {
		return;
	}

	let sector_id = next_sector_id(sector_data);
	let sector = sector_data
		.sectors
		.entry(sector_id)
		.or_insert_with(|| Sector {
			id: sector_id,
			..Default::default()
		});

	for (location, _) in grid.iter() {
		sector_data.cell_map.insert(*location, sector_id);
		sector.cells.push(*location);
	}
}

pub fn sync_sector_cells(sector_data: &SectorData, grid: &mut crate::core::CellGrid) {
	for (&location, &sector) in &sector_data.cell_map {
		if let Some(cell) = grid.get_mut(location) {
			cell.sector = sector;
		}
	}
}

pub fn group_cells(
	neighbors_map: &NeighborsMap,
	depths: &PathDepths,
	strip_map: &mut StripMap,
	fork_locations: &mut Vec<Vector3i>,
	sector_data: &mut SectorData,
	second_pass: bool,
	third_pass: bool,
) {
	sector_data.origin = 1;

	let mut strips = Strips::new();
	let mut fork_strips = Vec::new();
	let mut fork_location_set = HashSet::new();

	gather_neighbor_strips(neighbors_map, depths, &mut strips, strip_map);
	gather_fork_strips(neighbors_map, depths, &strips, &mut fork_strips);

	for &strip_index in &fork_strips {
		for &location in &strips[strip_index] {
			fork_locations.push(location);
			fork_location_set.insert(location);
		}
	}

	propagate_sectors_first_pass(
		neighbors_map,
		depths,
		&fork_strips,
		&strips,
		&fork_location_set,
		sector_data,
	);

	if second_pass {
		propagate_sectors_second_pass(neighbors_map, &fork_location_set, sector_data);
	}

	if third_pass {
		propagate_sectors_third_pass(neighbors_map, sector_data);
	}

	connect_neighbor_sectors(neighbors_map, sector_data);
}
