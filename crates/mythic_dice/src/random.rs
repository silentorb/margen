/// Deterministic pseudo-random index in `[0, range)`.
pub fn psuedorandom(seed: i32, range: i32) -> i32 {
	seed.wrapping_abs() % range
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wraps_negative_seed() {
		assert_eq!(psuedorandom(-7, 5), 2);
		assert_eq!(psuedorandom(7, 5), 2);
	}
}
