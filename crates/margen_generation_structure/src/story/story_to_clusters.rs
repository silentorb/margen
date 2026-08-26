use margen_generation::core::directions::{get_grid_direction_vector, get_surrounding_offsets};
use margen_generation::core::world_cell::{cell_add_attributes, cell_attribute, CellGrid};
use margen_generation::SectorId;
use margen_generation_graphing::{
	get_immediate_node_descendants, DagConnectionId, DagGraph, DagNodeId,
};
use crate::winding_path::connect_cells_with_path;
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::collections::{HashMap, HashSet, VecDeque};

pub type SectorLocationMap = HashMap<SectorId, Vector3i>;

fn add_sector_node(
	node: DagNodeId,
	origin: Vector3i,
	grid: &mut CellGrid,
	sector_map: &mut SectorLocationMap,
) {
	let cell = grid.entry_or_default(origin);
	cell.sector = node;
	cell_add_attributes(cell, cell_attribute::PATH);
	sector_map.insert(node, origin);
}

fn get_neighbor_options(grid: &CellGrid, origin: Vector3i) -> Vec<Vector3i> {
	let mut options = Vec::new();
	for offset in get_surrounding_offsets() {
		let location = origin + offset;
		if !grid.contains(location) {
			options.push(location);
		}
	}
	options
}

fn select_distant_cell(origin: Vector3i, grid: &CellGrid, dice: &mut Dice) -> Vector3i {
	let direction = dice.get_int(5);
	let direction_vector = get_grid_direction_vector(direction as i8);
	for i in 0..1000 {
		let location = origin + direction_vector * i;
		if !grid.contains(location) {
			return location;
		}
	}
	Vector3i::none()
}

/// Converts abstract story nodes to spatial cluster cells.
pub fn story_to_clusters(
	dice: &mut Dice,
	graph: &DagGraph,
	origin: Vector3i,
	grid: &mut CellGrid,
	sector_map: &mut SectorLocationMap,
	portal_connections: &mut HashSet<DagConnectionId>,
) -> bool {
	let mut queue = VecDeque::new();
	queue.push_back(graph.root);

	add_sector_node(graph.root, origin, grid, sector_map);

	while let Some(node) = queue.pop_front() {
		let location = sector_map[&node];
		let descendants = get_immediate_node_descendants(&graph.connections, node);
		let mut location_options = get_neighbor_options(grid, location);

		for (key, _) in descendants {
			let child = key.1;
			if sector_map.contains_key(&child) {
				continue;
			}

			let neighbor_location = if !location_options.is_empty() {
				let neighbor = dice.remove_from_vec(&mut location_options);
				// Match C++ order: connect before the neighbor cell exists (no-op if missing).
				connect_cells_with_path(grid, location, neighbor);
				neighbor
			} else {
				let neighbor = select_distant_cell(location, grid, dice);
				portal_connections.insert((node, child));
				neighbor
			};

			if neighbor_location.is_none() {
				return false;
			}

			add_sector_node(child, neighbor_location, grid, sector_map);
			queue.push_back(child);
		}
	}

	true
}
