use mythic_math::Vector3i;

pub const ALL_DIRECTIONS_COUNT: i8 = 20;
pub const FLAT_DIRECTION_COUNT: i8 = 6;
pub const HORIZONTAL_DIRECTION_COUNT: i8 = 18;

pub type DirectionIndex = i8;

pub mod directions {
	use super::DirectionIndex;

	pub const NORTH: DirectionIndex = 0;
	pub const NORTHEAST: DirectionIndex = 1;
	pub const SOUTHEAST: DirectionIndex = 2;
	pub const SOUTH: DirectionIndex = 3;
	pub const SOUTHWEST: DirectionIndex = 4;
	pub const NORTHWEST: DirectionIndex = 5;

	pub const UP_NORTH: DirectionIndex = 6;
	pub const UP_NORTHEAST: DirectionIndex = 7;
	pub const UP_SOUTHEAST: DirectionIndex = 8;
	pub const UP_SOUTH: DirectionIndex = 9;
	pub const UP_SOUTHWEST: DirectionIndex = 10;
	pub const UP_NORTHWEST: DirectionIndex = 11;

	pub const DOWN_NORTH: DirectionIndex = 12;
	pub const DOWN_NORTHEAST: DirectionIndex = 13;
	pub const DOWN_SOUTHEAST: DirectionIndex = 14;
	pub const DOWN_SOUTH: DirectionIndex = 15;
	pub const DOWN_SOUTHWEST: DirectionIndex = 16;
	pub const DOWN_NORTHWEST: DirectionIndex = 17;

	pub const UP: DirectionIndex = 18;
	pub const DOWN: DirectionIndex = 19;
}

// Right-handed Z-up: +Y north, +X east. Flat hex neighbors (cube distance 1).
const GRID_DIRECTION_VECTORS: [Vector3i; ALL_DIRECTIONS_COUNT as usize] = [
	Vector3i::new(0, 1, 0),
	Vector3i::new(1, 0, 0),
	Vector3i::new(1, -1, 0),
	Vector3i::new(0, -1, 0),
	Vector3i::new(-1, 0, 0),
	Vector3i::new(-1, 1, 0),
	Vector3i::new(0, 1, 1),
	Vector3i::new(1, 0, 1),
	Vector3i::new(1, -1, 1),
	Vector3i::new(0, -1, 1),
	Vector3i::new(-1, 0, 1),
	Vector3i::new(-1, 1, 1),
	Vector3i::new(0, 1, -1),
	Vector3i::new(1, 0, -1),
	Vector3i::new(1, -1, -1),
	Vector3i::new(0, -1, -1),
	Vector3i::new(-1, 0, -1),
	Vector3i::new(-1, 1, -1),
	Vector3i::new(0, 0, 1),
	Vector3i::new(0, 0, -1),
];

pub fn get_direction_vectors() -> &'static [Vector3i; ALL_DIRECTIONS_COUNT as usize] {
	&GRID_DIRECTION_VECTORS
}

pub fn get_direction_index_from_vector(direction: Vector3i) -> DirectionIndex {
	for (i, d) in GRID_DIRECTION_VECTORS.iter().enumerate() {
		if *d == direction {
			return i as DirectionIndex;
		}
	}
	-1
}

pub fn get_grid_direction_vector(direction: DirectionIndex) -> Vector3i {
	GRID_DIRECTION_VECTORS[direction as u8 as usize]
}

pub fn rotate_direction_index(direction: DirectionIndex, turns: i32) -> DirectionIndex {
	if direction >= directions::UP {
		return direction;
	}
	let layer = direction / FLAT_DIRECTION_COUNT;
	(direction + turns as i8 + FLAT_DIRECTION_COUNT) % FLAT_DIRECTION_COUNT + layer * FLAT_DIRECTION_COUNT
}

pub fn rotate_direction_index_pitch(direction: DirectionIndex, height: i32) -> DirectionIndex {
	if height == 0 {
		return direction;
	}
	if direction < FLAT_DIRECTION_COUNT {
		return if height > 0 {
			direction + FLAT_DIRECTION_COUNT
		} else {
			direction + FLAT_DIRECTION_COUNT * 2
		};
	}
	if direction < FLAT_DIRECTION_COUNT * 2 {
		if height == -1 {
			return direction - FLAT_DIRECTION_COUNT;
		}
		if height < -1 {
			return direction + FLAT_DIRECTION_COUNT;
		}
		return direction;
	}
	if direction < FLAT_DIRECTION_COUNT * 3 {
		if height == 1 {
			return direction - FLAT_DIRECTION_COUNT * 2;
		}
		if height > 1 {
			return direction - FLAT_DIRECTION_COUNT;
		}
		return direction;
	}
	direction
}

pub fn rotate_direction_index_yaw_pitch(
	direction: DirectionIndex,
	yaw: i32,
	pitch: i32,
) -> DirectionIndex {
	rotate_direction_index_pitch(rotate_direction_index(direction, yaw), pitch)
}

pub fn get_opposite_direction_index(direction: DirectionIndex) -> DirectionIndex {
	match direction {
		directions::SOUTHEAST => directions::NORTHWEST,
		directions::SOUTH => directions::NORTH,
		directions::SOUTHWEST => directions::NORTHEAST,
		directions::NORTHWEST => directions::SOUTHEAST,
		directions::NORTH => directions::SOUTH,
		directions::NORTHEAST => directions::SOUTHWEST,
		directions::UP_SOUTHEAST => directions::DOWN_NORTHWEST,
		directions::UP_SOUTH => directions::DOWN_NORTH,
		directions::UP_SOUTHWEST => directions::DOWN_NORTHEAST,
		directions::UP_NORTHWEST => directions::DOWN_SOUTHEAST,
		directions::UP_NORTH => directions::DOWN_SOUTH,
		directions::UP_NORTHEAST => directions::DOWN_SOUTHWEST,
		directions::DOWN_SOUTHEAST => directions::UP_NORTHWEST,
		directions::DOWN_SOUTH => directions::UP_NORTH,
		directions::DOWN_SOUTHWEST => directions::UP_NORTHEAST,
		directions::DOWN_NORTHWEST => directions::UP_SOUTHEAST,
		directions::DOWN_NORTH => directions::UP_SOUTH,
		directions::DOWN_NORTHEAST => directions::UP_SOUTHWEST,
		directions::UP => directions::DOWN,
		directions::DOWN => directions::UP,
		_ => panic!("Can't get opposite of invalid direction"),
	}
}

pub fn is_upward_direction(index: DirectionIndex) -> bool {
	index == directions::UP
		|| (index >= directions::UP_NORTH && index <= directions::UP_NORTHWEST)
}

pub fn is_downward_direction(index: DirectionIndex) -> bool {
	index == directions::DOWN
		|| (index >= directions::DOWN_NORTH && index <= directions::DOWN_NORTHWEST)
}

pub fn is_horizontal_direction(index: DirectionIndex) -> bool {
	index != directions::UP && index != directions::DOWN
}

pub fn is_vertical_direction(index: DirectionIndex) -> bool {
	index == directions::UP || index == directions::DOWN
}

pub fn get_surrounding_offsets() -> [Vector3i; 6] {
	[
		Vector3i::new(0, 1, 0),
		Vector3i::new(1, 0, 0),
		Vector3i::new(1, -1, 0),
		Vector3i::new(0, -1, 0),
		Vector3i::new(-1, 0, 0),
		Vector3i::new(-1, 1, 0),
	]
}

pub fn get_self_and_surrounding_offsets() -> [Vector3i; 7] {
	[
		Vector3i::new(0, 0, 0),
		Vector3i::new(0, 1, 0),
		Vector3i::new(1, 0, 0),
		Vector3i::new(1, -1, 0),
		Vector3i::new(0, -1, 0),
		Vector3i::new(-1, 0, 0),
		Vector3i::new(-1, 1, 0),
	]
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn round_trip_known_vectors() {
		assert_eq!(
			get_direction_index_from_vector(Vector3i::new(0, 1, 0)),
			directions::NORTH
		);
		assert_eq!(
			get_direction_index_from_vector(Vector3i::new(1, 0, 0)),
			directions::NORTHEAST
		);
		assert_eq!(
			get_direction_index_from_vector(Vector3i::new(0, 0, 1)),
			directions::UP
		);
		assert_eq!(
			get_grid_direction_vector(directions::NORTH),
			Vector3i::new(0, 1, 0)
		);
		assert_eq!(
			get_opposite_direction_index(directions::NORTH),
			directions::SOUTH
		);
		assert_eq!(get_surrounding_offsets().len(), 6);
	}
}
