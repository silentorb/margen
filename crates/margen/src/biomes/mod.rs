use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Tag string matching Unreal `Generation.Biome.Flat`.
pub const BIOME_TAG_FLAT: &str = "Generation.Biome.Flat";

/// Opaque placeable handle for material lists (adapter fills real assets later).
pub type MaterialPlaceableId = u32;

#[derive(Clone, Debug, Default)]
pub struct NestedBiome {
	pub inner_biome: Option<Arc<Biome>>,
	pub edge_biome: Option<Arc<Biome>>,
}

/// Engine-agnostic biome data (Unreal `UBiome` generation-relevant fields).
#[derive(Clone, Debug, Default)]
pub struct Biome {
	pub id: u32,
	pub materials: Vec<MaterialPlaceableId>,
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

/// Constant biome getter used by story composition when no distribution is available.
pub fn constant_get_biome(biome: Arc<Biome>) -> impl Fn(mythic::Vector3i) -> Arc<Biome> {
	move |_| Arc::clone(&biome)
}
