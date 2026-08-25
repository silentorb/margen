use crate::Dice;

pub fn shuffle_vec<T: Clone>(dice: &mut Dice, input: &[T]) -> Vec<T> {
	let mut temp = input.to_vec();
	let mut output = Vec::with_capacity(input.len());
	while !temp.is_empty() {
		let index = dice.get_int_range(0, (temp.len() - 1) as i32) as usize;
		output.push(temp.remove(index));
	}
	output
}

/// Randomly removes entries until `buffer.len() == count`, preserving order of survivors.
pub fn select_many_ordered<T>(dice: &mut Dice, count: usize, buffer: &mut Vec<T>) {
	while buffer.len() > count {
		let index = dice.get_int_range(0, (buffer.len() - 1) as i32) as usize;
		buffer.remove(index);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn shuffle_vec_preserves_elements() {
		let mut dice = Dice::new(7);
		let input = vec![1, 2, 3, 4];
		let shuffled = shuffle_vec(&mut dice, &input);
		let mut sorted = shuffled.clone();
		sorted.sort();
		assert_eq!(sorted, input);
	}

	#[test]
	fn select_many_ordered_reduces_to_count() {
		let mut dice = Dice::new(11);
		let mut buffer = vec![1, 2, 3, 4, 5];
		select_many_ordered(&mut dice, 2, &mut buffer);
		assert_eq!(buffer.len(), 2);
	}
}
