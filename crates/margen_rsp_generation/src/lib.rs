//! Relative Space Partitioning (RSP) generation.
//!
//! Skeletal path: seed one high-level node, then binary-subdivide until level 0.

use margen_rsp::{
	angle_unit, dot2, opposite_angle, RspAngle, RspConstraint, RspGraph, RspNode, RspNodeId,
};
use mythic_dice::Dice;
use mythic_math::Vector2;
use std::collections::BTreeMap;
use std::f32::consts::TAU;

/// One node at `level` 2, fully subdivided to four level-0 leaves.
pub fn generate_skeletal_rsp(dice: &mut Dice) -> RspGraph {
	let id = RspNodeId(0);
	let seed = RspGraph {
		nodes: BTreeMap::from([(id, RspNode { id, level: 2 })]),
		constraints: Vec::new(),
	};
	subdivide_rsp_fully(seed, dice)
}

/// Repeatedly run [`subdivide_rsp_pass`] until every node has `level == 0`.
pub fn subdivide_rsp_fully(mut graph: RspGraph, dice: &mut Dice) -> RspGraph {
	while graph.nodes.values().any(|node| node.level > 0) {
		graph = subdivide_rsp_pass(&graph, dice);
	}
	graph
}

struct Split {
	child_pos: RspNodeId,
	child_neg: RspNodeId,
	split_angle: RspAngle,
}

/// One subdivision pass: every node with `level > 0` becomes two children at `level - 1`.
///
/// Parent connections are inherited by the **nearest** child only (dot product of the
/// split axis against the parent→other direction from the existing constraint angle).
pub fn subdivide_rsp_pass(graph: &RspGraph, dice: &mut Dice) -> RspGraph {
	let parents: Vec<RspNodeId> = graph
		.nodes
		.values()
		.filter(|node| node.level > 0)
		.map(|node| node.id)
		.collect();

	if parents.is_empty() {
		return graph.clone();
	}

	let mut next_id = graph
		.nodes
		.keys()
		.map(|id| id.0)
		.max()
		.unwrap_or(0)
		.saturating_add(1);

	let mut splits: BTreeMap<RspNodeId, Split> = BTreeMap::new();
	for parent_id in &parents {
		let child_pos = RspNodeId(next_id);
		next_id += 1;
		let child_neg = RspNodeId(next_id);
		next_id += 1;
		let split_angle = dice.get_float_range(0.0, TAU);
		splits.insert(
			*parent_id,
			Split {
				child_pos,
				child_neg,
				split_angle,
			},
		);
	}

	let mut new_nodes: BTreeMap<RspNodeId, RspNode> = BTreeMap::new();
	for node in graph.nodes.values() {
		if let Some(split) = splits.get(&node.id) {
			let child_level = node.level - 1;
			new_nodes.insert(
				split.child_pos,
				RspNode {
					id: split.child_pos,
					level: child_level,
				},
			);
			new_nodes.insert(
				split.child_neg,
				RspNode {
					id: split.child_neg,
					level: child_level,
				},
			);
		} else {
			new_nodes.insert(node.id, *node);
		}
	}

	let mut new_constraints: Vec<RspConstraint> = Vec::new();
	for constraint in &graph.constraints {
		let toward_from_subject_to_reference = angle_unit(opposite_angle(constraint.angle));
		let toward_from_reference_to_subject = angle_unit(constraint.angle);

		let subject = resolve_endpoint(
			constraint.subject,
			&splits,
			toward_from_subject_to_reference,
		);
		let reference = resolve_endpoint(
			constraint.reference,
			&splits,
			toward_from_reference_to_subject,
		);
		new_constraints.push(RspConstraint {
			subject,
			angle: constraint.angle,
			reference,
		});
	}

	for split in splits.values() {
		new_constraints.push(RspConstraint {
			subject: split.child_pos,
			angle: split.split_angle,
			reference: split.child_neg,
		});
	}

	RspGraph {
		nodes: new_nodes,
		constraints: new_constraints,
	}
}

fn resolve_endpoint(
	endpoint: RspNodeId,
	splits: &BTreeMap<RspNodeId, Split>,
	toward_other: Vector2,
) -> RspNodeId {
	match splits.get(&endpoint) {
		Some(split) => nearest_child_id(
			split.child_pos,
			split.child_neg,
			split.split_angle,
			toward_other,
		),
		None => endpoint,
	}
}

/// Binary split: `child_pos` lies at `+angle_unit(split_angle)` from the parent midpoint,
/// `child_neg` at the opposite offset. Picks the child closer to `toward_other`.
pub fn nearest_child_id(
	child_pos: RspNodeId,
	child_neg: RspNodeId,
	split_angle: RspAngle,
	toward_other: Vector2,
) -> RspNodeId {
	let u = angle_unit(split_angle);
	if dot2(toward_other, u) >= 0.0 {
		child_pos
	} else {
		child_neg
	}
}

/// Trivial editor layout in margen XY (RH: +Y north). Not final spatial realization.
///
/// Offsets each subject by `angle_unit(angle) * spacing` from its reference when the
/// reference is already placed. Finally shifts all positions so the bounding-box
/// center sits at the origin (editor/camera-friendly).
pub fn layout_rsp_debug_2d(graph: &RspGraph, spacing: f32) -> Vec<(RspNodeId, Vector2)> {
	let mut positions: BTreeMap<RspNodeId, Vector2> = BTreeMap::new();
	for &id in graph.nodes.keys() {
		positions.insert(id, Vector2::zero());
	}
	if let Some((&first, _)) = graph.nodes.iter().next() {
		positions.insert(first, Vector2::zero());
	}
	for constraint in &graph.constraints {
		let reference_pos = positions
			.get(&constraint.reference)
			.copied()
			.unwrap_or(Vector2::zero());
		let offset = angle_unit(constraint.angle) * spacing;
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
		.keys()
		.map(|id| (*id, positions.get(id).copied().unwrap_or(Vector2::zero())))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_rsp::opposite_angle;
	use std::f32::consts::FRAC_PI_2;

	#[test]
	fn nearest_child_follows_dot_product() {
		let pos = RspNodeId(1);
		let neg = RspNodeId(2);
		// Split along +X; other is east of parent → + child is nearer.
		assert_eq!(
			nearest_child_id(pos, neg, 0.0, Vector2::new(1.0, 0.0)),
			pos
		);
		// Other is west → − child is nearer.
		assert_eq!(
			nearest_child_id(pos, neg, 0.0, Vector2::new(-1.0, 0.0)),
			neg
		);
	}

	#[test]
	fn skeletal_graph_subdivides_level_two_to_four_leaves() {
		let mut dice = Dice::new(0);
		let graph = generate_skeletal_rsp(&mut dice);
		assert_eq!(graph.nodes.len(), 4);
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(!graph.constraints.is_empty());
		for constraint in &graph.constraints {
			assert!(constraint.angle.is_finite());
			assert!(graph.nodes.contains_key(&constraint.subject));
			assert!(graph.nodes.contains_key(&constraint.reference));
		}
	}

	#[test]
	fn single_split_inherits_connection_to_nearest_child_only() {
		let parent = RspNodeId(0);
		let other = RspNodeId(1);
		// Parent is north of other (subject=parent, angle=π/2, reference=other).
		let graph = RspGraph {
			nodes: BTreeMap::from([
				(parent, RspNode { id: parent, level: 1 }),
				(other, RspNode { id: other, level: 0 }),
			]),
			constraints: vec![RspConstraint {
				subject: parent,
				angle: FRAC_PI_2,
				reference: other,
			}],
		};

		// Force a known split: temporarily use dice that returns 0 for the angle.
		// Dice::new(0) first get_float_range(0, TAU) is deterministic; instead call
		// pass and also verify via an explicit second graph built with known split.
		let mut dice = Dice::new(1);
		let split_angle = dice.get_float_range(0.0, TAU);
		// Re-seed and run the real pass (one random draw).
		let mut dice = Dice::new(1);
		let out = subdivide_rsp_pass(&graph, &mut dice);

		assert_eq!(out.nodes.len(), 3); // two children + other
		assert!(out.nodes.contains_key(&other));
		assert!(!out.nodes.contains_key(&parent));

		let inherited: Vec<_> = out
			.constraints
			.iter()
			.filter(|c| c.reference == other || c.subject == other)
			.collect();
		assert_eq!(inherited.len(), 1, "only nearest child inherits the edge");

		let toward_other = angle_unit(opposite_angle(FRAC_PI_2)); // parent → other
		let u = angle_unit(split_angle);
		let expect_pos = dot2(toward_other, u) >= 0.0;
		let c = inherited[0];
		let child = if c.subject == other {
			c.reference
		} else {
			c.subject
		};
		let children: Vec<_> = out
			.nodes
			.keys()
			.copied()
			.filter(|id| *id != other)
			.collect();
		assert_eq!(children.len(), 2);
		let (child_pos, child_neg) = if children[0].0 < children[1].0 {
			(children[0], children[1])
		} else {
			(children[1], children[0])
		};
		// child_pos is allocated first (lower id) in pass order.
		let expected = if expect_pos { child_pos } else { child_neg };
		assert_eq!(child, expected);
		assert!((c.angle - FRAC_PI_2).abs() < 1e-5);
	}

	#[test]
	fn debug_layout_offsets_along_angle() {
		let subject = RspNodeId(0);
		let reference = RspNodeId(1);
		let graph = RspGraph {
			nodes: BTreeMap::from([
				(subject, RspNode { id: subject, level: 0 }),
				(reference, RspNode { id: reference, level: 0 }),
			]),
			constraints: vec![RspConstraint {
				subject,
				angle: FRAC_PI_2,
				reference,
			}],
		};
		let layout = layout_rsp_debug_2d(&graph, 10.0);
		let subject_pos = layout.iter().find(|(id, _)| *id == subject).unwrap().1;
		let reference_pos = layout.iter().find(|(id, _)| *id == reference).unwrap().1;
		let delta = Vector2::new(subject_pos.x - reference_pos.x, subject_pos.y - reference_pos.y);
		assert!(delta.x.abs() < 1e-3, "north offset should be vertical, dx={}", delta.x);
		assert!(delta.y > 0.0, "subject should be north of reference");
		let mid_y = (subject_pos.y + reference_pos.y) * 0.5;
		assert!(mid_y.abs() < 1e-4, "layout should be centered, mid_y={mid_y}");
	}

	#[test]
	fn opposite_angle_used_for_subject_toward_reference() {
		let toward = angle_unit(opposite_angle(0.0));
		assert!((toward.x + 1.0).abs() < 1e-5);
		assert!(toward.y.abs() < 1e-5);
	}
}
