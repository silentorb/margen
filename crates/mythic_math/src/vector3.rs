use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Sub};

/// Integer grid / voxel coordinate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Vector3i {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl Vector3i {
	pub const fn new(x: i32, y: i32, z: i32) -> Self {
		Self { x, y, z }
	}

	pub const fn zero() -> Self {
		Self { x: 0, y: 0, z: 0 }
	}

	pub const fn none() -> Self {
		Self {
			x: i32::MIN,
			y: i32::MIN,
			z: i32::MIN,
		}
	}

	pub fn is_none(self) -> bool {
		self == Self::none()
	}
}

impl Add for Vector3i {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl Sub for Vector3i {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl Mul<i32> for Vector3i {
	type Output = Self;

	fn mul(self, rhs: i32) -> Self {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl Mul<Vector3i> for i32 {
	type Output = Vector3i;

	fn mul(self, rhs: Vector3i) -> Vector3i {
		rhs * self
	}
}

impl Hash for Vector3i {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let value = (self.x as u32) ^ ((self.y as u32) << 10) ^ ((self.z as u32) << 20);
		state.write_usize(value as usize);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::hash_map::DefaultHasher;

	fn hash_of(v: Vector3i) -> u64 {
		let mut hasher = DefaultHasher::new();
		v.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn arithmetic_and_hash() {
		let a = Vector3i::new(1, 2, 3);
		let b = Vector3i::new(4, 5, 6);
		assert_eq!(a + b, Vector3i::new(5, 7, 9));
		assert_eq!(b - a, Vector3i::new(3, 3, 3));
		assert_eq!(a * 2, Vector3i::new(2, 4, 6));
		assert_eq!(Vector3i::zero(), Vector3i::new(0, 0, 0));
		assert_ne!(Vector3i::none(), Vector3i::zero());
		assert_eq!(hash_of(a), hash_of(a));
		assert_ne!(hash_of(a), hash_of(b));
	}
}
