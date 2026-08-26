use margen_generation::{Biome, DirectionIndex, SectorId};
use mythic_math::{Vector2, Vector2i, Vector3, Vector3i};
use std::collections::HashMap;
use std::sync::Arc;

/// 3D cell location + direction (solid cell → open neighbor).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FaceLocation {
	pub location: Vector3i,
	pub direction: i32,
}

impl FaceLocation {
	pub fn new(location: Vector3i, direction: i32) -> Self {
		Self {
			location,
			direction,
		}
	}
}

#[derive(Clone, Debug)]
pub struct CellFace {
	pub sector: SectorId,
	pub biome: Option<Arc<Biome>>,
}

pub type CellFaceMap = HashMap<FaceLocation, CellFace>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FaceColumnLocation {
	pub location: Vector2i,
	pub direction: i32,
}

impl FaceColumnLocation {
	pub fn new(location: Vector2i, direction: i32) -> Self {
		Self {
			location,
			direction,
		}
	}
}

pub fn to_face_column_location(face_location: FaceLocation) -> FaceColumnLocation {
	FaceColumnLocation {
		location: Vector2i::new(face_location.location.x, face_location.location.y),
		direction: face_location.direction,
	}
}

pub fn to_face_location(column: FaceColumnLocation, z: i32) -> FaceLocation {
	FaceLocation {
		location: Vector3i::new(column.location.x, column.location.y, z),
		direction: column.direction,
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ColumnSpan {
	pub bottom: i32,
	pub top: i32,
}

pub type FaceColumnSpans = Vec<ColumnSpan>;
pub type FaceColumnSpansMap = HashMap<FaceColumnLocation, FaceColumnSpans>;

pub type SurfacePoints = Vec<Vector3>;
pub type SurfaceUvs = Vec<Vector2>;

#[derive(Clone, Debug, Default)]
pub struct FaceStructure {
	pub points: SurfacePoints,
	pub collapsed_point: i32,
}

pub type FaceStructureMap = HashMap<FaceLocation, FaceStructure>;

#[derive(Clone, Debug, Default)]
pub struct FaceAppearance {
	pub uvs: SurfaceUvs,
	pub material_id: i32,
}

impl FaceAppearance {
	pub fn new() -> Self {
		Self {
			uvs: Vec::new(),
			material_id: -1,
		}
	}
}

pub type FaceAppearanceMap = HashMap<FaceLocation, FaceAppearance>;

#[derive(Clone, Debug)]
pub struct RenderFace {
	pub points: SurfacePoints,
	pub uvs: SurfaceUvs,
	pub location: Vector3i,
	pub material_id: i32,
}

impl Default for RenderFace {
	fn default() -> Self {
		Self {
			points: Vec::new(),
			uvs: Vec::new(),
			location: Vector3i::zero(),
			material_id: -1,
		}
	}
}

pub type RenderFaces = Vec<RenderFace>;
pub type RenderFaceMap = HashMap<SectorId, RenderFaces>;

/// Face → selected material placeable id (topology metadata for later steps).
pub type FaceMetadataMap = HashMap<FaceLocation, margen_generation::MaterialPlaceableId>;

pub type DirectionIndexI32 = DirectionIndex;
