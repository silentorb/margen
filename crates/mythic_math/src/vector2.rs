use std::ops::{Add, Mul, Sub};

/// Integer 2D grid coordinate (axial hex / planar).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Vector2i {
	pub x: i32,
	pub y: i32,
}

impl Vector2i {
	pub const fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

	pub const fn zero() -> Self {
		Self { x: 0, y: 0 }
	}
}

impl Add for Vector2i {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Sub for Vector2i {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl Mul<i32> for Vector2i {
	type Output = Self;

	fn mul(self, rhs: i32) -> Self {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

/// Float 2D vector (UVs, planar offsets). Matches Unreal `FVector2f` / `Vector2f`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32,
}

impl Vector2 {
	pub const fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub const fn zero() -> Self {
		Self { x: 0.0, y: 0.0 }
	}
}

impl Add for Vector2 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Sub for Vector2 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl Mul<f32> for Vector2 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn integer_arithmetic() {
		let a = Vector2i::new(1, 2);
		let b = Vector2i::new(3, 4);
		assert_eq!(a + b, Vector2i::new(4, 6));
		assert_eq!(b - a, Vector2i::new(2, 2));
		assert_eq!(a * 2, Vector2i::new(2, 4));
	}

	#[test]
	fn float_arithmetic() {
		let a = Vector2::new(1.0, 2.0);
		let b = Vector2::new(0.5, 0.25);
		assert_eq!(a + b, Vector2::new(1.5, 2.25));
		assert_eq!(a - b, Vector2::new(0.5, 1.75));
		assert_eq!(a * 2.0, Vector2::new(2.0, 4.0));
	}
}
