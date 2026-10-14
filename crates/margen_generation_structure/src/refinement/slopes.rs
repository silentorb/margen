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

	#[test]
	fn no_upward_paths_yields_empty_map() {
		let mut grid = CellGrid::new();
		grid.insert(
			Vector3i::zero(),
			path_cell_with(&[directions::NORTH]),
		);
		assert!(apply_path_slopes(&grid).is_empty());
	}

	fn two_cell_up_slope(up_dir: DirectionIndex) -> (CellGrid, Vector3i) {
		use margen_generation::core::directions::{
			get_grid_direction_vector, get_opposite_direction_index,
		};
		let origin = Vector3i::zero();
		let above = origin + get_grid_direction_vector(up_dir);
		let mut grid = CellGrid::new();
		grid.insert(origin, path_cell_with(&[up_dir]));
		grid.insert(
			above,
			path_cell_with(&[get_opposite_direction_index(up_dir)]),
		);
		(grid, origin)
	}

	fn assert_upper_ring_scales(offsets: &HexCellCorners, expected: [f32; 6]) {
		for (i, &want) in expected.iter().enumerate() {
			let corner = i + 6;
			assert_eq!(
				offsets[corner].z, want,
				"upper corner {corner}: got {} want {want} (full {offsets:?})",
				offsets[corner].z
			);
		}
		assert_eq!(offsets[MIDDLE_CENTER_UPPER].z, 0.0);
		for i in 0..6 {
			assert_eq!(offsets[i].z, 1.0, "lower corner {i}");
		}
	}

	#[test]
	fn single_upward_path_sets_center_upper_to_zero() {
		let (grid, origin) = two_cell_up_slope(directions::UP_NORTH);
		let map = apply_path_slopes(&grid);
		let offsets = map.get(&origin).expect("slope cell");
		assert_eq!(offsets[MIDDLE_CENTER_UPPER].z, 0.0);
		for i in 0..6 {
			assert_eq!(offsets[i].z, 1.0, "lower corner {i}");
		}
	}

	/// Upper-ring scales for a lone two-cell upward slope: edge toward `yaw` stays 1, else 0.
	fn expected_lone_up_ring(yaw: DirectionIndex) -> [f32; 6] {
		let mut expected = [0.0_f32; 6];
		let a = ((yaw + 5) % 6) as usize;
		let b = (yaw % 6) as usize;
		expected[a] = 1.0;
		expected[b] = 1.0;
		expected
	}

	#[test]
	fn each_upward_direction_keeps_its_edge_high() {
		for yaw in 0..6i8 {
			let up = yaw + directions::UP_NORTH;
			let (grid, origin) = two_cell_up_slope(up);
			let map = apply_path_slopes(&grid);
			let offsets = map.get(&origin).expect("slope cell");
			assert_upper_ring_scales(offsets, expected_lone_up_ring(yaw));
		}
	}

	#[test]
	fn has_upward_paths_detects_bits() {
		assert!(has_upward_paths(1u32 << directions::UP_NORTH));
		assert!(has_upward_paths(1u32 << directions::UP));
		assert!(!has_upward_paths(1u32 << directions::NORTH));
	}

	fn assert_shared_upper_vertex_z_congruity(grid: &CellGrid) {
		use margen_generation::core::hex_corners::{
			apply_relative_cell_offsets, get_default_corner_offsets, resolve_cell_coordinates,
		};
		use margen_generation::core::shared_cell_vertices::SharedCellCorner;
		use margen_generation::core::world_cell::cell_is_path;
		use std::collections::HashMap;

		let hex_map = apply_path_slopes(grid);
		let mut by_key: HashMap<(i32, i32, i32, i8), Vec<f32>> = HashMap::new();

		for (location, cell) in grid.iter() {
			if !cell_is_path(cell) {
				continue;
			}
			let mut corners = get_default_corner_offsets();
			if let Some(offsets) = hex_map.get(location) {
				apply_relative_cell_offsets(offsets, &mut corners);
			}
			let center = resolve_cell_coordinates(*location);
			for local in 6..12i8 {
				let shared = SharedCellCorner::new(*location, local);
				if shared.corner_index < 0 {
					continue;
				}
				let key = (
					shared.location.x,
					shared.location.y,
					shared.location.z,
					shared.corner_index,
				);
				let world_z = center.z + corners[local as usize].z;
				by_key.entry(key).or_default().push(world_z);
			}
		}

		for (key, zs) in by_key {
			if zs.len() < 2 {
				continue;
			}
			let first = zs[0];
			for &z in &zs[1..] {
				assert!(
					(z - first).abs() < 1e-4,
					"shared corner {key:?} Z mismatch: {zs:?}"
				);
			}
		}
	}

	#[test]
	fn each_lone_up_slope_has_congruent_shared_upper_vertices() {
		for yaw in 0..6i8 {
			let (grid, _) = two_cell_up_slope(yaw + directions::UP_NORTH);
			assert_shared_upper_vertex_z_congruity(&grid);
		}
	}

	#[test]
	fn upward_fan_has_congruent_shared_upper_vertices() {
		use margen_generation::core::directions::{
			get_grid_direction_vector, get_opposite_direction_index,
		};
		let origin = Vector3i::zero();
		let mut grid = CellGrid::new();
		grid.insert(
			origin,
			path_cell_with(&[directions::UP_NORTH, directions::UP_NORTHEAST]),
		);
		for up in [directions::UP_NORTH, directions::UP_NORTHEAST] {
			let dest = origin + get_grid_direction_vector(up);
			grid.insert(dest, path_cell_with(&[get_opposite_direction_index(up)]));
		}
		assert_shared_upper_vertex_z_congruity(&grid);
	}

	/// Unreal axial `(x,y,z)` → RH lattice automorphism `(x, -x-y, z)`.
	fn from_unreal_axial(x: i32, y: i32, z: i32) -> Vector3i {
		Vector3i::new(x, -x - y, z)
	}

	#[test]
	fn unreal_slope_variant1_congruent_links_keep_upper_vertices_aligned() {
		// USlope_Variant1 cells, transformed UE → RH. Only place links that pass
		// congruence (forced chains can encode impossible pitch pairs).
		use crate::winding_path::congruence::{
			can_cells_be_connected, new_solving_cell_from_world, CongruenceError,
		};
		use crate::winding_path::grid_population::connect_cells_with_path;
		use margen_generation::core::directions::get_direction_index_from_vector;

		let points: Vec<Vector3i> = [
			(-1, 1, 1),
			(-2, 1, 1),
			(-2, 2, 2),
			(-3, 3, 3),
		]
		.into_iter()
		.map(|(x, y, z)| from_unreal_axial(x, y, z))
		.collect();
		let mut grid = CellGrid::new();
		for &p in &points {
			grid.insert(p, path_cell_with(&[]));
		}
		for i in 0..points.len() {
			for j in (i + 1)..points.len() {
				let delta = points[j] - points[i];
				let dir = get_direction_index_from_vector(delta);
				if dir < 0 {
					continue;
				}
				let a = new_solving_cell_from_world(grid.get(points[i]).unwrap());
				let b = new_solving_cell_from_world(grid.get(points[j]).unwrap());
				if can_cells_be_connected(&a, &b, dir) == CongruenceError::None {
					connect_cells_with_path(&mut grid, points[i], points[j]);
				}
			}
		}
		assert_shared_upper_vertex_z_congruity(&grid);
	}

	#[test]
	fn unreal_slope_spiral2_chain_has_congruent_upper_vertices() {
		let points: Vec<Vector3i> = [(2, 1, 2), (2, 2, 3), (3, 1, 1), (3, 0, 1)]
			.into_iter()
			.map(|(x, y, z)| from_unreal_axial(x, y, z))
			.collect();
		// Spiral2 is not a single chain — connect pairwise neighbors only.
		use crate::winding_path::grid_population::connect_cells_with_path;
		let mut grid = CellGrid::new();
		for &p in &points {
			grid.insert(p, path_cell_with(&[]));
		}
		for i in 0..points.len() {
			for j in (i + 1)..points.len() {
				let delta = points[j] - points[i];
				if margen_generation::core::directions::get_direction_index_from_vector(delta) >= 0
				{
					connect_cells_with_path(&mut grid, points[i], points[j]);
				}
			}
		}
		assert_shared_upper_vertex_z_congruity(&grid);
	}
}
