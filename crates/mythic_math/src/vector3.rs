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

/// Float 3D vector. World axes for generation: see margen `docs/coordinates.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vector3 {
	pub const fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}

	pub const fn zero() -> Self {
		Self {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}
	}

	pub fn from_i32(v: Vector3i) -> Self {
		Self {
			x: v.x as f32,
			y: v.y as f32,
			z: v.z as f32,
		}
	}
}

impl Add for Vector3 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl Sub for Vector3 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl Mul<f32> for Vector3 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl Mul<Vector3> for Vector3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
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
	fn integer_arithmetic_and_hash() {
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

	#[test]
	fn float_arithmetic() {
		let a = Vector3::new(1.0, 2.0, 3.0);
		let b = Vector3::new(0.5, 0.5, 0.5);
		assert_eq!(a + b, Vector3::new(1.5, 2.5, 3.5));
		assert_eq!(a - b, Vector3::new(0.5, 1.5, 2.5));
		assert_eq!(a * 2.0, Vector3::new(2.0, 4.0, 6.0));
		assert_eq!(a * b, Vector3::new(0.5, 1.0, 1.5));
		assert_eq!(Vector3::from_i32(Vector3i::new(1, 2, 3)), a);
	}
}
