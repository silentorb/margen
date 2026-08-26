use crate::types::NewCellContext;
use crate::winding_path::common_blocks::winding_path_block_default;
use crate::winding_path::grid_population::add_connected_block;
use crate::winding_path::growth::winding_path;
use margen_generation::biomes::{
	constant_get_biome, default_biome, Biome, BiomeDistribution,
};
use margen_generation::core::world_cell::CellGrid;
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WorldGridConfig {
	pub max_blocks: i32,
	pub headroom: i32,
	/// Biome pool for spatial sampling. Empty or single entry uses a constant biome.
	pub biomes: Vec<Arc<Biome>>,
}

impl Default for WorldGridConfig {
	fn default() -> Self {
		Self {
			max_blocks: 40,
			headroom: 3,
			biomes: vec![default_biome()],
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct WorldGridResult {
	pub grid: CellGrid,
	pub player_start: Vector3i,
	pub step: u32,
}

fn place_minimal_seed(
	grid: &mut CellGrid,
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	headroom: i32,
	step: &mut u32,
) {
	let origin = Vector3i::zero();
	let biome = get_biome(origin);
	let context = NewCellContext::new(biome, *step, 0);
	let block = winding_path_block_default(headroom);
	add_connected_block(grid, origin, &context, &block);
	*step += 1;
}

/// Prefab-minimal seed + `winding_path` growth (Unreal `generateWorldGrid` with `UMinimalGenerator`).
pub fn generate_world_grid(dice: &mut Dice, config: &WorldGridConfig) -> WorldGridResult {
	let mut result = WorldGridResult {
		player_start: Vector3i::zero(),
		step: 1,
		..Default::default()
	};

	if config.biomes.len() <= 1 {
		let biome = config
			.biomes
			.first()
			.cloned()
			.unwrap_or_else(default_biome);
		let mut get_biome = constant_get_biome(biome);
		place_minimal_seed(
			&mut result.grid,
			&mut get_biome,
			config.headroom,
			&mut result.step,
		);
		winding_path(
			dice,
			&mut result.grid,
			&mut get_biome,
			config.headroom,
			&mut result.step,
			config.max_blocks,
		);
	} else {
		let mut distribution = BiomeDistribution::new(dice.clone(), config.biomes.clone());
		let mut get_biome = |loc: Vector3i| distribution.get_biome(loc);
		place_minimal_seed(
			&mut result.grid,
			&mut get_biome,
			config.headroom,
			&mut result.step,
		);
		winding_path(
			dice,
			&mut result.grid,
			&mut get_biome,
			config.headroom,
			&mut result.step,
			config.max_blocks,
		);
	}

	result
}

pub fn generate_world_grid_default(dice: &mut Dice, max_blocks: i32) -> WorldGridResult {
	let config = WorldGridConfig {
		max_blocks,
		..Default::default()
	};
	generate_world_grid(dice, &config)
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_generation::core::world_cell::{cell_attribute, cell_is_path};

	fn path_cell_count(grid: &CellGrid) -> usize {
		grid.iter().filter(|(_, c)| cell_is_path(c)).count()
	}

	fn headroom_column_overlap(grid: &CellGrid, headroom: i32) -> bool {
		let path_floors: Vec<Vector3i> = grid
			.iter()
			.filter(|(_, c)| cell_is_path(c))
			.map(|(loc, _)| *loc)
			.collect();
		for floor in &path_floors {
			for other in &path_floors {
				if floor == other {
					continue;
				}
				if floor.x == other.x && floor.y == other.y {
					let dz = (floor.z - other.z).abs();
					if dz < headroom {
						return true;
					}
				}
			}
		}
		false
	}

	#[test]
	fn world_grid_grows_connected_path_cells() {
		let mut dice = Dice::new(7);
		let config = WorldGridConfig {
			max_blocks: 12,
			headroom: 3,
			..Default::default()
		};
		let result = generate_world_grid(&mut dice, &config);
		assert!(path_cell_count(&result.grid) >= 2);
		assert!(result
			.grid
			.iter()
			.any(|(_, c)| (c.attributes & cell_attribute::SOLID) != 0));
		assert_eq!(result.player_start, Vector3i::zero());
		assert!(result.grid.contains(result.player_start) || path_cell_count(&result.grid) > 0);
		assert!(!headroom_column_overlap(&result.grid, config.headroom));
	}

	#[test]
	fn world_grid_is_deterministic_for_seed() {
		let config = WorldGridConfig {
			max_blocks: 20,
			headroom: 3,
			..Default::default()
		};
		let a = generate_world_grid(&mut Dice::new(42), &config);
		let b = generate_world_grid(&mut Dice::new(42), &config);
		assert_eq!(a.grid.len(), b.grid.len());
		assert_eq!(a.step, b.step);
		let mut locs_a: Vec<_> = a.grid.iter().map(|(l, _)| *l).collect();
		let mut locs_b: Vec<_> = b.grid.iter().map(|(l, _)| *l).collect();
		locs_a.sort_by_key(|v| (v.x, v.y, v.z));
		locs_b.sort_by_key(|v| (v.x, v.y, v.z));
		assert_eq!(locs_a, locs_b);
	}

	#[test]
	fn world_grid_respects_max_blocks_step() {
		let config = WorldGridConfig {
			max_blocks: 5,
			headroom: 3,
			..Default::default()
		};
		let result = generate_world_grid(&mut Dice::new(1), &config);
		assert!(result.step > 1);
		assert!(result.step <= (config.max_blocks as u32) + 1);
		assert_eq!(path_cell_count(&result.grid), (result.step - 1) as usize);
	}
}
