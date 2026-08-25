use mythic::{Vector2i, Vector3i};

fn signs_match(x: i32, y: i32) -> bool {
	let a_positive = x.wrapping_abs() == x;
	let b_positive = y.wrapping_abs() == y;
	a_positive == b_positive
}

pub fn get_hex_length(x: i32, y: i32) -> i32 {
	if signs_match(x, y) {
		(x + y).wrapping_abs()
	} else {
		x.wrapping_abs().max(y.wrapping_abs())
	}
}

pub fn get_hex_s(value: Vector2i) -> i32 {
	-value.x - value.y
}

pub fn get_hex_distance_2d_xy(first: Vector2i, second: Vector2i) -> i32 {
	get_hex_length(second.x - first.x, second.y - first.y)
}

pub fn get_hex_distance_2d(first: Vector3i, second: Vector3i) -> i32 {
	get_hex_length(second.x - first.x, second.y - first.y)
}

pub fn xy(v: Vector3i) -> Vector2i {
	Vector2i::new(v.x, v.y)
}

pub fn xyz(v: Vector2i, z: i32) -> Vector3i {
	Vector3i::new(v.x, v.y, z)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hex_s_and_distance() {
		assert_eq!(get_hex_s(Vector2i::new(1, 0)), -1);
		assert_eq!(get_hex_s(Vector2i::new(0, 0)), 0);
		assert_eq!(
			get_hex_distance_2d(Vector3i::zero(), Vector3i::new(1, 0, 0)),
			1
		);
		assert_eq!(
			get_hex_distance_2d(Vector3i::zero(), Vector3i::new(1, -1, 0)),
			1
		);
	}
}
