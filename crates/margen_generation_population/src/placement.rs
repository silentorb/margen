use crate::PlaceableId;
use margen_generation::SectorId;
use mythic_math::Vector3i;

/// Abstract resolved placement (Unreal `ActorPlacement`) before host instantiation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorPlacement {
	pub locations: Vec<Vector3i>,
	pub placeable_id: PlaceableId,
	pub biome_id: u32,
	pub sector: SectorId,
}

impl ActorPlacement {
	pub fn new(location: Vector3i, placeable_id: PlaceableId) -> Self {
		Self {
			locations: vec![location],
			placeable_id,
			biome_id: 0,
			sector: 0,
		}
	}
}

pub type ActorPlacements = Vec<ActorPlacement>;
