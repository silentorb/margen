use margen_generation::biomes::Biome;
use margen_generation::core::sector::SectorId;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct NewCellContext {
	pub biome: Arc<Biome>,
	pub step: u32,
	pub sector: SectorId,
}

impl NewCellContext {
	pub fn new(biome: Arc<Biome>, step: u32, sector: SectorId) -> Self {
		Self {
			biome,
			step,
			sector,
		}
	}
}
