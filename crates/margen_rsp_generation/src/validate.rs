//! Independent checker for RSP partition invariants and edge conflicts.
//!
//! The generator guarantees these by construction (see `docs/rsp.md`). This module
//! re-checks them geometrically so tests and hosts catch any regression. All checks
//! use a clearance tolerance scaled to the partition size, so near-misses fail too.

use margen_rsp::{cross_point, dot_point, length_point, RspCell, RspLayer, RspNodeId, RspPoint};

use crate::partition::polygon_area;

/// First invariant violation found by [`validate_rsp_embedding`].
#[derive(Clone, Debug, PartialEq)]
pub enum RspEmbeddingError {
	/// A graph node has no cell or no point in the partition.
	MissingGeometry(RspNodeId),
	/// `partition.doorways` is not parallel to `graph.constraints`.
	DoorwayCountMismatch { doorways: usize, constraints: usize },
	/// Cell is not convex counter-clockwise with positive area.
	InvalidCell(RspNodeId),
	/// Two cells' interiors overlap.
	OverlappingCells(RspNodeId, RspNodeId),
	/// Cell areas do not add up to the seed square.
	IncompleteCover { covered: f64, expected: f64 },
	/// Node point is not strictly inside its cell.
	PointNotInterior(RspNodeId),
	/// Doorway of `constraint` is not on the boundary of `node`'s cell.
	DoorwayOffWall { constraint: usize, node: RspNodeId },
	/// Two constraints share a doorway.
	SharedDoorway(usize, usize),
	/// Two edge drawings touch somewhere other than a shared node.
	EdgeConflict(usize, usize),
	/// A node point lies on an edge drawing that does not start at it.
	NodeOnEdge { node: RspNodeId, constraint: usize },
}

/// Relative clearance used by every check (fraction of the partition diagonal).
const RELATIVE_EPS: f64 = 1e-9;

/// Check I1 (convex tiling), I2 (points interior), I3 (doorways on shared walls, distinct)
/// and that no two edge drawings (`subject → doorway → reference`) conflict.
pub fn validate_rsp_embedding(layer: &RspLayer) -> Result<(), RspEmbeddingError> {
	let graph = &layer.graph;
	let partition = &layer.partition;
	for id in graph.nodes.keys() {
		if !partition.cells.contains_key(id) || !partition.points.contains_key(id) {
			return Err(RspEmbeddingError::MissingGeometry(*id));
		}
	}
	if partition.doorways.len() != graph.constraints.len() {
		return Err(RspEmbeddingError::DoorwayCountMismatch {
			doorways: partition.doorways.len(),
			constraints: graph.constraints.len(),
		});
	}
	let eps = tolerance(layer);

	for id in graph.nodes.keys() {
		if !cell_is_convex(&partition.cells[id], eps) {
			return Err(RspEmbeddingError::InvalidCell(*id));
		}
	}

	let ids: Vec<RspNodeId> = graph.nodes.keys().copied().collect();
	for i in 0..ids.len() {
		for j in (i + 1)..ids.len() {
			if !cells_separated(&partition.cells[&ids[i]], &partition.cells[&ids[j]], eps) {
				return Err(RspEmbeddingError::OverlappingCells(ids[i], ids[j]));
			}
		}
	}

	let covered: f64 = ids.iter().map(|id| polygon_area(&partition.cells[id])).sum();
	let expected = bounding_area(layer);
	if (covered - expected).abs() > RELATIVE_EPS * expected.max(f64::MIN_POSITIVE) * 10.0 {
		return Err(RspEmbeddingError::IncompleteCover { covered, expected });
	}

	for id in &ids {
		if min_inward_distance(&partition.cells[id], partition.points[id]) <= eps {
			return Err(RspEmbeddingError::PointNotInterior(*id));
		}
	}

	for (index, constraint) in graph.constraints.iter().enumerate() {
		let door = partition.doorways[index];
		for node in [constraint.subject, constraint.reference] {
			let Some(cell) = partition.cells.get(&node) else {
				return Err(RspEmbeddingError::MissingGeometry(node));
			};
			let distance = min_inward_distance(cell, door);
			if distance < -eps || distance > eps {
				return Err(RspEmbeddingError::DoorwayOffWall {
					constraint: index,
					node,
				});
			}
		}
	}

	for i in 0..partition.doorways.len() {
		for j in (i + 1)..partition.doorways.len() {
			if length_point(partition.doorways[i] - partition.doorways[j]) <= eps {
				return Err(RspEmbeddingError::SharedDoorway(i, j));
			}
		}
	}

	match first_edge_conflict(layer, eps) {
		Some(error) => Err(error),
		None => Ok(()),
	}
}

/// Number of conflicting pairs among edge drawings and node points (0 for a valid embedding).
pub fn count_edge_conflicts(layer: &RspLayer) -> u32 {
	let eps = tolerance(layer);
	let mut count = 0u32;
	for_each_edge_conflict(layer, eps, |_| {
		count += 1;
		true
	});
	count
}

/// Smallest gaps in an embedding (`INFINITY` when there is nothing to measure).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspClearance {
	/// Minimum distance between two node points.
	pub node_node: f64,
	/// Minimum distance from a node point to an edge half it is not an endpoint of.
	pub node_edge: f64,
}

/// Measure node–node and node–foreign-edge clearance of a layer's drawing.
pub fn measure_clearance(layer: &RspLayer) -> RspClearance {
	let points: Vec<(RspNodeId, RspPoint)> =
		layer.partition.points.iter().map(|(id, p)| (*id, *p)).collect();
	let mut node_node = f64::INFINITY;
	for i in 0..points.len() {
		for j in (i + 1)..points.len() {
			node_node = node_node.min(length_point(points[j].1 - points[i].1));
		}
	}
	let mut node_edge = f64::INFINITY;
	for (index, constraint) in layer.graph.constraints.iter().enumerate() {
		let Some(door) = layer.partition.doorways.get(index).copied() else {
			continue;
		};
		for end in [constraint.subject, constraint.reference] {
			let Some(from) = layer.partition.points.get(&end).copied() else {
				continue;
			};
			for (id, p) in &points {
				if *id != constraint.subject && *id != constraint.reference {
					node_edge = node_edge.min(point_segment_distance(*p, from, door));
				}
			}
		}
	}
	RspClearance {
		node_node,
		node_edge,
	}
}

fn first_edge_conflict(layer: &RspLayer, eps: f64) -> Option<RspEmbeddingError> {
	let mut found = None;
	for_each_edge_conflict(layer, eps, |error| {
		found = Some(error);
		false
	});
	found
}

/// One drawn half of an edge: node point → doorway.
struct HalfEdge {
	constraint: usize,
	node: RspNodeId,
	from: RspPoint,
	to: RspPoint,
}

/// Visit conflicts; `visit` returns `false` to stop.
fn for_each_edge_conflict(
	layer: &RspLayer,
	eps: f64,
	mut visit: impl FnMut(RspEmbeddingError) -> bool,
) {
	let partition = &layer.partition;
	let mut halves: Vec<HalfEdge> = Vec::with_capacity(layer.graph.constraints.len() * 2);
	for (index, constraint) in layer.graph.constraints.iter().enumerate() {
		let Some(door) = partition.doorways.get(index).copied() else {
			continue;
		};
		for node in [constraint.subject, constraint.reference] {
			if let Some(from) = partition.points.get(&node).copied() {
				halves.push(HalfEdge {
					constraint: index,
					node,
					from,
					to: door,
				});
			}
		}
	}

	for i in 0..halves.len() {
		for j in (i + 1)..halves.len() {
			let a = &halves[i];
			let b = &halves[j];
			if a.constraint == b.constraint {
				continue;
			}
			let conflict = if a.node == b.node {
				same_direction(a.to - a.from, b.to - b.from)
			} else {
				segment_distance(a.from, a.to, b.from, b.to) <= eps
			};
			if conflict && !visit(RspEmbeddingError::EdgeConflict(a.constraint, b.constraint)) {
				return;
			}
		}
	}

	for (node, point) in &partition.points {
		for half in &halves {
			if half.node == *node {
				continue;
			}
			if point_segment_distance(*point, half.from, half.to) <= eps
				&& !visit(RspEmbeddingError::NodeOnEdge {
					node: *node,
					constraint: half.constraint,
				}) {
				return;
			}
		}
	}
}

fn tolerance(layer: &RspLayer) -> f64 {
	let (min, max) = bounds(layer);
	let diagonal = length_point(max - min);
	RELATIVE_EPS * diagonal.max(f64::MIN_POSITIVE)
}

fn bounds(layer: &RspLayer) -> (RspPoint, RspPoint) {
	let mut min = RspPoint::new(f64::INFINITY, f64::INFINITY);
	let mut max = RspPoint::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
	for cell in layer.partition.cells.values() {
		for v in &cell.vertices {
			min = RspPoint::new(min.x.min(v.x), min.y.min(v.y));
			max = RspPoint::new(max.x.max(v.x), max.y.max(v.y));
		}
	}
	if !min.x.is_finite() {
		return (RspPoint::default(), RspPoint::default());
	}
	(min, max)
}

/// Area of the seed square, recovered as the bounding box of all cells.
fn bounding_area(layer: &RspLayer) -> f64 {
	let (min, max) = bounds(layer);
	(max.x - min.x) * (max.y - min.y)
}

/// Non-degenerate edges of a cell as `(start, end)` pairs.
fn cell_edges(cell: &RspCell) -> impl Iterator<Item = (RspPoint, RspPoint)> + '_ {
	let n = cell.vertices.len();
	(0..n)
		.map(move |i| (cell.vertices[i], cell.vertices[(i + 1) % n]))
		.filter(|(a, b)| a != b)
}

/// Signed distance of `p` from the line through `a → b`, positive on the inner (left) side.
fn inward_distance(a: RspPoint, b: RspPoint, p: RspPoint) -> f64 {
	let edge = b - a;
	cross_point(edge, p - a) / length_point(edge)
}

/// Smallest inward distance of `p` over all cell edges (negative means outside).
fn min_inward_distance(cell: &RspCell, p: RspPoint) -> f64 {
	cell_edges(cell)
		.map(|(a, b)| inward_distance(a, b, p))
		.fold(f64::INFINITY, f64::min)
}

fn cell_is_convex(cell: &RspCell, eps: f64) -> bool {
	if cell.vertices.len() < 3 || polygon_area(cell) <= eps * eps {
		return false;
	}
	for (a, b) in cell_edges(cell) {
		for v in &cell.vertices {
			if inward_distance(a, b, *v) < -eps {
				return false;
			}
		}
	}
	true
}

/// Separating-axis test: some edge line of one cell has the other cell entirely outside.
fn cells_separated(a: &RspCell, b: &RspCell, eps: f64) -> bool {
	let separated_by = |owner: &RspCell, other: &RspCell| {
		cell_edges(owner).any(|(p, q)| {
			other
				.vertices
				.iter()
				.all(|v| inward_distance(p, q, *v) <= eps)
		})
	};
	separated_by(a, b) || separated_by(b, a)
}

/// Halves leaving the same node overlap only if they point the same way.
fn same_direction(a: RspPoint, b: RspPoint) -> bool {
	let scale = length_point(a) * length_point(b);
	if scale == 0.0 {
		return true;
	}
	cross_point(a, b).abs() <= RELATIVE_EPS * scale && dot_point(a, b) > 0.0
}

fn point_segment_distance(p: RspPoint, a: RspPoint, b: RspPoint) -> f64 {
	let ab = b - a;
	let len_sq = dot_point(ab, ab);
	if len_sq == 0.0 {
		return length_point(p - a);
	}
	let t = (dot_point(p - a, ab) / len_sq).clamp(0.0, 1.0);
	length_point(p - (a + ab * t))
}

fn segment_distance(a0: RspPoint, a1: RspPoint, b0: RspPoint, b1: RspPoint) -> f64 {
	let o1 = cross_point(a1 - a0, b0 - a0);
	let o2 = cross_point(a1 - a0, b1 - a0);
	let o3 = cross_point(b1 - b0, a0 - b0);
	let o4 = cross_point(b1 - b0, a1 - b0);
	if o1 * o2 < 0.0 && o3 * o4 < 0.0 {
		return 0.0;
	}
	point_segment_distance(a0, b0, b1)
		.min(point_segment_distance(a1, b0, b1))
		.min(point_segment_distance(b0, a0, a1))
		.min(point_segment_distance(b1, a0, a1))
}
