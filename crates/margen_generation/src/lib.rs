//! Core generation types: cells, biomes, distribution.

pub mod biomes;
pub mod core;
pub mod distribution;

pub use biomes::{
	biome_from_id, constant_get_biome, default_biome, surface_default_biome, Biome, BiomeDistribution,
	BiomeNodeId,
	BiomeQueryResult, MaterialPlaceable, MaterialPlaceableId, BIOME_TAG_FLAT,
};
pub use core::*;
pub use distribution::{get_or_create_lazy_point, sample_lazy_points};
