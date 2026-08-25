use crate::biomes::Biome;
use crate::core::directions::get_surrounding_offsets;
use crate::core::hex_shape::get_hex_distance_2d;
use crate::core::world_cell::CellGrid;
use crate::dependency::{DagConnectionMap, DagNodeId};
use crate::structure::clusters::graphing_utility::scale_cell_location;
use crate::structure::story::story_to_clusters::SectorLocationMap;
use crate::structure::types::NewCellContext;
use crate::structure::winding_path::grid_population::{add_connected_block, Block};
use mythic::{Dice, Vector3i};
use std::sync::Arc;

struct ClusterInfo {
	center: Vector3i,
	biome: Arc<Biome>,
	node: DagNodeId,
}

fn get_nearest_neighbor(dice: &mut Dice, origin: Vector3i, target: Vector3i) -> Vector3i {
	let mut options = Vec::new();
	let mut shortest_distance = 100_000;

	for offset in get_surrounding_offsets() {
		let location = origin + offset;
		let distance = get_hex_distance_2d(location, target);
		if distance <= shortest_distance {
			if distance < shortest_distance {
				options.clear();
			}
			options.push(location);
			shortest_distance = distance;
		}
	}

	dice.from_slice(&options)
}

fn draw_hex_line(dice: &mut Dice, clusters: &[ClusterInfo; 2], block: &Block, grid: &mut CellGrid) {
	let distance = get_hex_distance_2d(clusters[0].center, clusters[1].center);
	let mut current_location = clusters[0].center;
	let steps = distance - 1;

	for _ in 0..steps {
		current_location = get_nearest_neighbor(dice, current_location, clusters[1].center);
		if !grid.contains(current_location) {
			// Port Unreal comparison as-is (both distances use clusters[0]).
			let nearest_cluster = if get_hex_distance_2d(current_location, clusters[0].center)
				<= get_hex_distance_2d(current_location, clusters[0].center)
			{
				&clusters[0]
			} else {
				&clusters[1]
			};
			let context = NewCellContext::new(Arc::clone(&nearest_cluster.biome), 0, nearest_cluster.node);
			add_connected_block(grid, current_location, &context, block);
		}
	}
}

pub fn connect_cluster_cells(
	dice: &mut Dice,
	connections: &DagConnectionMap,
	sector_location_map: &SectorLocationMap,
	get_biome: &dyn Fn(Vector3i) -> Arc<Biome>,
	radius: i32,
	block: &Block,
	grid: &mut CellGrid,
) {
	for &(a, b) in connections.keys() {
		let Some(&location_a) = sector_location_map.get(&a) else {
			continue;
		};
		let Some(&location_b) = sector_location_map.get(&b) else {
			continue;
		};
		let center_a = scale_cell_location(location_a, radius as f32);
		let center_b = scale_cell_location(location_b, radius as f32);
		let biome_a = get_biome(center_a);
		let biome_b = get_biome(center_b);
		let clusters = [
			ClusterInfo {
				center: center_a,
				biome: biome_a,
				node: a,
			},
			ClusterInfo {
				center: center_b,
				biome: biome_b,
				node: b,
			},
		];
		draw_hex_line(dice, &clusters, block, grid);
	}
}
