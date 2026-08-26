pub mod common_blocks;
pub mod congruence;
pub mod grid_population;
pub mod growth;
pub mod path_management;
pub mod path_shaping;
pub mod types;

pub use common_blocks::{winding_path_block, winding_path_block_default};
pub use congruence::{
	block_space_is_available, can_cells_be_connected, can_place_block, cell_has_no_flat_slope_conflicts,
	cell_has_viable_paths, gather_neighbor_cell_paths_array, new_solving_cell_from_biome,
	new_solving_cell_from_world, CongruenceError, SolvingCell,
};
pub use grid_population::{
	add_block, add_connected_block, block_has_available_space, connect_cell_neighbors,
	connect_cells_with_path, connect_cells_with_path_cells, connect_cells_with_paths,
	create_new_cell, try_add_block, Block, BlockLayer,
};
pub use growth::winding_path;
pub use path_management::{
	gather_available_locations, gather_initial_available_locations, get_possible_location,
	update_possible_locations,
};
pub use path_shaping::get_cell_priority;
pub use types::{CellPriority, LocationPools, SolvingState};
