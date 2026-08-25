use crate::analysis::types::PathDepths;
use crate::structure::querying::NeighborsMap;
use mythic::Vector3i;
use std::collections::VecDeque;

pub fn map_path_depths(
	neighbors_map: &NeighborsMap,
	start_location: Vector3i,
	depths: &mut PathDepths,
) {
	depths.clear();
	let mut queue = VecDeque::new();
	queue.push_back(start_location);
	depths.insert(start_location, 0);

	while let Some(location) = queue.pop_front() {
		let depth = depths[&location];
		let next_depth = depth + 1;
		if let Some(neighbors) = neighbors_map.get(&location) {
			for &next_location in neighbors {
				if !depths.contains_key(&next_location) {
					depths.insert(next_location, next_depth);
					queue.push_back(next_location);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[test]
	fn bfs_assigns_increasing_depth_from_start() {
		let a = Vector3i::new(0, 0, 0);
		let b = Vector3i::new(1, 0, 0);
		let c = Vector3i::new(2, 0, 0);

		let mut neighbors_map = HashMap::new();
		neighbors_map.insert(a, vec![b]);
		neighbors_map.insert(b, vec![a, c]);
		neighbors_map.insert(c, vec![b]);

		let mut depths = PathDepths::new();
		map_path_depths(&neighbors_map, a, &mut depths);

		assert_eq!(depths[&a], 0);
		assert_eq!(depths[&b], 1);
		assert_eq!(depths[&c], 2);
	}

	#[test]
	fn unreachable_cells_are_absent() {
		let a = Vector3i::new(0, 0, 0);
		let island = Vector3i::new(5, 5, 0);

		let mut neighbors_map = HashMap::new();
		neighbors_map.insert(a, vec![]);
		neighbors_map.insert(island, vec![]);

		let mut depths = PathDepths::new();
		map_path_depths(&neighbors_map, a, &mut depths);

		assert_eq!(depths.len(), 1);
		assert!(!depths.contains_key(&island));
	}
}
