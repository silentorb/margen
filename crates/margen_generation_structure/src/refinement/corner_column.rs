//! Corner-column queries for path slopes (and later wall-shift).
//!
//! Fresh types: a column is cell XY + flat corner direction + Z span — not Unreal's
//! packed `FIntVector` key. Outcomes match GenerationStructure corner-column space/pathing.

use margen_generation::core::directions::{
	get_direction_index_from_vector, get_grid_direction_vector, rotate_direction_index, DirectionIndex,
	FLAT_DIRECTION_COUNT,
};
use margen_generation::core::pathing::CellPaths;
use margen_generation::core::shared_cell_vertices::SharedCellCorner;
use margen_generation::core::world_cell::{
	cell_has_attribute, cell_is_path, cell_is_solid, cell_attribute, CellGrid,
};
use mythic_math::{Vector2i, Vector3i};

const CORNER_SIDE_COUNT: usize = 3;

/// Vertical span at a shared hex corner (XY root + flat corner direction).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CornerColumn {
	pub origin_xy: Vector2i,
	/// Flat direction `0..6` identifying which corner (after canonicalization).
	pub corner_dir: DirectionIndex,
	pub bottom: i32,
	pub top: i32,
}

#[derive(Clone, Debug, Default)]
pub struct CornerColumnSpace {
	/// Side indices (`0..3`) that have a path cell in the Z span.
	pub path_sides: Vec<u8>,
	pub path_locations: Vec<Vector3i>,
	/// Side indices that have a solid wall cell in the Z span.
	pub wall_sides: Vec<u8>,
}

fn flat_direction_xy(direction: DirectionIndex) -> Vector2i {
	let v = get_grid_direction_vector(direction % FLAT_DIRECTION_COUNT);
	Vector2i::new(v.x, v.y)
}

/// Three XY offsets for cells meeting at a corner (root, +dir, +rotate_cw(dir)).
fn corner_side_offsets(corner_dir: DirectionIndex) -> [Vector2i; CORNER_SIDE_COUNT] {
	let d0 = corner_dir % FLAT_DIRECTION_COUNT;
	[
		Vector2i::zero(),
		flat_direction_xy(d0),
		flat_direction_xy(rotate_direction_index(d0, 1)),
	]
}

/// Column for slope inspection at `location`'s horizontal corner `direction` (`0..6`).
///
/// Reuses [`SharedCellCorner`] canonicalization so the same physical corner converges
/// on one XY root; Z span is always `location.z - 1 ..= location.z + 1`.
pub fn slope_corner_column(location: Vector3i, direction: DirectionIndex) -> CornerColumn {
	let shared = SharedCellCorner::new(location, direction);
	CornerColumn {
		origin_xy: Vector2i::new(shared.location.x, shared.location.y),
		corner_dir: shared.corner_index % FLAT_DIRECTION_COUNT,
		bottom: location.z - 1,
		top: location.z + 1,
	}
}

pub fn corner_column_space(grid: &CellGrid, column: CornerColumn) -> CornerColumnSpace {
	let mut space = CornerColumnSpace::default();
	let offsets = corner_side_offsets(column.corner_dir);

	for (side, offset) in offsets.iter().enumerate() {
		let location_2d = column.origin_xy + *offset;
		let mut has_path = false;
		let mut has_wall = false;
		let mut path_location = Vector3i::zero();

		for h in column.bottom..=column.top {
			let neighbor = Vector3i::new(location_2d.x, location_2d.y, h);
			if let Some(cell) = grid.get(neighbor) {
				if cell_is_path(cell) {
					has_path = true;
					path_location = neighbor;
				}
				if cell_is_solid(cell)
					&& !cell_has_attribute(cell, cell_attribute::INDEPENDENT_CELL_MESH)
				{
					has_wall = true;
				}
			}
		}

		if has_path {
			space.path_sides.push(side as u8);
			space.path_locations.push(path_location);
		}
		if has_wall {
			space.wall_sides.push(side as u8);
		}
	}

	space
}

fn cells_connected_by_path(origin: Vector3i, neighbor: Vector3i, paths: CellPaths) -> bool {
	let dir = get_direction_index_from_vector(neighbor - origin);
	dir >= 0 && (paths & (1u32 << dir)) != 0
}

fn max_node_connections(node_count: usize) -> usize {
	node_count.saturating_mul(node_count.saturating_sub(1)) / 2
}

/// Side-pair indices among `path_locations` that are path-connected (outcome of UE corner pathing).
pub fn corner_connected_path_sides(
	grid: &CellGrid,
	path_locations: &[Vector3i],
) -> Vec<u8> {
	let n = path_locations.len();
	let steps = max_node_connections(n);
	let mut connected = Vec::new();

	for i in 0..steps {
		let Some(cell) = grid.get(path_locations[i]) else {
			continue;
		};
		let a = path_locations[i];
		let b = path_locations[(i + 1) % n];
		if cells_connected_by_path(a, b, cell.paths) {
			connected.push(i as u8);
		}
	}

	connected
}
