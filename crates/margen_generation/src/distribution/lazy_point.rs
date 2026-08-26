use crate::core::directions::get_self_and_surrounding_offsets;
use crate::core::hex_shape::get_hex_distance_2d;
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::collections::HashMap;

/// Sector key matching Unreal `Vector3i(location.X, location.Y, 0) / sectorSize`.
pub fn sector_key(location: Vector3i, sector_size: i32) -> Vector3i {
	Vector3i::new(
		location.x / sector_size,
		location.y / sector_size,
		0,
	)
}

/// Create or return an existing lazy point at `location` (Unreal `getOrCreateLazyPoint`).
///
/// `new_point` receives `(sector_size, dice, location, next_id)`.
pub fn get_or_create_lazy_point<'a, P, F>(
	dice: &mut Dice,
	sector_size: i32,
	points: &'a mut HashMap<Vector3i, P>,
	location: Vector3i,
	new_point: F,
	next_id: &mut u32,
) -> &'a mut P
where
	F: FnOnce(i32, &mut Dice, Vector3i, u32) -> P,
{
	if !points.contains_key(&location) {
		let id = *next_id;
		*next_id += 1;
		let point = new_point(sector_size, dice, location, id);
		points.insert(location, point);
	}
	points.get_mut(&location).expect("just inserted or existing")
}

/// Sample nearest lazy point among self + surrounding sector neighbors (Unreal `sampleLazyPoints`).
pub fn sample_lazy_points<'a, P, F>(
	dice: &mut Dice,
	sector_size: i32,
	points: &'a mut HashMap<Vector3i, P>,
	location: Vector3i,
	nearest_distance: &mut i32,
	new_point: F,
	next_id: &mut u32,
	point_location: impl Fn(&P) -> Vector3i,
) -> Option<&'a P>
where
	F: Fn(i32, &mut Dice, Vector3i, u32) -> P,
{
	*nearest_distance = -1;
	let sector_location = sector_key(location, sector_size);

	let mut best_key: Option<Vector3i> = None;
	for offset in get_self_and_surrounding_offsets() {
		let neighbor_location = sector_location + offset;
		get_or_create_lazy_point(
			dice,
			sector_size,
			points,
			neighbor_location,
			|ss, d, loc, id| new_point(ss, d, loc, id),
			next_id,
		);
		let point = points.get(&neighbor_location).expect("just created");
		let distance = get_hex_distance_2d(location, point_location(point));
		if *nearest_distance == -1 || distance < *nearest_distance {
			*nearest_distance = distance;
			best_key = Some(neighbor_location);
		}
	}

	best_key.map(|key| points.get(&key).expect("best key present"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Clone, Debug, PartialEq, Eq)]
	struct TestPoint {
		location: Vector3i,
		id: u32,
	}

	#[test]
	fn lazy_create_reuses_same_sector_key() {
		let mut dice = Dice::new(1);
		let mut points = HashMap::new();
		let mut next_id = 1u32;
		let new_point = |sector_size: i32, _dice: &mut Dice, location: Vector3i, id: u32| TestPoint {
			location: location * sector_size,
			id,
		};

		let key = Vector3i::new(1, 0, 0);
		let first_id = {
			let p = get_or_create_lazy_point(
				&mut dice,
				2,
				&mut points,
				key,
				new_point,
				&mut next_id,
			);
			p.id
		};
		let second_id = {
			let p = get_or_create_lazy_point(
				&mut dice,
				2,
				&mut points,
				key,
				new_point,
				&mut next_id,
			);
			p.id
		};
		assert_eq!(first_id, second_id);
		assert_eq!(points.len(), 1);
		assert_eq!(next_id, 2);
	}

	#[test]
	fn sample_nearest_of_surrounding() {
		let mut dice = Dice::new(1);
		let mut points = HashMap::new();
		let mut next_id = 1u32;
		let mut nearest = -1;
		let result = sample_lazy_points(
			&mut dice,
			2,
			&mut points,
			Vector3i::zero(),
			&mut nearest,
			|sector_size, _dice, location, id| TestPoint {
				location: location * sector_size,
				id,
			},
			&mut next_id,
			|p| p.location,
		);
		assert!(result.is_some());
		assert_eq!(result.unwrap().location, Vector3i::zero());
		assert_eq!(nearest, 0);
		assert_eq!(points.len(), 7);
	}
}
