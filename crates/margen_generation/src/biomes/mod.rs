pub mod distribution;
pub mod query;

pub use distribution::{BiomeDistribution, BiomeDistributionConfig};
pub use query::{BiomeNode, BiomeNodeId, BiomeQueryResult};

use mythic_math::{Vector2, Vector2i};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Tag string matching Unreal `Generation.Biome.Flat`.
pub const BIOME_TAG_FLAT: &str = "Generation.Biome.Flat";

/// Opaque placeable handle for material lists (adapter fills real assets later).
pub type MaterialPlaceableId = u32;

/// Engine-agnostic material placeable (Unreal `UMaterialPlaceable` generation fields).
#[derive(Clone, Debug)]
pub struct MaterialPlaceable {
	pub id: MaterialPlaceableId,
	pub face_span: Vector2,
	pub requirements: BTreeSet<String>,
	pub prevented_tags: BTreeSet<String>,
	pub minimum_length: Vector2i,
}

impl Default for MaterialPlaceable {
	fn default() -> Self {
		Self {
			id: 0,
			face_span: Vector2::new(1.0, 1.0),
			requirements: BTreeSet::new(),
			prevented_tags: BTreeSet::new(),
			minimum_length: Vector2i::new(1, 1),
		}
	}
}

impl MaterialPlaceable {
	pub fn new(id: MaterialPlaceableId) -> Self {
		Self {
			id,
			..Default::default()
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct NestedBiome {
	pub inner_biome: Option<Arc<Biome>>,
	pub edge_biome: Option<Arc<Biome>>,
}

/// Engine-agnostic biome data (Unreal `UBiome` generation-relevant fields).
#[derive(Clone, Debug, Default)]
pub struct Biome {
	pub id: u32,
	pub materials: Vec<MaterialPlaceable>,
	pub tags: BTreeSet<String>,
	pub population_tags: BTreeMap<String, i32>,
	pub nested_biomes: Vec<NestedBiome>,
	pub debug_color: [u8; 4],
	pub range_offset: i32,
}

impl Biome {
	pub fn new(id: u32) -> Self {
		Self {
			id,
			..Default::default()
		}
	}

	pub fn has_tag(&self, tag: &str) -> bool {
		self.tags.contains(tag)
	}

	pub fn is_flat(&self) -> bool {
		self.has_tag(BIOME_TAG_FLAT)
	}
}

pub fn default_biome() -> Arc<Biome> {
	Arc::new(Biome::new(0))
}

/// Biome with wall/floor/ceiling material placeables for Surfacing smoke tests and FFI hosts.
pub fn surface_default_biome() -> Arc<Biome> {
	let mut wall = MaterialPlaceable::new(1);
	wall.requirements.insert("Face.Wall".to_string());
	let mut floor = MaterialPlaceable::new(2);
	floor.requirements.insert("Face.Floor".to_string());
	let mut ceiling = MaterialPlaceable::new(3);
	ceiling.requirements.insert("Face.Ceiling".to_string());
	let mut biome = Biome::new(0);
	biome.materials = vec![wall, floor, ceiling];
	biome.debug_color = [128, 128, 128, 255];
	Arc::new(biome)
}

/// Constant biome getter used by story composition when no distribution is available.
pub fn constant_get_biome(biome: Arc<Biome>) -> impl Fn(mythic_math::Vector3i) -> Arc<Biome> {
	move |_| Arc::clone(&biome)
}

/// Build a biome from a numeric id (hosts / FFI). Ids 0..=2 get distinct debug colors.
pub fn biome_from_id(id: u32) -> Arc<Biome> {
	let mut biome = Biome::new(id);
	biome.debug_color = match id {
		0 => [128, 128, 128, 255],
		1 => [80, 160, 80, 255],
		2 => [80, 80, 180, 255],
		_ => [id as u8, id as u8, id as u8, 255],
	};
	Arc::new(biome)
}
