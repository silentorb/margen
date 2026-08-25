use crate::biomes::{constant_get_biome, default_biome, Biome};
use crate::core::world_cell::CellGrid;
use crate::dependency::{DagConnectionId, DagGraph, DagNodeId};
use crate::story::{generate_story_graph, story_to_location_branching};
use crate::structure::clusters::{
	connect_cluster_cells, rasterize_cluster_grid, ClusterRasterizationConfig,
};
use crate::structure::story::story_to_clusters::{story_to_clusters, SectorLocationMap};
use crate::structure::winding_path::winding_path_block_default;
use mythic::{Dice, Vector3i};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct StoryGridConfig {
	pub headroom: i32,
	pub raster: ClusterRasterizationConfig,
	pub connect_clusters: bool,
	pub biome: Arc<Biome>,
}

impl Default for StoryGridConfig {
	fn default() -> Self {
		Self {
			headroom: 3,
			raster: ClusterRasterizationConfig {
				radius: 3,
				margin: 1,
				headroom: 3,
			},
			connect_clusters: false,
			biome: default_biome(),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct StoryGridResult {
	pub graph: DagGraph,
	pub grid: CellGrid,
	pub sector_location_map: SectorLocationMap,
	pub portal_connections: HashSet<DagConnectionId>,
	pub success: bool,
}

pub fn generate_story_grid(
	dice: &mut Dice,
	node_count: DagNodeId,
	origin: Vector3i,
) -> StoryGridResult {
	generate_story_grid_with_config(dice, node_count, origin, &StoryGridConfig::default())
}

pub fn generate_story_grid_with_config(
	dice: &mut Dice,
	node_count: DagNodeId,
	origin: Vector3i,
	config: &StoryGridConfig,
) -> StoryGridResult {
	let mut result = StoryGridResult::default();

	let story_graph = generate_story_graph(dice, node_count);
	let branching_connections = story_to_location_branching(dice, &story_graph.graph, 4);

	result.graph.root = story_graph.graph.root;
	result.graph.nodes = story_graph.graph.nodes;
	result.graph.connections = branching_connections;

	result.success = story_to_clusters(
		dice,
		&result.graph,
		origin,
		&mut result.grid,
		&mut result.sector_location_map,
		&mut result.portal_connections,
	);

	if !result.success {
		return result;
	}

	let mut raster_config = config.raster;
	raster_config.headroom = config.headroom;
	let block = winding_path_block_default(config.headroom);
	let get_biome = constant_get_biome(Arc::clone(&config.biome));
	result.grid = rasterize_cluster_grid(&get_biome, &result.grid, &raster_config, &block);

	if config.connect_clusters {
		connect_cluster_cells(
			dice,
			&result.graph.connections,
			&result.sector_location_map,
			&get_biome,
			raster_config.radius,
			&block,
			&mut result.grid,
		);
	}

	result
}

pub fn generate_story_grid_at_origin(dice: &mut Dice, node_count: DagNodeId) -> StoryGridResult {
	generate_story_grid(dice, node_count, Vector3i::zero())
}

/// Floor origins in a hex of room radius `r` (radius includes center; length = r - 1).
pub fn hex_room_floor_count(room_radius: i32) -> usize {
	if room_radius < 1 {
		return 0;
	}
	let length = room_radius - 1;
	let mut count = 0usize;
	for column in -length..=length {
		let column_length = length * 2 - column.abs() + 1;
		count += column_length as usize;
	}
	count
}

/// Cells per room for default winding_path_block (4 layers when headroom_padding >= 1).
pub fn cells_per_winding_block(headroom: i32) -> usize {
	let headroom_padding = (headroom - 2).max(0) as usize;
	1 + 1 + headroom_padding + 1
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::biomes::{constant_get_biome, default_biome};
	use crate::core::world_cell::{cell_attribute, CellGrid, WorldCell};
	use crate::structure::clusters::{
		rasterize_cluster_grid, scale_cell_location, ClusterRasterizationConfig,
	};
	use crate::structure::winding_path::winding_path_block_default;

	#[test]
	fn story_grid_expands_after_rasterize() {
		let config = StoryGridConfig::default();
		let floors = hex_room_floor_count(config.raster.radius - config.raster.margin);
		let per_block = cells_per_winding_block(config.headroom);
		assert_eq!(floors, 7);
		assert_eq!(per_block, 4);

		for node_count in 3..=8 {
			for seed in 1..=3 {
				let mut dice = Dice::new(seed);
				let result = generate_story_grid_at_origin(&mut dice, node_count);
				assert!(result.success, "seed={seed} nodes={node_count}");
				assert_eq!(
					result.grid.len(),
					node_count as usize * floors * per_block,
					"seed={seed} nodes={node_count}"
				);
				assert!(result.portal_connections.is_empty());
				assert!(result
					.grid
					.iter()
					.any(|(_, c)| (c.attributes & cell_attribute::PATH) != 0));
				assert!(result
					.grid
					.iter()
					.any(|(_, c)| (c.attributes & cell_attribute::SOLID) != 0));
			}
		}
	}

	#[test]
	fn rasterize_single_cell_footprint() {
		let mut input = CellGrid::new();
		input.insert(Vector3i::zero(), WorldCell {
			sector: 1,
			..Default::default()
		});
		let config = ClusterRasterizationConfig {
			radius: 3,
			margin: 1,
			headroom: 3,
		};
		let block = winding_path_block_default(3);
		let biome = default_biome();
		let get_biome = constant_get_biome(biome);
		let output = rasterize_cluster_grid(&get_biome, &input, &config, &block);
		assert_eq!(output.len(), 7 * 4);
	}

	#[test]
	fn connect_clusters_adds_corridor_cells() {
		let mut dice = Dice::new(1);
		let mut config = StoryGridConfig::default();
		config.connect_clusters = true;
		let without = generate_story_grid_with_config(&mut Dice::new(1), 3, Vector3i::zero(), &StoryGridConfig::default());
		let with = generate_story_grid_with_config(&mut dice, 3, Vector3i::zero(), &config);
		assert!(with.success);
		assert!(with.grid.len() >= without.grid.len());
	}

	#[test]
	fn scale_cell_location_known() {
		assert_eq!(
			scale_cell_location(Vector3i::new(1, 0, 0), 3.0),
			Vector3i::new(5, -2, 0)
		);
	}
}
