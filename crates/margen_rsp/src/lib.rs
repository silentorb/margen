//! Relative Space Partitioning (RSP) core model.
//!
//! Constraints are 2D angles in margen XY (RH Z-up planar): radians with
//! **0 = +X east**, **π/2 = +Y north**. Node `level` is remaining subdivisions
//! (bottom = 0).
//!
//! Alongside the relative graph, each layer carries an [`RspPartition`]: convex
//! cells (one per node, nested inside the parent's cell), node points strictly
//! inside their cells, and one doorway per constraint on the wall shared by its
//! endpoint cells. See `docs/rsp.md` for why that rules out overlaps.

use mythic_math::Vector2;
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::ops::{Add, Mul, Sub};

/// Stable node identifier within an [`RspGraph`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RspNodeId(pub u32);

/// Planar direction as radians in margen XY (0 = east / +X, π/2 = north / +Y).
pub type RspAngle = f32;

/// Node with subdivision level (remaining splits until 0).
///
/// `parent` is the immediate node this one replaced during subdivision (`None` for roots).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RspNode {
	pub id: RspNodeId,
	pub level: i32,
	pub parent: Option<RspNodeId>,
}

/// Subject lies at `angle` relative to `reference`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspConstraint {
	pub subject: RspNodeId,
	pub angle: RspAngle,
	pub reference: RspNodeId,
}

/// Relative constraint graph (no absolute placement).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspGraph {
	pub nodes: BTreeMap<RspNodeId, RspNode>,
	pub constraints: Vec<RspConstraint>,
}

/// Double-precision planar point in margen XY (partition geometry).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RspPoint {
	pub x: f64,
	pub y: f64,
}

impl RspPoint {
	pub const fn new(x: f64, y: f64) -> Self {
		Self { x, y }
	}
}

impl Add for RspPoint {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self::new(self.x + rhs.x, self.y + rhs.y)
	}
}

impl Sub for RspPoint {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self::new(self.x - rhs.x, self.y - rhs.y)
	}
}

impl Mul<f64> for RspPoint {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self {
		Self::new(self.x * rhs, self.y * rhs)
	}
}

/// Convex cell owned by one node: counter-clockwise vertices.
///
/// Collinear vertices are allowed and meaningful: every doorway touching the
/// cell is stored as one of its vertices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspCell {
	pub vertices: Vec<RspPoint>,
}

/// Geometric realization of one [`RspGraph`] layer.
///
/// - `cells[id]`: the node's convex cell; cells have disjoint interiors and tile the seed cell.
/// - `points[id]`: the node's position, strictly inside its cell.
/// - `doorways[i]`: point for `graph.constraints[i]`, a vertex of both endpoint cells.
///   An edge is drawn `subject point → doorway → reference point`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspPartition {
	pub cells: BTreeMap<RspNodeId, RspCell>,
	pub points: BTreeMap<RspNodeId, RspPoint>,
	pub doorways: Vec<RspPoint>,
}

/// One subdivision stage: the relative graph plus its partition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspLayer {
	pub graph: RspGraph,
	pub partition: RspPartition,
}

/// Subdivision snapshots: `[0]` is the pre-subdivision input, then one layer after each pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RspHistory {
	pub layers: Vec<RspLayer>,
}

impl RspHistory {
	/// Final (leaf) layer after all subdivision passes.
	pub fn final_layer(&self) -> &RspLayer {
		self.layers
			.last()
			.expect("RspHistory must contain at least one layer")
	}

	/// Final (leaf) graph after all subdivision passes.
	pub fn final_graph(&self) -> &RspGraph {
		&self.final_layer().graph
	}
}

/// 2D cross product `a × b` (z of the 3D cross).
pub fn cross_point(a: RspPoint, b: RspPoint) -> f64 {
	a.x * b.y - a.y * b.x
}

/// Dot product of two [`RspPoint`] vectors.
pub fn dot_point(a: RspPoint, b: RspPoint) -> f64 {
	a.x * b.x + a.y * b.y
}

/// Euclidean length of an [`RspPoint`] vector.
pub fn length_point(a: RspPoint) -> f64 {
	dot_point(a, a).sqrt()
}

/// Constraint angle from `reference` to `subject` (subject lies at this angle).
pub fn angle_between_points(reference: RspPoint, subject: RspPoint) -> RspAngle {
	let delta = subject - reference;
	delta.y.atan2(delta.x) as RspAngle
}

/// Opposite planar direction (`angle + π`).
pub fn opposite_angle(angle: RspAngle) -> RspAngle {
	angle + PI
}

/// Unit vector in margen XY for `angle` (0 = +X, π/2 = +Y).
pub fn angle_unit(angle: RspAngle) -> Vector2 {
	Vector2::new(angle.cos(), angle.sin())
}

/// Dot product of two planar vectors.
pub fn dot2(a: Vector2, b: Vector2) -> f32 {
	a.x * b.x + a.y * b.y
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::f32::consts::{FRAC_PI_2, PI};

	#[test]
	fn opposite_angle_adds_pi() {
		assert!((opposite_angle(0.0) - PI).abs() < 1e-5);
		assert!((opposite_angle(FRAC_PI_2) - (FRAC_PI_2 + PI)).abs() < 1e-5);
	}

	#[test]
	fn angle_unit_cardinal_axes() {
		let east = angle_unit(0.0);
		assert!((east.x - 1.0).abs() < 1e-5);
		assert!(east.y.abs() < 1e-5);
		let north = angle_unit(FRAC_PI_2);
		assert!(north.x.abs() < 1e-5);
		assert!((north.y - 1.0).abs() < 1e-5);
	}

	#[test]
	fn graph_level_and_angle_round_trip() {
		let a = RspNodeId(0);
		let b = RspNodeId(1);
		let graph = RspGraph {
			nodes: BTreeMap::from([
				(a, RspNode {
					id: a,
					level: 2,
					parent: None,
				}),
				(b, RspNode {
					id: b,
					level: 0,
					parent: None,
				}),
			]),
			constraints: vec![RspConstraint {
				subject: a,
				angle: FRAC_PI_2,
				reference: b,
			}],
		};
		assert_eq!(graph.nodes.len(), 2);
		assert_eq!(graph.nodes.get(&a).unwrap().level, 2);
		assert_eq!(graph.constraints.len(), 1);
		assert!((graph.constraints[0].angle - FRAC_PI_2).abs() < 1e-5);
		assert_eq!(graph.constraints[0].subject, a);
		assert_eq!(graph.constraints[0].reference, b);
	}
}
