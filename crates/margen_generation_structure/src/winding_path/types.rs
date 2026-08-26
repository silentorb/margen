use margen_generation::core::world_cell::CellGrid;
use mythic_math::Vector3i;

pub type LocationPool = Vec<Vector3i>;

#[derive(Clone, Debug, Default)]
pub struct LocationPools {
	pub primary: LocationPool,
	pub fallback: LocationPool,
}

#[derive(Clone, Copy, Debug)]
pub struct SolvingState<'a> {
	pub grid: &'a CellGrid,
	pub headroom: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellPriority {
	Primary,
	Fallback,
	Blocked,
}
