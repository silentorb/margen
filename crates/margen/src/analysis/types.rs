use crate::core::{CellGrid, SectorData};
use mythic::Vector3i;
use std::collections::HashMap;

pub type PathDepths = HashMap<Vector3i, i32>;

#[derive(Clone, Debug)]
pub struct PartitionConfig {
	/// Mirrors Unreal `CELL_GROUP_SECOND_PASS` (default off).
	pub cell_group_second_pass: bool,
	/// Mirrors Unreal `CELL_GROUP_THIRD_PASS` (default off).
	pub cell_group_third_pass: bool,
}

impl Default for PartitionConfig {
	fn default() -> Self {
		Self {
			cell_group_second_pass: false,
			cell_group_third_pass: false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct AnalysisInput {
	pub grid: CellGrid,
	pub player_start: Vector3i,
	pub moveable_sector_height_offset: i32,
	pub config: PartitionConfig,
}

#[derive(Clone, Debug)]
pub struct AnalysisResult {
	pub sector_data: SectorData,
	pub path_depths: PathDepths,
	pub grid: CellGrid,
}

pub type StripMap = HashMap<Vector3i, usize>;
pub type Strips = Vec<Vec<Vector3i>>;
