use crate::core::hex_shape::{get_hex_s, xy, xyz};
use mythic::{Vector2i, Vector3i};

pub fn scale_cell_location_2d(sector: Vector2i, radius: f32) -> Vector2i {
	let radius = radius as i32;
	let s = get_hex_s(sector);
	Vector2i::new(
		radius * sector.x - (radius - 1) * s,
		radius * sector.y - (radius - 1) * sector.x,
	)
}

pub fn scale_cell_location(sector: Vector3i, radius: f32) -> Vector3i {
	xyz(scale_cell_location_2d(xy(sector), radius), sector.z)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn scale_origin_stays_origin() {
		assert_eq!(
			scale_cell_location(Vector3i::zero(), 3.0),
			Vector3i::zero()
		);
	}

	#[test]
	fn scale_neighbor_matches_unreal() {
		// radius=3, sector (1,0,0): s=-1 → (3 - 2*(-1), 0 - 2*1, 0) = (5, -2, 0)
		assert_eq!(
			scale_cell_location(Vector3i::new(1, 0, 0), 3.0),
			Vector3i::new(5, -2, 0)
		);
	}
}
