use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::{Mutex, OnceLock};

/// Seeded RNG used throughout generation pipelines.
#[derive(Clone, Debug)]
pub struct Dice {
	initial_seed: i32,
	stream: StdRng,
}

impl Dice {
	pub fn new(seed: i32) -> Self {
		Self {
			initial_seed: seed,
			stream: StdRng::seed_from_u64(seed as u64),
		}
	}

	pub fn initial_seed(&self) -> i32 {
		self.initial_seed
	}

	/// Inclusive `[0, max]`.
	pub fn get_int(&mut self, max: i32) -> i32 {
		self.get_int_range(0, max)
	}

	/// Inclusive `[min, max]`.
	pub fn get_int_range(&mut self, min: i32, max: i32) -> i32 {
		self.stream.gen_range(min..=max)
	}

	pub fn get_float(&mut self, max: f32) -> f32 {
		self.get_float_range(0.0, max)
	}

	pub fn get_float_range(&mut self, min: f32, max: f32) -> f32 {
		self.stream.gen_range(min..=max)
	}

	pub fn get_ranged_float(&mut self, max: f32) -> f32 {
		self.get_float_range(-max, max)
	}

	pub fn get_boolean(&mut self) -> bool {
		self.get_int(1) == 0
	}

	pub fn from_slice<T: Clone>(&mut self, container: &[T]) -> T {
		assert!(!container.is_empty());
		if container.len() == 1 {
			return container[0].clone();
		}
		let index = self.get_int_range(0, (container.len() - 1) as i32) as usize;
		container[index].clone()
	}

	pub fn remove_from_vec<T>(&mut self, container: &mut Vec<T>) -> T {
		assert!(!container.is_empty());
		if container.len() == 1 {
			return container.remove(0);
		}
		let index = self.get_int_range(0, (container.len() - 1) as i32) as usize;
		container.remove(index)
	}
}

impl Default for Dice {
	fn default() -> Self {
		Self::new(0)
	}
}

pub fn get_static_dice() -> &'static Mutex<Dice> {
	static DICE: OnceLock<Mutex<Dice>> = OnceLock::new();
	DICE.get_or_init(|| Mutex::new(Dice::new(0)))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deterministic_for_same_seed() {
		let mut a = Dice::new(42);
		let mut b = Dice::new(42);
		assert_eq!(a.initial_seed(), 42);
		assert_eq!(a.get_int(100), b.get_int(100));
		assert_eq!(a.get_float(1.0), b.get_float(1.0));
		assert_eq!(a.get_boolean(), b.get_boolean());
	}

	#[test]
	fn remove_from_vec_clears_size_one() {
		let mut dice = Dice::new(1);
		let mut values = vec![7];
		let removed = dice.remove_from_vec(&mut values);
		assert_eq!(removed, 7);
		assert!(values.is_empty());
	}

	#[test]
	fn remove_from_vec_shrinks_larger() {
		let mut dice = Dice::new(2);
		let mut values = vec![1, 2, 3];
		let before = values.len();
		let _ = dice.remove_from_vec(&mut values);
		assert_eq!(values.len(), before - 1);
	}
}
