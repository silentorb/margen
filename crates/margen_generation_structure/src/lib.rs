//! Layout structure: clusters, winding path, story grid.

pub mod clusters;
pub mod querying;
pub mod story;
pub mod types;
pub mod winding_path;

pub use clusters::{
	connect_cluster_cells, rasterize_cluster, rasterize_cluster_grid, scale_cell_location,
	ClusterRasterizationConfig,
};
pub use querying::{get_path_locations, grid_to_neighbors_map, NeighborsMap};
pub use story::{
	generate_story_grid, generate_story_grid_with_config, story_to_clusters, SectorLocationMap,
	StoryGridConfig, StoryGridResult,
};
pub use winding_path::{
	connect_cells_with_path, connect_cells_with_paths, winding_path_block,
	winding_path_block_default, Block, BlockLayer,
};
