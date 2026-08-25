//! Engine-agnostic maze-like world generation.

pub mod biomes;
pub mod core;
pub mod dependency;
pub mod distribution;
pub mod graphing;
pub mod story;
pub mod structure;

pub use biomes::{
	biome_from_id, default_biome, Biome, BiomeDistribution, BiomeNodeId, BiomeQueryResult,
	BIOME_TAG_FLAT,
};
pub use core::{
	cell_attribute, sector_attribute, CellAttributes, CellGrid, CellPaths, DirectionIndex, Sector,
	SectorAttributes, SectorData, SectorId, WorldCell, ALL_DIRECTIONS_COUNT, FLAT_DIRECTION_COUNT,
};
pub use dependency::{
	get_immediate_node_ancestors, get_immediate_node_ancestors_count, get_immediate_node_descendants,
	get_node_ancestors, get_node_descendants, DagConnection, DagConnectionId, DagConnectionMap,
	DagGraph, DagNodeId,
};
pub use graphing::{sectors_to_dag, SimpleDagGraph, SimpleDagNode};
pub use story::{
	generate_story_graph, story_connection_attribute, story_to_location_branching,
	story_to_location_branching_default, StoryGraph,
};
pub use structure::{
	connect_cells_with_path, connect_cells_with_paths, connect_cluster_cells, generate_story_grid,
	generate_story_grid_with_config, rasterize_cluster_grid, story_to_clusters,
	ClusterRasterizationConfig, SectorLocationMap, StoryGridConfig, StoryGridResult,
};
