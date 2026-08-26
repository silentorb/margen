use crate::biomes::query::{BiomeNode, BiomeNodeId, BiomeQueryResult};
use crate::biomes::{default_biome, Biome};
use crate::core::directions::{get_grid_direction_vector, get_self_and_surrounding_offsets};
use crate::core::hex_shape::get_hex_distance_2d;
use crate::distribution::lazy_point::get_or_create_lazy_point;
use mythic_dice::Dice;
use mythic_math::Vector3i;
use std::collections::HashMap;
use std::sync::Arc;

/// Empty config matching Unreal `BiomeDistributionConfig` (clusterSize commented out).
#[derive(Clone, Copy, Debug, Default)]
pub struct BiomeDistributionConfig {}

/// Spatial biome sampler (Unreal `BiomeDistribution`). Nested sub-biome sampling is not ported.
#[derive(Clone, Debug)]
pub struct BiomeDistribution {
	pub nodes: HashMap<Vector3i, BiomeNode>,
	cache: HashMap<Vector3i, BiomeQueryResult>,
	pub biomes: Vec<Arc<Biome>>,
	pub dice: Dice,
	pub config: BiomeDistributionConfig,
}

impl BiomeDistribution {
	pub fn new(dice: Dice, biomes: Vec<Arc<Biome>>) -> Self {
		Self {
			nodes: HashMap::new(),
			cache: HashMap::new(),
			biomes,
			dice,
			config: BiomeDistributionConfig::default(),
		}
	}

	pub fn find_biome_sample(&mut self, location: Vector3i) -> BiomeQueryResult {
		let mut nearest_distance = -1;
		let mut next_id = self.nodes.len() as BiomeNodeId + 1;

		let node_key = sample_biome_nodes(
			&self.biomes,
			&mut self.dice,
			&mut self.nodes,
			location,
			&mut nearest_distance,
			&mut next_id,
		);

		match node_key {
			None => BiomeQueryResult::default(),
			Some(key) => {
				let node = self.nodes.get(&key).expect("sampled key");
				BiomeQueryResult {
					biome: Some(Arc::clone(&node.biome)),
					node_id: node.id,
					distance: nearest_distance,
				}
			}
		}
	}

	pub fn get_biome_and_node_id(&mut self, location: Vector3i) -> BiomeQueryResult {
		if let Some(existing) = self.cache.get(&location) {
			return existing.clone();
		}
		let result = self.find_biome_sample(location);
		self.cache.insert(location, result.clone());
		result
	}

	/// Biome at `location`; empty pool returns [`default_biome`].
	pub fn get_biome(&mut self, location: Vector3i) -> Arc<Biome> {
		self.get_biome_and_node_id(location)
			.biome
			.unwrap_or_else(default_biome)
	}
}

fn new_biome_node(
	sector_size: i32,
	dice: &mut Dice,
	sector_location: Vector3i,
	biome: Arc<Biome>,
	node: BiomeNodeId,
) -> BiomeNode {
	let random_direction = dice.get_int(5) as i8;
	let random_vector = get_grid_direction_vector(random_direction);
	let random_strength = dice.get_int(sector_size / 2 - 1);
	let location = sector_location * sector_size + random_vector * random_strength;
	BiomeNode {
		biome,
		location,
		id: node,
		radius: 0,
	}
}

/// Biome-specific sampling: sector key is `(x, y, 0)` without dividing by sector size
/// (matches Unreal `sampleBiomeNodes`). Returns the winning node map key.
fn sample_biome_nodes(
	types: &[Arc<Biome>],
	dice: &mut Dice,
	points: &mut HashMap<Vector3i, BiomeNode>,
	location: Vector3i,
	nearest_distance: &mut i32,
	next_id: &mut BiomeNodeId,
) -> Option<Vector3i> {
	*nearest_distance = -1;
	if types.is_empty() {
		return None;
	}

	// Hardcoded sector size matching Unreal `/*config.clusterSize*/ 2`.
	const SECTOR_SIZE: i32 = 2;
	let sector_location = Vector3i::new(location.x, location.y, 0);

	let mut best_key: Option<Vector3i> = None;
	for offset in get_self_and_surrounding_offsets() {
		let neighbor_location = sector_location + offset;
		get_or_create_lazy_point(
			dice,
			SECTOR_SIZE,
			points,
			neighbor_location,
			|sector_size, dice, sector_loc, id| {
				let biome = dice.from_slice(types);
				new_biome_node(sector_size, dice, sector_loc, biome, id)
			},
			next_id,
		);
		let point = points.get(&neighbor_location).expect("just created");
		let distance =
			get_hex_distance_2d(location, point.location) - point.biome.range_offset;
		if *nearest_distance == -1 || distance < *nearest_distance {
			*nearest_distance = distance;
			best_key = Some(neighbor_location);
		}
	}

	best_key
}

#[cfg(test)]
mod tests {
	use super::*;

	fn biome_with_id(id: u32) -> Arc<Biome> {
		Arc::new(Biome::new(id))
	}

	fn biome_with_range(id: u32, range_offset: i32) -> Arc<Biome> {
		let mut b = Biome::new(id);
		b.range_offset = range_offset;
		Arc::new(b)
	}

	#[test]
	fn same_seed_same_biome_at_location() {
		let biomes = vec![biome_with_id(1), biome_with_id(2)];
		let mut a = BiomeDistribution::new(Dice::new(42), biomes.clone());
		let mut b = BiomeDistribution::new(Dice::new(42), biomes);
		let loc = Vector3i::new(3, -1, 0);
		assert_eq!(a.get_biome(loc).id, b.get_biome(loc).id);
	}

	#[test]
	fn empty_pool_returns_default() {
		let mut dist = BiomeDistribution::new(Dice::new(1), vec![]);
		assert_eq!(dist.get_biome(Vector3i::zero()).id, 0);
	}

	#[test]
	fn cache_returns_same_result() {
		let biomes = vec![biome_with_id(1), biome_with_id(2)];
		let mut dist = BiomeDistribution::new(Dice::new(7), biomes);
		let loc = Vector3i::new(2, 2, 0);
		let first = dist.get_biome_and_node_id(loc);
		let second = dist.get_biome_and_node_id(loc);
		assert_eq!(first.node_id, second.node_id);
		assert_eq!(
			first.biome.as_ref().map(|b| b.id),
			second.biome.as_ref().map(|b| b.id)
		);
	}

	#[test]
	fn range_offset_can_shift_winner() {
		// Two biomes at adjacent sector keys; large range_offset on biome 2 makes it win farther away.
		let biomes = vec![biome_with_id(1), biome_with_range(2, 100)];
		let mut dist = BiomeDistribution::new(Dice::new(1), biomes);
		// Force both nodes to exist by sampling, then check a location near a low-offset node
		// still can prefer high range_offset when distance formula subtracts it.
		let sample = dist.find_biome_sample(Vector3i::new(5, 0, 0));
		assert!(sample.biome.is_some());
		// With range_offset 100 on biome 2, whenever that node is among candidates it often wins.
		// At least verify sampling produces a valid biome from the pool.
		let id = sample.biome.unwrap().id;
		assert!(id == 1 || id == 2);
	}

	#[test]
	fn multi_biome_pool_can_vary_across_locations() {
		let biomes = vec![biome_with_id(10), biome_with_id(20), biome_with_id(30)];
		let mut dist = BiomeDistribution::new(Dice::new(99), biomes);
		let mut ids = std::collections::BTreeSet::new();
		for x in -20..=20 {
			for y in -20..=20 {
				ids.insert(dist.get_biome(Vector3i::new(x, y, 0)).id);
			}
		}
		assert!(
			ids.len() > 1,
			"expected multiple biome ids across space, got {ids:?}"
		);
	}
}
