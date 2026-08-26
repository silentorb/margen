use margen_generation::biomes::Biome;
use margen_generation::core::world_cell::CellGrid;
use crate::clusters::graphing_utility::scale_cell_location;
use crate::types::NewCellContext;
use crate::winding_path::grid_population::{add_connected_block, Block};
use mythic_math::Vector3i;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct ClusterRasterizationConfig {
	pub radius: i32,
	pub margin: i32,
	pub headroom: i32,
}

impl Default for ClusterRasterizationConfig {
	fn default() -> Self {
		Self {
			radius: 3,
			margin: 0,
			headroom: 3,
		}
	}
}

/// Radius includes the center cell — a grid of radius 1 contains a single cell.
pub fn rasterize_cluster(
	origin: Vector3i,
	radius: i32,
	block: &Block,
	context: &NewCellContext,
	grid: &mut CellGrid,
) {
	if radius < 0 {
		return;
	}
	let length = radius - 1;
	for column in -length..=length {
		let column_length = length * 2 - column.abs() + 1;
		let row_start = if column >= 0 {
			-length
		} else {
			-length - column
		};
		for row in 0..column_length {
			let location = origin + Vector3i::new(column, row_start + row, origin.z);
			add_connected_block(grid, location, context, block);
		}
	}
}

pub fn rasterize_cluster_grid(
	get_biome: &mut dyn FnMut(Vector3i) -> Arc<Biome>,
	input: &CellGrid,
	config: &ClusterRasterizationConfig,
	block: &Block,
) -> CellGrid {
	let mut output = CellGrid::new();
	let radius = config.radius;
	let room_radius = radius - config.margin;

	for (location, cell) in input.iter() {
		let origin = scale_cell_location(*location, radius as f32);
		let context = NewCellContext::new(get_biome(origin), cell.step, cell.sector);
		rasterize_cluster(origin, room_radius, block, &context, &mut output);
	}

	output
}
