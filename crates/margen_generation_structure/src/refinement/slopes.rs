//! Path-slope corner Z scales → [`HexCellCornersMap`].
//!
//! Outcome-equivalent to Unreal `gatherCellSlopeCornerInfo` / `applySlopes` / `getSlopeOffsets`,
//! without mirroring UE control flow or dead branches.

use crate::refinement::corner_column::{
	corner_column_space, corner_connected_path_sides, slope_corner_column, CornerColumnSpace,
};
use margen_generation::core::directions::{directions, DirectionIndex};
use margen_generation::core::hex_corners::{HexCellCorners, HexCellCornersMap, HEX_CELL_CORNER_COUNT};
use margen_generation::core::pathing::CellPaths;
use margen_generation::core::world_cell::CellGrid;
use mythic_math::{Vector3, Vector3i};

const MIDDLE_CENTER_UPPER: usize = 13;

#[derive(Clone, Debug)]
struct SlopeCornerInfo {
	space: CornerColumnSpace,
	connected_paths: Vec<u8>,
	/// Upper-ring corner index (`6..12`).
	corner: usize,
}

fn has_upward_paths(paths: CellPaths) -> bool {
	for d in directions::UP_NORTH..=directions::UP_NORTHWEST {
		if (paths & (1u32 << d)) != 0 {
			return true;
		}
	}
	(paths & (1u32 << directions::UP)) != 0
}

fn is_upper_slope_corner(
	path_locations: &[Vector3i],
	connected_paths: &[u8],
	z: i32,
) -> bool {
	let n = path_locations.len();
	if n == 0 {
		return false;
	}
	for &connected in connected_paths {
		let i = connected as usize;
		let first = path_locations[i % n];
		let second = path_locations[(i + 1) % n];
		if first.z == z || second.z == z {
			return true;
		}
	}
	false
}

fn slope_offsets_for_cell(z: i32, corners: &[SlopeCornerInfo]) -> HexCellCorners {
	let mut vertical = [1.0_f32; HEX_CELL_CORNER_COUNT];

	for corner_info in corners {
		if !is_upper_slope_corner(
			&corner_info.space.path_locations,
			&corner_info.connected_paths,
			z + 1,
		) {
			vertical[corner_info.corner] = if corner_info.space.path_sides.len() == 1 {
				0.0
			} else {
				-1.0
			};
		}
	}

	vertical[MIDDLE_CENTER_UPPER] = 0.0;

	let mut result = [Vector3::zero(); HEX_CELL_CORNER_COUNT];
	for (i, &scale_z) in vertical.iter().enumerate() {
		result[i] = Vector3::new(1.0, 1.0, scale_z);
	}
	result
}

fn gather_cell_slope_corners(grid: &CellGrid, location: Vector3i) -> Vec<SlopeCornerInfo> {
	let mut result = Vec::with_capacity(6);
	for direction in 0..6 {
		let direction = direction as DirectionIndex;
		let corner = (direction + 6) as usize;
		let column = slope_corner_column(location, direction);
		let space = corner_column_space(grid, column);
		let connected_paths = corner_connected_path_sides(grid, &space.path_locations);
		result.push(SlopeCornerInfo {
			space,
			connected_paths,
			corner,
		});
	}
	result
}

/// Build hex corner Z scales for every cell that has upward paths.
pub fn apply_path_slopes(grid: &CellGrid) -> HexCellCornersMap {
	let mut hex_map = HexCellCornersMap::new();

	for (location, cell) in grid.iter() {
		if !has_upward_paths(cell.paths) {
			continue;
		}
		let corners = gather_cell_slope_corners(grid, *location);
		let offsets = slope_offsets_for_cell(location.z, &corners);
		hex_map.insert(*location, offsets);
	}

	hex_map
}

/// True when `paths` includes any upward slope or vertical-up bit.
pub fn cell_has_upward_paths(paths: CellPaths) -> bool {
	has_upward_paths(paths)
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_generation::core::world_cell::{
		cell_add_attributes, cell_add_path, cell_attribute, WorldCell,
	};
	fn path_cell_with(paths: &[DirectionIndex]) -> WorldCell {
		let mut cell = WorldCell::default();
		cell_add_attributes(&mut cell, cell_attribute::PATH);
		for &d in paths {
			cell_add_path(&mut cell, d);
		}
		cell
	}

	fn solid_cell() -> WorldCell {
		let mut cell = WorldCell::default();
		cell_add_attributes(&mut cell, cell_attribute::SOLID);
		cell
	}

	#[test]
	fn no_upward_paths_yields_empty_map() {
		let mut grid = CellGrid::new();
		grid.insert(
			Vector3i::zero(),
			path_cell_with(&[directions::NORTH]),
		);
		assert!(apply_path_slopes(&grid).is_empty());
	}

	#[test]
	fn single_upward_path_sets_center_upper_to_zero() {
		let origin = Vector3i::zero();
		let above = Vector3i::new(0, 1, 1); // UP_NORTH neighbor
		let mut grid = CellGrid::new();
		grid.insert(origin, path_cell_with(&[directions::UP_NORTH]));
		grid.insert(above, path_cell_with(&[directions::DOWN_SOUTH]));

		let map = apply_path_slopes(&grid);
		let offsets = map.get(&origin).expect("slope cell");
		assert_eq!(offsets[MIDDLE_CENTER_UPPER].z, 0.0);
		// Lower ring stays at default scale 1.
		for i in 0..6 {
			assert_eq!(offsets[i].z, 1.0, "lower corner {i}");
		}
	}

	#[test]
	fn upper_path_neighbor_keeps_corner_at_one() {
		// Cell with upward path north; put a path at z+1 that participates in a connected
		// corner pair so is_upper_slope_corner returns true for that corner.
		let origin = Vector3i::zero();
		let up_north = Vector3i::new(0, 1, 1);
		let mut grid = CellGrid::new();

		let mut origin_cell = path_cell_with(&[directions::UP_NORTH]);
		cell_add_path(&mut origin_cell, directions::NORTH);
		grid.insert(origin, origin_cell);

		let mut up_cell = path_cell_with(&[directions::DOWN_SOUTH]);
		cell_add_path(&mut up_cell, directions::SOUTH);
		grid.insert(up_north, up_cell);

		// Flat north neighbor at same z so the corner column sees multiple path sides.
		grid.insert(
			Vector3i::new(0, 1, 0),
			path_cell_with(&[directions::SOUTH, directions::UP_SOUTH]),
		);

		let map = apply_path_slopes(&grid);
		let offsets = map.get(&origin).expect("slope cell");
		assert_eq!(offsets[MIDDLE_CENTER_UPPER].z, 0.0);

		// At least one upper-ring corner should remain 1 when neighboring upper path.
		let has_high = (6..12).any(|i| offsets[i].z == 1.0);
		assert!(has_high, "expected an upper corner at scale 1, got {offsets:?}");
	}

	#[test]
	fn lone_path_at_corner_sets_zero_scale() {
		// Isolated slope cell: only itself as path in most corner columns → scale 0.
		let origin = Vector3i::zero();
		let mut grid = CellGrid::new();
		grid.insert(origin, path_cell_with(&[directions::UP_NORTH]));
		// Destination of the upward path (needed for a realistic grid, but may not sit
		// in every corner column of origin).
		grid.insert(
			Vector3i::new(0, 1, 1),
			path_cell_with(&[directions::DOWN_SOUTH]),
		);
		grid.insert(Vector3i::new(0, 0, -1), solid_cell());

		let map = apply_path_slopes(&grid);
		let offsets = map.get(&origin).expect("slope cell");
		assert_eq!(offsets[MIDDLE_CENTER_UPPER].z, 0.0);

		let lowered = (6..12).filter(|&i| offsets[i].z <= 0.0).count();
		assert!(
			lowered >= 1,
			"expected at least one lowered upper corner, got {offsets:?}"
		);
	}

	#[test]
	fn has_upward_paths_detects_bits() {
		assert!(has_upward_paths(1u32 << directions::UP_NORTH));
		assert!(has_upward_paths(1u32 << directions::UP));
		assert!(!has_upward_paths(1u32 << directions::NORTH));
	}
}
