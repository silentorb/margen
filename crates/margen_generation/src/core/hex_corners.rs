use crate::core::shared_cell_vertices::{
	get_shared_vertex_offset, SharedCellCorner, SharedCellVertexOffsets,
};
use mythic_math::{Vector2, Vector3, Vector3i};
use std::collections::HashMap;

/// Last two corners are the lower and upper centers.
pub const HEX_CELL_CORNER_COUNT: usize = 14;

pub type HexCellCorners = [Vector3; HEX_CELL_CORNER_COUNT];
pub type HexCellCornersMap = HashMap<Vector3i, HexCellCorners>;

const SQUARE_ROOT_OF_THREE: f32 = 1.732_050_8;
const LONG_LEG: f32 = 1.0;
const SHORT_LEG: f32 = 1.0 / SQUARE_ROOT_OF_THREE;
const HYPOTENUSE: f32 = 2.0 / SQUARE_ROOT_OF_THREE;

const CORNERS_2D: [Vector2; 6] = [
	Vector2::new(SHORT_LEG, -LONG_LEG),
	Vector2::new(HYPOTENUSE, 0.0),
	Vector2::new(SHORT_LEG, LONG_LEG),
	Vector2::new(-SHORT_LEG, LONG_LEG),
	Vector2::new(-HYPOTENUSE, 0.0),
	Vector2::new(-SHORT_LEG, -LONG_LEG),
];

fn default_corner_offsets() -> HexCellCorners {
	[
		Vector3::new(CORNERS_2D[0].x, CORNERS_2D[0].y, -1.0),
		Vector3::new(CORNERS_2D[1].x, CORNERS_2D[1].y, -1.0),
		Vector3::new(CORNERS_2D[2].x, CORNERS_2D[2].y, -1.0),
		Vector3::new(CORNERS_2D[3].x, CORNERS_2D[3].y, -1.0),
		Vector3::new(CORNERS_2D[4].x, CORNERS_2D[4].y, -1.0),
		Vector3::new(CORNERS_2D[5].x, CORNERS_2D[5].y, -1.0),
		Vector3::new(CORNERS_2D[0].x, CORNERS_2D[0].y, 1.0),
		Vector3::new(CORNERS_2D[1].x, CORNERS_2D[1].y, 1.0),
		Vector3::new(CORNERS_2D[2].x, CORNERS_2D[2].y, 1.0),
		Vector3::new(CORNERS_2D[3].x, CORNERS_2D[3].y, 1.0),
		Vector3::new(CORNERS_2D[4].x, CORNERS_2D[4].y, 1.0),
		Vector3::new(CORNERS_2D[5].x, CORNERS_2D[5].y, 1.0),
		Vector3::new(0.0, 0.0, -1.0),
		Vector3::new(0.0, 0.0, 1.0),
	]
}

pub fn get_default_corner_offsets() -> HexCellCorners {
	default_corner_offsets()
}

pub fn get_corner_vector_2d(direction: usize) -> Vector2 {
	CORNERS_2D[direction % 6]
}

pub fn resolve_cell_coordinates(location: Vector3i) -> Vector3 {
	let base = Vector3::from_i32(location);
	Vector3::new(
		base.x * SHORT_LEG * 1.5,
		base.y + base.x * LONG_LEG / 2.0,
		base.z,
	)
}

pub fn resolve_cell_coordinates_scaled(cell_size: Vector3, location: Vector3i) -> Vector3 {
	resolve_cell_coordinates(location) * cell_size
}

pub fn resolve_relative_corner(
	half_size: Vector3,
	corner_offsets: &SharedCellVertexOffsets,
	location: Vector3i,
	corner: u8,
) -> Vector3 {
	let defaults = get_default_corner_offsets();
	let shared_offset =
		get_shared_vertex_offset(corner_offsets, SharedCellCorner::new(location, corner as i8), 0);
	(defaults[corner as usize] + shared_offset) * half_size
}

pub fn resolve_absolute_corner(
	cell_size: Vector3,
	corner_offsets: &SharedCellVertexOffsets,
	location: Vector3i,
	corner: u8,
) -> Vector3 {
	let offset = resolve_relative_corner(cell_size * 0.5, corner_offsets, location, corner);
	let center = resolve_cell_coordinates_scaled(cell_size, location);
	center + offset
}

fn get_ring_center(corners: &HexCellCorners, offset: usize) -> Vector3 {
	let mut result = Vector3::zero();
	for i in offset..(offset + 6) {
		result = result + corners[i];
	}
	result * (1.0 / 6.0)
}

fn set_top_and_bottom(corners: &mut HexCellCorners) {
	corners[12] = get_ring_center(corners, 0);
	corners[13] = get_ring_center(corners, 6);
}

/// Unit-space relative corners (no cell size scaling). Used by face arrange.
pub fn resolve_relative_cell_corner_coordinates(
	corner_offsets: &SharedCellVertexOffsets,
	location: Vector3i,
) -> HexCellCorners {
	let defaults = get_default_corner_offsets();
	let mut result = [Vector3::zero(); HEX_CELL_CORNER_COUNT];
	for corner in 0..12u8 {
		let shared_offset =
			get_shared_vertex_offset(corner_offsets, SharedCellCorner::new(location, corner as i8), 0);
		result[corner as usize] = defaults[corner as usize] + shared_offset;
	}
	set_top_and_bottom(&mut result);
	result
}

pub fn resolve_relative_cell_corner_coordinates_scaled(
	cell_size: Vector3,
	corner_offsets: &SharedCellVertexOffsets,
	location: Vector3i,
) -> HexCellCorners {
	let half_size = cell_size * 0.5;
	let mut result = [Vector3::zero(); HEX_CELL_CORNER_COUNT];
	for corner in 0..12u8 {
		result[corner as usize] =
			resolve_relative_corner(half_size, corner_offsets, location, corner);
	}
	set_top_and_bottom(&mut result);
	result
}

pub fn resolve_absolute_cell_corner_coordinates(
	cell_size: Vector3,
	half_size: Vector3,
	corner_offsets: &SharedCellVertexOffsets,
	location: Vector3i,
) -> HexCellCorners {
	let center = resolve_cell_coordinates_scaled(cell_size, location);
	let mut result = [Vector3::zero(); HEX_CELL_CORNER_COUNT];
	for corner in 0..12u8 {
		result[corner as usize] =
			center + resolve_relative_corner(half_size, corner_offsets, location, corner);
	}
	set_top_and_bottom(&mut result);
	result
}

pub fn apply_relative_cell_offsets(offsets: &HexCellCorners, corners: &mut HexCellCorners) {
	// Currently this only considers the Z-axis because this function uses a complex notion of relativity
	// (relativity *between multiple* points) and the Z-axis is the simplest axis for these cell shapes.
	for i in 0..HEX_CELL_CORNER_COUNT {
		let offset = offsets[i];
		if offset.z >= 0.0 {
			corners[i] = corners[i] * offset;
		} else {
			let other_z = if i <= 11 {
				(i + 6) % 12
			} else {
				25 - i
			};
			corners[i].z = corners[other_z].z * -offset.z;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty_offsets_resolve_default_prism() {
		let offsets = SharedCellVertexOffsets::new();
		let corners = resolve_relative_cell_corner_coordinates(&offsets, Vector3i::zero());
		assert_eq!(corners[12].z, -1.0);
		assert_eq!(corners[13].z, 1.0);
		for i in 0..6 {
			assert_eq!(corners[i].z, -1.0);
			assert_eq!(corners[i + 6].z, 1.0);
		}
	}

	#[test]
	fn absolute_corners_include_cell_center() {
		let offsets = SharedCellVertexOffsets::new();
		let cell_size = Vector3::new(2.0, 2.0, 2.0);
		let location = Vector3i::new(1, 0, 0);
		let corners = resolve_absolute_cell_corner_coordinates(
			cell_size,
			cell_size * 0.5,
			&offsets,
			location,
		);
		let center = resolve_cell_coordinates_scaled(cell_size, location);
		assert!((corners[12].x - center.x).abs() < 1e-5);
		assert!((corners[12].y - center.y).abs() < 1e-5);
	}
}
