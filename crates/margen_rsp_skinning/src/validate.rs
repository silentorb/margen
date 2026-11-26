//! Independent check of a floor plan's conclusion: footprints are convex, have disjoint
//! interiors, and touch only along declared joins.
//!
//! [`fit_floor_plan`](crate::fit_floor_plan) guarantees this by construction; this module
//! re-checks it geometrically (separating-axis tests) so regressions are caught.

use std::collections::BTreeMap;

use margen_rsp::{cross_point, length_point, RspPoint};

use crate::RspFloorPlan;

/// First problem found by [`validate_floor_plan`].
#[derive(Clone, Debug, PartialEq)]
pub enum RspFloorPlanError {
	/// Piece polygon is not convex counter-clockwise with positive area.
	InvalidPiece(usize),
	/// A join edge has no partner edge (same endpoints, opposite direction) on another piece.
	UnmatchedJoin { piece: usize, edge: usize },
	/// Two pieces that share no join are not strictly separated.
	Overlap(usize, usize),
}

const RELATIVE_EPS: f64 = 1e-9;

pub fn validate_floor_plan(plan: &RspFloorPlan) -> Result<(), RspFloorPlanError> {
	let pieces = &plan.pieces;
	let eps = tolerance(plan);
	for (index, piece) in pieces.iter().enumerate() {
		if !is_convex(&piece.polygon, eps) {
			return Err(RspFloorPlanError::InvalidPiece(index));
		}
	}

	// Partner of every join edge: another piece with the reversed edge.
	let key = |p: RspPoint| (p.x.to_bits(), p.y.to_bits());
	let mut join_owner: BTreeMap<_, usize> = BTreeMap::new();
	for (index, piece) in pieces.iter().enumerate() {
		for edge in &piece.joins {
			let (a, b) = edge_points(&piece.polygon, *edge);
			join_owner.insert((key(a), key(b)), index);
		}
	}
	let mut joined: Vec<Vec<usize>> = vec![Vec::new(); pieces.len()];
	for (index, piece) in pieces.iter().enumerate() {
		for edge in &piece.joins {
			let (a, b) = edge_points(&piece.polygon, *edge);
			let partner = join_owner.get(&(key(b), key(a))).filter(|other| **other != index);
			match partner {
				Some(other) => joined[index].push(*other),
				None => {
					return Err(RspFloorPlanError::UnmatchedJoin {
						piece: index,
						edge: *edge,
					})
				}
			}
		}
	}

	let boxes: Vec<(RspPoint, RspPoint)> = pieces.iter().map(|p| bounds(&p.polygon)).collect();
	for i in 0..pieces.len() {
		for j in (i + 1)..pieces.len() {
			let (a_min, a_max) = boxes[i];
			let (b_min, b_max) = boxes[j];
			if a_max.x + eps < b_min.x
				|| b_max.x + eps < a_min.x
				|| a_max.y + eps < b_min.y
				|| b_max.y + eps < a_min.y
			{
				continue;
			}
			let a = &pieces[i].polygon;
			let b = &pieces[j].polygon;
			// Joined pieces are convex, counter-clockwise and share an edge in opposite
			// directions, so each lies on its own side of that edge line: interiors disjoint.
			if !joined[i].contains(&j) && !separated(a, b, eps) {
				return Err(RspFloorPlanError::Overlap(i, j));
			}
		}
	}
	Ok(())
}

fn edge_points(polygon: &[RspPoint], edge: usize) -> (RspPoint, RspPoint) {
	(polygon[edge], polygon[(edge + 1) % polygon.len()])
}

/// Separating-axis test for convex counter-clockwise polygons: some edge line of one has
/// the other entirely at least `margin` outside it (a negative margin allows touching).
fn separated(a: &[RspPoint], b: &[RspPoint], margin: f64) -> bool {
	let by_edges_of = |owner: &[RspPoint], other: &[RspPoint]| {
		let n = owner.len();
		(0..n).any(|i| {
			let p = owner[i];
			let q = owner[(i + 1) % n];
			let len = length_point(q - p);
			len > 0.0 && other.iter().all(|v| cross_point(q - p, *v - p) / len <= -margin)
		})
	};
	by_edges_of(a, b) || by_edges_of(b, a)
}

fn is_convex(polygon: &[RspPoint], eps: f64) -> bool {
	let n = polygon.len();
	if n < 3 {
		return false;
	}
	let mut twice_area = 0.0;
	for i in 0..n {
		let a = polygon[i];
		let b = polygon[(i + 1) % n];
		twice_area += cross_point(a, b);
		let len = length_point(b - a);
		if len == 0.0 {
			return false;
		}
		if polygon.iter().any(|v| cross_point(b - a, *v - a) / len < -eps) {
			return false;
		}
	}
	twice_area > 0.0
}

fn bounds(polygon: &[RspPoint]) -> (RspPoint, RspPoint) {
	polygon.iter().fold(
		(
			RspPoint::new(f64::INFINITY, f64::INFINITY),
			RspPoint::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
		),
		|(min, max), v| {
			(
				RspPoint::new(min.x.min(v.x), min.y.min(v.y)),
				RspPoint::new(max.x.max(v.x), max.y.max(v.y)),
			)
		},
	)
}

fn tolerance(plan: &RspFloorPlan) -> f64 {
	let (min, max) = plan
		.pieces
		.iter()
		.map(|p| bounds(&p.polygon))
		.fold(
			(
				RspPoint::new(f64::INFINITY, f64::INFINITY),
				RspPoint::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
			),
			|(min, max), (a, b)| {
				(
					RspPoint::new(min.x.min(a.x), min.y.min(a.y)),
					RspPoint::new(max.x.max(b.x), max.y.max(b.y)),
				)
			},
		);
	if !min.x.is_finite() {
		return RELATIVE_EPS;
	}
	RELATIVE_EPS * length_point(max - min).max(1.0)
}
