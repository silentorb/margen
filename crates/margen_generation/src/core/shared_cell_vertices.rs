use crate::core::directions::DirectionIndex;
use mythic_math::{Vector3, Vector3i};
use std::collections::HashMap;

pub const COMPRESSED_CORNER_INDEX_SCALAR: i32 = 100_000;

/// Canonical shared corner key (location + corner index after positive-axis normalization).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedCellCorner {
	pub location: Vector3i,
	pub corner_index: DirectionIndex,
}

impl SharedCellCorner {
	/// Build a normalized corner key (Unreal `SharedCellCorner` constructor).
	pub fn new(new_location: Vector3i, new_corner_index: DirectionIndex) -> Self {
		if !(0..=11).contains(&new_corner_index) {
			return Self {
				location: Vector3i::zero(),
				corner_index: -1,
			};
		}

		let mut location = new_location;
		let mut corner_index = new_corner_index;

		// Only two directions/corners out of 12 are supported (positive axis).
		if !(7..=8).contains(&corner_index) {
			if corner_index < 6 {
				corner_index += 6;
				location = location + Vector3i::new(0, 0, -1);
			}

			match corner_index {
				6 => {
					location = location + Vector3i::new(0, -1, 0);
					corner_index = 8;
				}
				9 => {
					location = location + Vector3i::new(-1, 1, 0);
					corner_index = 7;
				}
				10 => {
					location = location + Vector3i::new(-1, 0, 0);
					corner_index = 8;
				}
				11 => {
					location = location + Vector3i::new(-1, 0, 0);
					corner_index = 7;
				}
				_ => {}
			}
		}

		Self {
			location,
			corner_index,
		}
	}

	pub fn from_compressed(compressed: Vector3i) -> Self {
		let corner_index = (compressed.z / COMPRESSED_CORNER_INDEX_SCALAR) as DirectionIndex;
		Self {
			location: Vector3i::new(
				compressed.x,
				compressed.y,
				compressed.z - corner_index as i32 * COMPRESSED_CORNER_INDEX_SCALAR,
			),
			corner_index,
		}
	}

	pub fn to_vector3i(self) -> Vector3i {
		Vector3i::new(
			self.location.x,
			self.location.y,
			self.location.z + self.corner_index as i32 * COMPRESSED_CORNER_INDEX_SCALAR,
		)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SharedVertexOffset {
	pub value: Vector3,
	/// When true, `value` is a grid offset to a neighboring vertex this vertex is wielded to.
	pub is_wielded: bool,
}

pub type SharedCellVertexOffsets = HashMap<Vector3i, SharedVertexOffset>;

pub fn get_shared_vertex_offset(
	offsets: &SharedCellVertexOffsets,
	key: SharedCellCorner,
	depth: i32,
) -> Vector3 {
	if let Some(entry) = offsets.get(&key.to_vector3i()) {
		if entry.is_wielded {
			let value = entry.value;
			let grid_offset = Vector3i::new(value.x as i32, value.y as i32, value.z as i32);
			if depth > 8 || grid_offset == Vector3i::zero() {
				Vector3::zero()
			} else {
				value * Vector3::new(1.0, 1.0, 2.0)
					+ get_shared_vertex_offset(
						offsets,
						SharedCellCorner::new(key.location + grid_offset, key.corner_index),
						depth + 1,
					)
			}
		} else {
			entry.value
		}
	} else {
		Vector3::zero()
	}
}

pub fn set_shared_vertex_offset(
	offsets: &mut SharedCellVertexOffsets,
	key: SharedCellCorner,
	offset: Vector3,
) {
	let compressed = key.to_vector3i();
	if let Some(entry) = offsets.get(&compressed) {
		if entry.is_wielded {
			return;
		}
	}
	offsets.insert(
		compressed,
		SharedVertexOffset {
			value: offset,
			is_wielded: false,
		},
	);
}

pub fn set_shared_vertex_offset_stack(
	offsets: &mut SharedCellVertexOffsets,
	location: Vector3i,
	direction: DirectionIndex,
	offset: Vector3,
) {
	set_shared_vertex_offset(offsets, SharedCellCorner::new(location, direction), offset);
	set_shared_vertex_offset(
		offsets,
		SharedCellCorner::new(location, direction + 6),
		offset,
	);
}

pub fn modify_shared_vertex_offset(
	offsets: &mut SharedCellVertexOffsets,
	key: SharedCellCorner,
	offset: Vector3,
) {
	let compressed = key.to_vector3i();
	if let Some(entry) = offsets.get(&compressed) {
		if !entry.is_wielded {
			offsets.insert(
				compressed,
				SharedVertexOffset {
					value: entry.value + offset,
					is_wielded: false,
				},
			);
		}
	} else {
		offsets.insert(
			compressed,
			SharedVertexOffset {
				value: offset,
				is_wielded: false,
			},
		);
	}
}

pub fn wield_shared_vertex_offsets(
	offsets: &mut SharedCellVertexOffsets,
	key: SharedCellCorner,
	offset: Vector3i,
) {
	offsets.insert(
		key.to_vector3i(),
		SharedVertexOffset {
			value: Vector3::from_i32(offset),
			is_wielded: true,
		},
	);
}

pub fn apply_corner_shifting_entry(
	target: &mut SharedCellVertexOffsets,
	key: SharedCellCorner,
	entry: SharedVertexOffset,
) {
	if entry.is_wielded {
		target.insert(key.to_vector3i(), entry);
	} else {
		modify_shared_vertex_offset(target, key, entry.value);
	}
}

pub fn apply_corner_shifting(
	target: &mut SharedCellVertexOffsets,
	source: &SharedCellVertexOffsets,
) {
	for (compressed, entry) in source {
		apply_corner_shifting_entry(target, SharedCellCorner::from_compressed(*compressed), *entry);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_get_round_trip() {
		let mut offsets = SharedCellVertexOffsets::new();
		let key = SharedCellCorner::new(Vector3i::zero(), 7);
		set_shared_vertex_offset(&mut offsets, key, Vector3::new(0.1, 0.2, 0.3));
		let got = get_shared_vertex_offset(&offsets, key, 0);
		assert_eq!(got, Vector3::new(0.1, 0.2, 0.3));
	}

	#[test]
	fn wield_follows_neighbor() {
		let mut offsets = SharedCellVertexOffsets::new();
		let target = SharedCellCorner::new(Vector3i::new(1, 0, 0), 7);
		set_shared_vertex_offset(&mut offsets, target, Vector3::new(0.5, 0.0, 0.0));
		let source = SharedCellCorner::new(Vector3i::zero(), 7);
		wield_shared_vertex_offsets(&mut offsets, source, Vector3i::new(1, 0, 0));
		let got = get_shared_vertex_offset(&offsets, source, 0);
		// wield value (1,0,0) * (1,1,2) + neighbor offset (0.5,0,0)
		assert_eq!(got, Vector3::new(1.5, 0.0, 0.0));
	}

	#[test]
	fn normalize_lower_corner_to_positive_axis() {
		let corner = SharedCellCorner::new(Vector3i::zero(), 0);
		assert_eq!(corner.corner_index, 8);
		assert_eq!(corner.location, Vector3i::new(0, -1, -1));
	}
}
