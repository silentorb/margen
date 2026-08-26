use crate::types::NewCellContext;
use crate::winding_path::common_blocks::winding_path_block;
use crate::winding_path::congruence::{can_place_block, new_solving_cell_from_biome};
use crate::winding_path::grid_population::{add_connected_block, Block};
use crate::winding_path::path_management::{
	gather_initial_available_locations, get_possible_location, update_possible_locations,
};
use crate::winding_path::path_shaping::get_cell_priority;
use crate::winding_path::types::{CellPriority, LocationPools, SolvingState};
use margen_generation::biomes::Biome;
use margen_generation::core::world_cell::{cell_attribute, CellGrid};
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::sync::Arc;

struct BlockLibrary {
	pathway: Block,
}

fn new_block_library(headroom: i32) -> BlockLibrary {
	BlockLibrary {
		pathway: winding_path_block(headroom, cell_attribute::SLOTS),
	}
}

/// Grow a `CellGrid` by repeatedly placing winding-path blocks (Unreal `windingPath`).
pub fn winding_path(
	dice: &mut Dice,
	grid: &mut CellGrid,
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	headroom: i32,
	step: &mut u32,
	max_blocks: i32,
) {
	let mut iteration = 1i32;
	let blocks = new_block_library(headroom);
	let mut location_pools = LocationPools::default();

	{
		let state = SolvingState { grid, headroom };
		gather_initial_available_locations(get_biome, state, &mut location_pools, None);
	}

	while *step <= max_blocks as u32 {
		let Some(location) = get_possible_location(dice, &mut location_pools) else {
			return;
		};

		let biome = get_biome(location);
		let solving = new_solving_cell_from_biome(&biome);
		let can_place = can_place_block(grid, headroom, &solving, location);

		if can_place {
			let priority = {
				let state = SolvingState { grid, headroom };
				get_cell_priority(get_biome, state, location)
			};

			if !location_pools.primary.is_empty() && priority != CellPriority::Primary {
				location_pools.fallback.push(location);
			} else {
				let context = NewCellContext::new(biome, *step, 0);
				add_connected_block(grid, location, &context, &blocks.pathway);
				{
					let state = SolvingState { grid, headroom };
					update_possible_locations(
						get_biome,
						state,
						location,
						&mut location_pools,
						None,
					);
				}
				*step += 1;
			}
		}

		iteration += 1;
		if iteration > 10_000 {
			break;
		}
	}
}
