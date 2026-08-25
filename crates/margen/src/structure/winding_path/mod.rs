pub mod common_blocks;
pub mod congruence;
pub mod grid_population;

pub use common_blocks::{winding_path_block, winding_path_block_default};
pub use congruence::{
	can_cells_be_connected, gather_neighbor_cell_paths_array, new_solving_cell_from_world,
	CongruenceError, SolvingCell,
};
pub use grid_population::{
	add_block, add_connected_block, block_has_available_space, connect_cell_neighbors,
	connect_cells_with_path, connect_cells_with_path_cells, connect_cells_with_paths,
	create_new_cell, try_add_block, Block, BlockLayer,
};
