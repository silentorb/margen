//! Relative Space Partitioning (RSP) generation.
//!
//! First pass is skeletal: a fixed two-node planar graph for host plumbing.

use margen_rsp::{RspConstraint, RspGraph, RspNodeId, NORTH};
use mythic_math::Vector2;
use std::collections::{BTreeMap, BTreeSet};

/// Two nodes with a single North constraint (planar; no absolute placement).
pub fn generate_skeletal_rsp() -> RspGraph {
	let subject = RspNodeId(0);
	let reference = RspNodeId(1);
	RspGraph {
		nodes: BTreeSet::from([subject, reference]),
		constraints: vec![RspConstraint {
			subject,
			direction: NORTH,
			reference,
		}],
	}
}

/// Trivial editor layout in margen XY (RH: +Y north). Not final spatial realization.
///
/// Places an unplaced reference at the origin (or leaves it), then offsets each
/// unplaced subject by `direction * spacing` from its reference when the
/// reference is already placed. Finally shifts all positions so the bounding-box
/// center sits at the origin (editor/camera-friendly).
pub fn layout_rsp_debug_2d(graph: &RspGraph, spacing: f32) -> Vec<(RspNodeId, Vector2)> {
	let mut positions: BTreeMap<RspNodeId, Vector2> = BTreeMap::new();
	for &id in &graph.nodes {
		positions.insert(id, Vector2::zero());
	}
	if let Some(&first) = graph.nodes.iter().next() {
		positions.insert(first, Vector2::zero());
	}
	for constraint in &graph.constraints {
		let reference_pos = positions
			.get(&constraint.reference)
			.copied()
			.unwrap_or(Vector2::zero());
		let offset = Vector2::new(
			constraint.direction.x as f32 * spacing,
			constraint.direction.y as f32 * spacing,
		);
		positions.insert(constraint.subject, reference_pos + offset);
		positions
			.entry(constraint.reference)
			.or_insert(reference_pos);
	}
	if !positions.is_empty() {
		let mut min_x = f32::INFINITY;
		let mut max_x = f32::NEG_INFINITY;
		let mut min_y = f32::INFINITY;
		let mut max_y = f32::NEG_INFINITY;
		for pos in positions.values() {
			min_x = min_x.min(pos.x);
			max_x = max_x.max(pos.x);
			min_y = min_y.min(pos.y);
			max_y = max_y.max(pos.y);
		}
		let center = Vector2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
		for pos in positions.values_mut() {
			*pos = Vector2::new(pos.x - center.x, pos.y - center.y);
		}
	}
	graph
		.nodes
		.iter()
		.map(|id| (*id, positions.get(id).copied().unwrap_or(Vector2::zero())))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_rsp::{is_planar, NORTH};

	#[test]
	fn skeletal_graph_has_two_nodes_and_north_constraint() {
		let graph = generate_skeletal_rsp();
		assert_eq!(graph.nodes.len(), 2);
		assert!(graph.nodes.contains(&RspNodeId(0)));
		assert!(graph.nodes.contains(&RspNodeId(1)));
		assert_eq!(graph.constraints.len(), 1);
		let c = &graph.constraints[0];
		assert_eq!(c.subject, RspNodeId(0));
		assert_eq!(c.reference, RspNodeId(1));
		assert_eq!(c.direction, NORTH);
		assert!(is_planar(c.direction));
	}

	#[test]
	fn debug_layout_places_north_subject_above_reference() {
		let graph = generate_skeletal_rsp();
		let layout = layout_rsp_debug_2d(&graph, 10.0);
		assert_eq!(layout.len(), 2);
		let subject = layout.iter().find(|(id, _)| *id == RspNodeId(0)).unwrap();
		let reference = layout.iter().find(|(id, _)| *id == RspNodeId(1)).unwrap();
		assert_eq!(subject.1.x, reference.1.x);
		assert!(subject.1.y > reference.1.y);
		let mid_y = (subject.1.y + reference.1.y) * 0.5;
		assert!(mid_y.abs() < 1e-4, "layout should be centered on origin, mid_y={mid_y}");
		assert!((subject.1.x + reference.1.x).abs() < 1e-4);
	}
}
