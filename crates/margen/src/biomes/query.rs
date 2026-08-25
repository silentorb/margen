use crate::biomes::Biome;
use std::sync::Arc;

/// Biome sample node id (Unreal `NodeId` for biome distribution; distinct from `DagNodeId`).
pub type BiomeNodeId = u32;

#[derive(Clone, Debug, Default)]
pub struct BiomeQueryResult {
	pub biome: Option<Arc<Biome>>,
	pub node_id: BiomeNodeId,
	pub distance: i32,
}

#[derive(Clone, Debug)]
pub struct BiomeNode {
	pub biome: Arc<Biome>,
	pub location: mythic::Vector3i,
	pub id: BiomeNodeId,
	pub radius: i32,
}
