//! Relative Space Partitioning (RSP) generation.
//!
//! Skeletal path: seed one high-level node, then subdivide until level 0 using
//! equal-weight per-level pools of binary / ternary-open / ternary-closed forms.

use margen_rsp::{
	angle_unit, dot2, opposite_angle, RspAngle, RspConstraint, RspGraph, RspNode, RspNodeId,
};
use mythic_dice::Dice;
use mythic_math::Vector2;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::f32::consts::TAU;

/// Subdivision shape applied to one parent node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RspSplitForm {
	Binary,
	TernaryOpen,
	TernaryClosed,
}

const EQUAL_WEIGHT_FORMS: [RspSplitForm; 3] = [
	RspSplitForm::Binary,
	RspSplitForm::TernaryOpen,
	RspSplitForm::TernaryClosed,
];

/// Build a pool of `count` forms with equal weights (counts differ by at most 1),
/// then shuffle with `dice`. Remainder slots go to forms in [`EQUAL_WEIGHT_FORMS`] order.
pub fn allocate_split_forms(count: usize, dice: &mut Dice) -> Vec<RspSplitForm> {
	if count == 0 {
		return Vec::new();
	}
	let n_forms = EQUAL_WEIGHT_FORMS.len();
	let base = count / n_forms;
	let rem = count % n_forms;
	let mut pool = Vec::with_capacity(count);
	for (i, form) in EQUAL_WEIGHT_FORMS.iter().enumerate() {
		let extra = if i < rem { 1 } else { 0 };
		for _ in 0..(base + extra) {
			pool.push(*form);
		}
	}
	shuffle_forms(&mut pool, dice);
	pool
}

fn shuffle_forms(pool: &mut [RspSplitForm], dice: &mut Dice) {
	for i in (1..pool.len()).rev() {
		let j = dice.get_int_range(0, i as i32) as usize;
		pool.swap(i, j);
	}
}

/// One node at `level = subdivisions` (clamped `>= 0`), fully subdivided to level 0.
pub fn generate_skeletal_rsp(dice: &mut Dice, subdivisions: i32) -> RspGraph {
	let level = subdivisions.max(0);
	let id = RspNodeId(0);
	let seed = RspGraph {
		nodes: BTreeMap::from([(id, RspNode { id, level })]),
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

/// One child of a split, with local unit offset from the parent midpoint.
#[derive(Clone, Copy, Debug)]
pub struct ChildSlot {
	pub id: RspNodeId,
	pub offset: Vector2,
}

struct Split {
	children: Vec<ChildSlot>,
	sibling_edges: Vec<(RspNodeId, RspNodeId, RspAngle)>,
}

/// One subdivision pass: every node with `level > 0` is replaced by children.
///
/// Forms are allocated from equal-weight per-level pools (not independent rolls).
/// Parent connections are inherited by the **nearest** child only (max dot product
/// of child offset against the parent→other direction).
pub fn subdivide_rsp_pass(graph: &RspGraph, dice: &mut Dice) -> RspGraph {
	let mut parents_by_level: BTreeMap<i32, Vec<RspNodeId>> = BTreeMap::new();
	for node in graph.nodes.values() {
		if node.level > 0 {
			parents_by_level
				.entry(node.level)
				.or_default()
				.push(node.id);
		}
	}

	if parents_by_level.is_empty() {
		return graph.clone();
	}

	for parents in parents_by_level.values_mut() {
		parents.sort();
	}

	let mut next_id = graph
		.nodes
		.keys()
		.map(|id| id.0)
		.max()
		.unwrap_or(0)
		.saturating_add(1);

	let mut splits: BTreeMap<RspNodeId, Split> = BTreeMap::new();
	for parents in parents_by_level.values() {
		let forms = allocate_split_forms(parents.len(), dice);
		for (parent_id, form) in parents.iter().zip(forms.into_iter()) {
			let split = materialize_split(*parent_id, form, &mut next_id, dice);
			splits.insert(*parent_id, split);
		}
	}

	let mut new_nodes: BTreeMap<RspNodeId, RspNode> = BTreeMap::new();
	for node in graph.nodes.values() {
		if let Some(split) = splits.get(&node.id) {
			let child_level = node.level - 1;
			for child in &split.children {
				new_nodes.insert(
					child.id,
					RspNode {
						id: child.id,
						level: child_level,
					},
				);
			}
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
		for &(subject, reference, angle) in &split.sibling_edges {
			new_constraints.push(RspConstraint {
				subject,
				angle,
				reference,
			});
		}
	}

	RspGraph {
		nodes: new_nodes,
		constraints: new_constraints,
	}
}

fn materialize_split(
	_parent_id: RspNodeId,
	form: RspSplitForm,
	next_id: &mut u32,
	dice: &mut Dice,
) -> Split {
	match form {
		RspSplitForm::Binary => {
			let split_angle = dice.get_float_range(0.0, TAU);
			let u = angle_unit(split_angle);
			let child_pos = alloc_child(next_id);
			let child_neg = alloc_child(next_id);
			Split {
				children: vec![
					ChildSlot {
						id: child_pos,
						offset: u,
					},
					ChildSlot {
						id: child_neg,
						offset: Vector2::new(-u.x, -u.y),
					},
				],
				sibling_edges: vec![(child_pos, child_neg, split_angle)],
			}
		}
		RspSplitForm::TernaryOpen | RspSplitForm::TernaryClosed => {
			let base = dice.get_float_range(0.0, TAU);
			let mut children = Vec::with_capacity(3);
			for k in 0..3 {
				let angle = base + (k as f32) * (TAU / 3.0);
				children.push(ChildSlot {
					id: alloc_child(next_id),
					offset: angle_unit(angle),
				});
			}
			let mut sides = vec![(0usize, 1usize), (1, 2), (2, 0)];
			if form == RspSplitForm::TernaryOpen {
				let drop_index = dice.get_int_range(0, 2) as usize;
				sides.remove(drop_index);
			}
			let sibling_edges = sides
				.into_iter()
				.map(|(i, j)| {
					let a = &children[i];
					let b = &children[j];
					let delta = Vector2::new(b.offset.x - a.offset.x, b.offset.y - a.offset.y);
					let angle = delta.y.atan2(delta.x);
					(b.id, a.id, angle)
				})
				.collect();
			Split {
				children,
				sibling_edges,
			}
		}
	}
}

fn alloc_child(next_id: &mut u32) -> RspNodeId {
	let id = RspNodeId(*next_id);
	*next_id = next_id.saturating_add(1);
	id
}

fn resolve_endpoint(
	endpoint: RspNodeId,
	splits: &BTreeMap<RspNodeId, Split>,
	toward_other: Vector2,
) -> RspNodeId {
	match splits.get(&endpoint) {
		Some(split) => nearest_child(&split.children, toward_other),
		None => endpoint,
	}
}

/// Picks the child whose offset best aligns with `toward_other` (max dot product).
/// Near-ties (within `1e-5`) break to the lowest child id.
pub fn nearest_child(children: &[ChildSlot], toward_other: Vector2) -> RspNodeId {
	assert!(!children.is_empty());
	const TIE_EPS: f32 = 1e-5;
	let mut best_id = children[0].id;
	let mut best_dot = dot2(toward_other, children[0].offset);
	for child in &children[1..] {
		let d = dot2(toward_other, child.offset);
		let better = d > best_dot + TIE_EPS
			|| ((d - best_dot).abs() <= TIE_EPS && child.id < best_id);
		if better {
			best_dot = d;
			best_id = child.id;
		}
	}
	best_id
}

/// Binary convenience: `child_pos` at `+angle_unit(split_angle)`, `child_neg` opposite.
pub fn nearest_child_id(
	child_pos: RspNodeId,
	child_neg: RspNodeId,
	split_angle: RspAngle,
	toward_other: Vector2,
) -> RspNodeId {
	let u = angle_unit(split_angle);
	nearest_child(
		&[
			ChildSlot {
				id: child_pos,
				offset: u,
			},
			ChildSlot {
				id: child_neg,
				offset: Vector2::new(-u.x, -u.y),
			},
		],
		toward_other,
	)
}

/// Trivial editor layout in margen XY (RH: +Y north). Not final spatial realization.
///
/// Places nodes via a spanning-tree walk so cycle-closing edges do not overwrite
/// positions. Finally shifts so the bounding-box center sits at the origin.
pub fn layout_rsp_debug_2d(graph: &RspGraph, spacing: f32) -> Vec<(RspNodeId, Vector2)> {
	let mut positions: BTreeMap<RspNodeId, Vector2> = BTreeMap::new();
	let mut placed: HashSet<RspNodeId> = HashSet::new();

	if graph.nodes.is_empty() {
		return Vec::new();
	}

	let start = *graph.nodes.keys().next().expect("non-empty nodes");
	positions.insert(start, Vector2::zero());
	placed.insert(start);

	let mut queue = VecDeque::from([start]);
	while let Some(current) = queue.pop_front() {
		let current_pos = positions
			.get(&current)
			.copied()
			.unwrap_or(Vector2::zero());
		for constraint in &graph.constraints {
			if constraint.reference == current && !placed.contains(&constraint.subject) {
				let offset = angle_unit(constraint.angle) * spacing;
				positions.insert(constraint.subject, current_pos + offset);
				placed.insert(constraint.subject);
				queue.push_back(constraint.subject);
			} else if constraint.subject == current && !placed.contains(&constraint.reference) {
				let offset = angle_unit(opposite_angle(constraint.angle)) * spacing;
				positions.insert(constraint.reference, current_pos + offset);
				placed.insert(constraint.reference);
				queue.push_back(constraint.reference);
			}
		}
	}

	for &id in graph.nodes.keys() {
		positions.entry(id).or_insert(Vector2::zero());
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
	fn allocate_split_forms_equal_weights() {
		for n in 0..=6 {
			let mut dice = Dice::new(7);
			let pool = allocate_split_forms(n, &mut dice);
			assert_eq!(pool.len(), n);
			let mut counts = [0usize; 3];
			for form in &pool {
				let idx = match form {
					RspSplitForm::Binary => 0,
					RspSplitForm::TernaryOpen => 1,
					RspSplitForm::TernaryClosed => 2,
				};
				counts[idx] += 1;
			}
			if n > 0 {
				let min = *counts.iter().min().unwrap();
				let max = *counts.iter().max().unwrap();
				assert!(max - min <= 1, "n={n} counts={counts:?}");
			}
		}
	}

	#[test]
	fn allocate_split_forms_remainder_order() {
		let mut dice = Dice::new(0);
		// Consume shuffle draws by copying the unshuffled expectation via a fresh pool
		// built the same way then verifying counts for N=4: 2,1,1 in form order base.
		let pool = allocate_split_forms(4, &mut dice);
		let mut counts = [0usize; 3];
		for form in &pool {
			match form {
				RspSplitForm::Binary => counts[0] += 1,
				RspSplitForm::TernaryOpen => counts[1] += 1,
				RspSplitForm::TernaryClosed => counts[2] += 1,
			}
		}
		assert_eq!(counts, [2, 1, 1]);
	}

	#[test]
	fn nearest_child_follows_dot_product() {
		let pos = RspNodeId(1);
		let neg = RspNodeId(2);
		assert_eq!(
			nearest_child_id(pos, neg, 0.0, Vector2::new(1.0, 0.0)),
			pos
		);
		assert_eq!(
			nearest_child_id(pos, neg, 0.0, Vector2::new(-1.0, 0.0)),
			neg
		);
	}

	#[test]
	fn nearest_child_ternary_wedges_and_tie_break() {
		let a = RspNodeId(1);
		let b = RspNodeId(2);
		let c = RspNodeId(3);
		let children = [
			ChildSlot {
				id: a,
				offset: angle_unit(0.0),
			},
			ChildSlot {
				id: b,
				offset: angle_unit(TAU / 3.0),
			},
			ChildSlot {
				id: c,
				offset: angle_unit(2.0 * TAU / 3.0),
			},
		];
		assert_eq!(nearest_child(&children, angle_unit(0.0)), a);
		assert_eq!(nearest_child(&children, angle_unit(TAU / 3.0)), b);
		assert_eq!(nearest_child(&children, angle_unit(2.0 * TAU / 3.0)), c);

		// Exact tie between a and b at midpoint: lowest id wins (a).
		let mid_ab = angle_unit(TAU / 6.0);
		let da = dot2(mid_ab, children[0].offset);
		let db = dot2(mid_ab, children[1].offset);
		assert!((da - db).abs() < 1e-5);
		assert_eq!(nearest_child(&children, mid_ab), a);
	}

	#[test]
	fn skeletal_subdivisions_zero_is_single_leaf() {
		let mut dice = Dice::new(0);
		let graph = generate_skeletal_rsp(&mut dice, 0);
		assert_eq!(graph.nodes.len(), 1);
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(graph.constraints.is_empty());
	}

	#[test]
	fn skeletal_graph_subdivides_to_level_zero() {
		let mut dice = Dice::new(0);
		let graph = generate_skeletal_rsp(&mut dice, 2);
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(!graph.nodes.is_empty());
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

		// With one parent, equal-weight pool of size 1 is Binary (first form).
		let mut dice = Dice::new(1);
		let split_angle = dice.get_float_range(0.0, TAU);
		let mut dice = Dice::new(1);
		let out = subdivide_rsp_pass(&graph, &mut dice);

		assert_eq!(out.nodes.len(), 3);
		assert!(out.nodes.contains_key(&other));
		assert!(!out.nodes.contains_key(&parent));

		let inherited: Vec<_> = out
			.constraints
			.iter()
			.filter(|c| c.reference == other || c.subject == other)
			.collect();
		assert_eq!(inherited.len(), 1, "only nearest child inherits the edge");

		let toward_other = angle_unit(opposite_angle(FRAC_PI_2));
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
		let expected = if expect_pos { child_pos } else { child_neg };
		assert_eq!(child, expected);
		assert!((c.angle - FRAC_PI_2).abs() < 1e-5);
	}

	#[test]
	fn ternary_open_and_closed_sibling_edge_counts() {
		let parent = RspNodeId(0);
		let mut next_id = 1u32;
		let mut dice = Dice::new(3);
		let closed = materialize_split(parent, RspSplitForm::TernaryClosed, &mut next_id, &mut dice);
		assert_eq!(closed.children.len(), 3);
		assert_eq!(closed.sibling_edges.len(), 3);

		let mut next_id = 1u32;
		let mut dice = Dice::new(3);
		let open = materialize_split(parent, RspSplitForm::TernaryOpen, &mut next_id, &mut dice);
		assert_eq!(open.children.len(), 3);
		assert_eq!(open.sibling_edges.len(), 2);

		for child in &closed.children {
			let len_sq = child.offset.x * child.offset.x + child.offset.y * child.offset.y;
			assert!((len_sq - 1.0).abs() < 1e-4);
		}
		let dots = [
			dot2(closed.children[0].offset, closed.children[1].offset),
			dot2(closed.children[1].offset, closed.children[2].offset),
			dot2(closed.children[2].offset, closed.children[0].offset),
		];
		for d in dots {
			assert!((d + 0.5).abs() < 1e-4, "120° offsets, cos=−0.5, got {d}");
		}
	}

	#[test]
	fn ternary_inheritance_nearest_only() {
		let parent = RspNodeId(0);
		let mut next_id = 1u32;
		let mut dice = Dice::new(5);
		let split = materialize_split(parent, RspSplitForm::TernaryClosed, &mut next_id, &mut dice);
		let toward_other = angle_unit(opposite_angle(0.0)); // west
		let expected = nearest_child(&split.children, toward_other);

		let mut splits = BTreeMap::new();
		splits.insert(parent, split);
		let resolved = resolve_endpoint(parent, &splits, toward_other);
		assert_eq!(resolved, expected);
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
	fn debug_layout_cycle_safe_triangle() {
		let a = RspNodeId(0);
		let b = RspNodeId(1);
		let c = RspNodeId(2);
		let graph = RspGraph {
			nodes: BTreeMap::from([
				(a, RspNode { id: a, level: 0 }),
				(b, RspNode { id: b, level: 0 }),
				(c, RspNode { id: c, level: 0 }),
			]),
			constraints: vec![
				RspConstraint {
					subject: b,
					angle: 0.0,
					reference: a,
				},
				RspConstraint {
					subject: c,
					angle: TAU / 3.0,
					reference: b,
				},
				RspConstraint {
					subject: a,
					angle: 2.0 * TAU / 3.0,
					reference: c,
				},
			],
		};
		let layout = layout_rsp_debug_2d(&graph, 10.0);
		assert_eq!(layout.len(), 3);
		for (_, pos) in &layout {
			assert!(pos.x.is_finite());
			assert!(pos.y.is_finite());
		}
		let positions: Vec<_> = layout.iter().map(|(_, p)| *p).collect();
		assert!(
			(positions[0].x - positions[1].x).abs() > 1e-3
				|| (positions[0].y - positions[1].y).abs() > 1e-3
		);
		assert!(
			(positions[1].x - positions[2].x).abs() > 1e-3
				|| (positions[1].y - positions[2].y).abs() > 1e-3
		);
		assert!(
			(positions[2].x - positions[0].x).abs() > 1e-3
				|| (positions[2].y - positions[0].y).abs() > 1e-3
		);
	}

	#[test]
	fn opposite_angle_used_for_subject_toward_reference() {
		let toward = angle_unit(opposite_angle(0.0));
		assert!((toward.x + 1.0).abs() < 1e-5);
		assert!(toward.y.abs() < 1e-5);
	}
}
