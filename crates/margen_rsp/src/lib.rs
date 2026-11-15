//! Relative Space Partitioning (RSP) core model.
//!
//! Directions are RH Z-up integer vectors. Generators may stay planar (`z == 0`)
//! while the type remains ready for Up/Down and arbitrary non-cardinal vectors.

use mythic_math::Vector3i;
use std::collections::BTreeSet;

/// Stable node identifier within an [`RspGraph`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RspNodeId(pub u32);

/// Relative direction as a 3D integer vector (RH Z-up).
pub type RspDirection = Vector3i;

/// +Y north (forward).
pub const NORTH: RspDirection = Vector3i { x: 0, y: 1, z: 0 };
/// -Y south.
pub const SOUTH: RspDirection = Vector3i { x: 0, y: -1, z: 0 };
/// +X east.
pub const EAST: RspDirection = Vector3i { x: 1, y: 0, z: 0 };
/// -X west.
pub const WEST: RspDirection = Vector3i { x: -1, y: 0, z: 0 };
/// +Z up (defined for forward compatibility; planar generators must not emit it yet).
pub const UP: RspDirection = Vector3i { x: 0, y: 0, z: 1 };
/// -Z down (defined for forward compatibility; planar generators must not emit it yet).
pub const DOWN: RspDirection = Vector3i { x: 0, y: 0, z: -1 };

/// Subject lies in `direction` relative to `reference`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RspConstraint {
	pub subject: RspNodeId,
	pub direction: RspDirection,
	pub reference: RspNodeId,
}

/// Relative constraint graph (no absolute placement).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RspGraph {
	pub nodes: BTreeSet<RspNodeId>,
	pub constraints: Vec<RspConstraint>,
}

/// Component-wise negate (works for Up/Down as well as planar cardinals).
pub fn opposite(direction: RspDirection) -> RspDirection {
	Vector3i {
		x: -direction.x,
		y: -direction.y,
		z: -direction.z,
	}
}

/// True when the direction has no vertical component.
pub fn is_planar(direction: RspDirection) -> bool {
	direction.z == 0
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opposite_swaps_north_south() {
		assert_eq!(opposite(NORTH), SOUTH);
		assert_eq!(opposite(SOUTH), NORTH);
		assert_eq!(opposite(EAST), WEST);
		assert_eq!(opposite(UP), DOWN);
	}

	#[test]
	fn planar_cardinals_have_zero_z() {
		assert!(is_planar(NORTH));
		assert!(is_planar(EAST));
		assert!(!is_planar(UP));
	}

	#[test]
	fn two_node_north_constraint_round_trip() {
		let a = RspNodeId(0);
		let b = RspNodeId(1);
		let graph = RspGraph {
			nodes: BTreeSet::from([a, b]),
			constraints: vec![RspConstraint {
				subject: a,
				direction: NORTH,
				reference: b,
			}],
		};
		assert_eq!(graph.nodes.len(), 2);
		assert_eq!(graph.constraints.len(), 1);
		assert_eq!(graph.constraints[0].direction, NORTH);
		assert_eq!(graph.constraints[0].subject, a);
		assert_eq!(graph.constraints[0].reference, b);
	}
}
