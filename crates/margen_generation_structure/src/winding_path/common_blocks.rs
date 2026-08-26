use margen_generation::core::world_cell::{cell_attribute, CellAttributes};
use crate::winding_path::grid_population::{Block, BlockLayer};

pub fn winding_path_block(headroom: i32, slot_attributes: CellAttributes) -> Block {
	// The lowest cell in the block is solid ground and isn't counted as headroom.
	let headroom_padding = headroom - 2;
	Block::new(
		[
			BlockLayer::new(cell_attribute::SOLID),
			BlockLayer::new(cell_attribute::PATH | slot_attributes),
			BlockLayer::with_count(cell_attribute::HEADROOM, headroom_padding),
			BlockLayer::new(cell_attribute::HEADROOM | slot_attributes),
		],
		-1,
	)
}

pub fn winding_path_block_default(headroom: i32) -> Block {
	winding_path_block(headroom, cell_attribute::SLOTS)
}
