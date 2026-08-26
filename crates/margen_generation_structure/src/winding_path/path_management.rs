use crate::winding_path::congruence::{
	can_place_block, get_cell_max_path_directions, new_solving_cell_from_biome,
};
use crate::winding_path::path_shaping::get_cell_priority;
use crate::winding_path::types::{CellPriority, LocationPools, SolvingState};
use margen_generation::biomes::Biome;
use margen_generation::core::directions::get_grid_direction_vector;
use margen_generation::core::world_cell::{cell_is_path, CellAttributes};
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::collections::HashMap;
use std::sync::Arc;

fn select_target_cell_pool<'a>(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	state: SolvingState<'_>,
	location: Vector3i,
	location_pools: &'a mut LocationPools,
) -> &'a mut Vec<Vector3i> {
	match get_cell_priority(get_biome, state, location) {
		CellPriority::Primary => &mut location_pools.primary,
		_ => &mut location_pools.fallback,
	}
}

fn select_source_location_pool(location_pools: &mut LocationPools) -> &mut Vec<Vector3i> {
	if !location_pools.primary.is_empty() {
		&mut location_pools.primary
	} else {
		&mut location_pools.fallback
	}
}

pub fn gather_available_locations(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	state: SolvingState<'_>,
	origin: Vector3i,
	attributes: CellAttributes,
	location_pools: &mut LocationPools,
	mut cell_priorities: Option<&mut HashMap<Vector3i, CellPriority>>,
) {
	let direction_count = get_cell_max_path_directions(attributes);

	for direction in 0..direction_count {
		let location = origin + get_grid_direction_vector(direction);
		if location_pools.primary.contains(&location) {
			continue;
		}

		// Logic fix (`incorrect-traversal-endpoint`): Unreal gather used `origin` for the
		// solving cell; the growth loop correctly uses `location`. Match the loop.
		let biome = get_biome(location);
		let solving = new_solving_cell_from_biome(&biome);
		if can_place_block(state.grid, state.headroom, &solving, location) {
			let pool = select_target_cell_pool(get_biome, state, location, location_pools);
			pool.push(location);
		} else if let Some(priorities) = cell_priorities.as_deref_mut() {
			if !state.grid.contains(location) {
				priorities.insert(location, CellPriority::Blocked);
			}
		}
	}
}

pub fn gather_initial_available_locations(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	state: SolvingState<'_>,
	location_pools: &mut LocationPools,
	mut cell_priorities: Option<&mut HashMap<Vector3i, CellPriority>>,
) {
	let path_origins: Vec<(Vector3i, CellAttributes)> = state
		.grid
		.iter()
		.filter(|(_, cell)| cell_is_path(cell))
		.map(|(loc, cell)| (*loc, cell.attributes))
		.collect();

	for (origin, attributes) in path_origins {
		gather_available_locations(
			get_biome,
			state,
			origin,
			attributes,
			location_pools,
			cell_priorities.as_deref_mut(),
		);
	}
}

pub fn remove_location_from_pools(location_pools: &mut LocationPools, location: Vector3i) {
	location_pools.primary.retain(|l| *l != location);
	location_pools.fallback.retain(|l| *l != location);
}

pub fn remove_possible_location_column(
	headroom: i32,
	location: Vector3i,
	location_pools: &mut LocationPools,
) {
	for i in -headroom..headroom {
		remove_location_from_pools(location_pools, location + Vector3i::new(0, 0, i));
	}
}

pub fn update_possible_locations(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	state: SolvingState<'_>,
	location: Vector3i,
	location_pools: &mut LocationPools,
	cell_priorities: Option<&mut HashMap<Vector3i, CellPriority>>,
) {
	remove_possible_location_column(state.headroom, location, location_pools);
	if let Some(cell) = state.grid.get(location) {
		gather_available_locations(
			get_biome,
			state,
			location,
			cell.attributes,
			location_pools,
			cell_priorities,
		);
	}
}

pub fn get_possible_location(
	dice: &mut Dice,
	location_pools: &mut LocationPools,
) -> Option<Vector3i> {
	let options = select_source_location_pool(location_pools);
	if options.is_empty() {
		return None;
	}
	Some(dice.remove_from_vec(options))
}
