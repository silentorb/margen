//! Relative Space Partitioning (RSP) core model.
//!
//! Constraints are 2D angles in margen XY (RH Z-up planar): radians with
//! **0 = +X east**, **π/2 = +Y north**. Node `level` is remaining subdivisions
//! (bottom = 0).

use mythic_math::Vector2;
use std::collections::BTreeMap;
use std::f32::consts::PI;

/// Stable node identifier within an [`RspGraph`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RspNodeId(pub u32);

/// Planar direction as radians in margen XY (0 = east / +X, π/2 = north / +Y).
pub type RspAngle = f32;

/// Node with subdivision level (remaining splits until 0).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RspNode {
	pub id: RspNodeId,
	pub level: i32,
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
				(a, RspNode { id: a, level: 2 }),
				(b, RspNode { id: b, level: 0 }),
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
