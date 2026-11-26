//! RSP skinning: turn a laid-out RSP layer into floor slabs.
//!
//! Each node becomes a low-poly cylindrical room and each constraint a pathway of fixed
//! width. The flow has two stages:
//!
//! 1. [`fit_floor_plan`] scales the partition and fits room and pathway footprints into it
//!    with margins. Every footprint lies inside its node's cell and pathways cross walls
//!    only through dedicated door slots, so the footprints have disjoint interiors and touch
//!    only along declared joins (proof in `docs/rsp-skinning.md`).
//! 2. [`skin_floor_plan`] hands every footprint to the skinner chosen by
//!    [`assign_skinners`]. Today that is always a flat slab; region- or biome-specific
//!    skinners plug in there.

mod extrude;
mod fit;
mod validate;
mod walls;

pub use extrude::{assign_skinners, extrude_piece, skin_floor_plan, RspPieceSkinner};
pub use fit::{center_rooms, fit_floor_plan, fit_floor_plan_at_scale, inscribed_circle};
pub use validate::{validate_floor_plan, RspFloorPlanError};

use margen_rsp::{RspLayer, RspNodeId, RspPoint};
use mythic_math::Vector3;

/// Skinning parameters in layout units (see `RspLayoutConfig::preferred_edge_length`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspSkinConfig {
	/// Walkable width of every pathway (and of the room side it attaches to).
	pub pathway_width: f32,
	/// Clearance kept between a room and its cell wall, and around door slots.
	pub gap: f32,
	/// Smallest room radius; rooms grow above this only where pathways crowd them.
	pub room_min_radius: f32,
	/// Room radius used when the cell has space for it.
	pub room_preferred_radius: f32,
	/// Depth of every floor slab.
	pub floor_thickness: f32,
	/// Arc resolution: room outlines get a vertex at least every `2π / target_sides`.
	pub target_sides: u32,
	/// Smallest angle kept between the wedges of two pathways leaving the same room (radians).
	pub angular_gap: f32,
}

impl Default for RspSkinConfig {
	fn default() -> Self {
		let pathway_width = 24.0;
		Self {
			pathway_width,
			gap: pathway_width * 0.1,
			room_min_radius: pathway_width * 0.75,
			room_preferred_radius: pathway_width * 1.25,
			floor_thickness: pathway_width * 0.25,
			target_sides: 6,
			angular_gap: 2.0_f32.to_radians(),
		}
	}
}

/// Why skinning failed.
#[derive(Clone, Debug, PartialEq)]
pub enum RspSkinError {
	/// A graph node has no cell or point.
	MissingGeometry(RspNodeId),
	/// The endpoint cells of a constraint touch only at a point, so there is no wall to
	/// cut a door into.
	NoSharedWall { constraint: usize },
	/// No scale up to [`MAX_SKIN_SCALE`] satisfied every margin; `blocker` is the last
	/// requirement that failed.
	ScaleLimit { blocker: RspSkinBlocker },
}

/// A margin requirement that fails at a particular scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RspSkinBlocker {
	/// The constraint's door slot is shorter than `pathway_width + 2·gap`.
	SlotTooShort { constraint: usize },
	/// Two doors seen from the node are closer than `angular_gap`.
	DoorsTooClose { node: RspNodeId },
	/// The room the node needs does not fit inside its cell with `gap` to spare.
	RoomTooLarge { node: RspNodeId },
	/// The door is not at least `gap` beyond the room side facing it.
	DoorBehindFace { node: RspNodeId, constraint: usize },
	/// The pathway wedge from the room side to the door is π or wider.
	WedgeTooWide { node: RspNodeId, constraint: usize },
}

/// Largest uniform scale [`fit_floor_plan`] tries before giving up.
pub const MAX_SKIN_SCALE: f64 = (1u64 << 20) as f64;

/// What a floor piece represents.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RspPieceKind {
	Room(RspNodeId),
	/// The half of `constraint`'s pathway inside `node`'s cell (room side to door).
	Pathway { constraint: usize, node: RspNodeId },
}

/// Convex counter-clockwise footprint at floor level.
#[derive(Clone, Debug, PartialEq)]
pub struct RspFloorPiece {
	pub kind: RspPieceKind,
	pub polygon: Vec<RspPoint>,
	/// Indices `i` of polygon edges `polygon[i] → polygon[i + 1]` that join another piece
	/// (a room side or a door). Joined edges share bit-identical endpoints.
	pub joins: Vec<usize>,
}

/// Room circle (all room vertices lie on it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspRoom {
	pub node: RspNodeId,
	pub center: RspPoint,
	pub radius: f64,
}

/// Footprints fitted into a scaled partition.
///
/// `layer` is the input layer scaled by `scale`, with each doorway moved to the centre of
/// its door; its partition invariants still hold.
#[derive(Clone, Debug, PartialEq)]
pub struct RspFloorPlan {
	pub layer: RspLayer,
	pub scale: f64,
	pub rooms: Vec<RspRoom>,
	pub pieces: Vec<RspFloorPiece>,
}

/// Triangle range of one floor piece in an [`RspMesh`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspMeshPiece {
	pub kind: RspPieceKind,
	/// First index into [`RspMesh::indices`] (a multiple of 3).
	pub index_begin: u32,
	pub index_count: u32,
}

/// Triangle mesh in margen XYZ (Z up). Flat shaded: vertices are not shared between faces.
/// Triangles are counter-clockwise seen from outside.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspMesh {
	pub positions: Vec<Vector3>,
	pub normals: Vec<Vector3>,
	pub indices: Vec<u32>,
	pub pieces: Vec<RspMeshPiece>,
}

/// Fit footprints and build slabs for a laid-out layer.
pub fn skin_rsp_layer(
	layer: &RspLayer,
	config: &RspSkinConfig,
) -> Result<(RspFloorPlan, RspMesh), RspSkinError> {
	let config = sanitize_config(config);
	let plan = fit_floor_plan(layer, &config)?;
	let mesh = skin_floor_plan(&plan, &assign_skinners(&plan, &config));
	Ok((plan, mesh))
}

/// Replace unusable values with defaults and keep the radii consistent with the width.
pub fn sanitize_config(config: &RspSkinConfig) -> RspSkinConfig {
	let defaults = RspSkinConfig::default();
	let positive = |value: f32, fallback: f32| {
		if value.is_finite() && value > 0.0 {
			value
		} else {
			fallback
		}
	};
	let pathway_width = positive(config.pathway_width, defaults.pathway_width);
	let gap = positive(config.gap, pathway_width * 0.1);
	// A room side of width `w` needs a circle of radius above `w / 2`.
	let room_min_radius =
		positive(config.room_min_radius, pathway_width * 0.75).max(pathway_width * 0.51);
	let room_preferred_radius =
		positive(config.room_preferred_radius, pathway_width * 1.25).max(room_min_radius);
	RspSkinConfig {
		pathway_width,
		gap,
		room_min_radius,
		room_preferred_radius,
		floor_thickness: positive(config.floor_thickness, pathway_width * 0.25),
		target_sides: config.target_sides.max(3),
		angular_gap: positive(config.angular_gap, defaults.angular_gap).min(0.5),
	}
}

#[cfg(test)]
mod tests;
