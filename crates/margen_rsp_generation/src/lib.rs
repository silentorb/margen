//! Relative Space Partitioning (RSP) generation.
//!
//! Skeletal path: seed one high-level node, then subdivide until level 0 using
//! per-level pools of binary / ternary-open / ternary-closed forms weighted by
//! [`DEFAULT_LOOP_WEIGHT`] (or a caller-supplied `loop_weight`).

use margen_rsp::{
	angle_unit, dot2, opposite_angle, RspAngle, RspConstraint, RspGraph, RspNode, RspNodeId,
};
use mythic_dice::Dice;
use mythic_math::Vector2;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::f32::consts::TAU;

/// Default share of each per-level form pool for [`RspSplitForm::TernaryClosed`].
///
/// Double the original equal-share `1/3`. Binary and TernaryOpen split the remainder.
pub const DEFAULT_LOOP_WEIGHT: f32 = 2.0 / 3.0;

/// Subdivision shape applied to one parent node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RspSplitForm {
	Binary,
	TernaryOpen,
	TernaryClosed,
}

const POOL_FORMS: [RspSplitForm; 3] = [
	RspSplitForm::Binary,
	RspSplitForm::TernaryOpen,
	RspSplitForm::TernaryClosed,
];

fn clamp_loop_weight(loop_weight: f32) -> f32 {
	if !loop_weight.is_finite() {
		return DEFAULT_LOOP_WEIGHT;
	}
	loop_weight.clamp(0.0, 1.0)
}

/// Build a pool of `count` forms from `loop_weight` (TernaryClosed share; Binary and
/// TernaryOpen each get `(1 - w) / 2`), using largest-remainder integer allocation,
/// then shuffle with `dice`. Remainder ties break in Binary → TernaryOpen → TernaryClosed order.
pub fn allocate_split_forms(count: usize, dice: &mut Dice, loop_weight: f32) -> Vec<RspSplitForm> {
	if count == 0 {
		return Vec::new();
	}
	let w_closed = clamp_loop_weight(loop_weight);
	let w_side = (1.0 - w_closed) * 0.5;
	let weights = [w_side, w_side, w_closed];

	let mut floors = [0usize; 3];
	let mut fracs = [0.0f32; 3];
	let mut assigned = 0usize;
	for i in 0..3 {
		let exact = (count as f32) * weights[i];
		floors[i] = exact.floor() as usize;
		fracs[i] = exact - floors[i] as f32;
		assigned += floors[i];
	}
	let mut rem = count - assigned;
	// Prefer strictly larger fractional parts; near-ties keep the lower form index
	// (Binary → TernaryOpen → TernaryClosed) so equal weights stay stable in f32.
	const FRAC_TIE_EPS: f32 = 1e-5;
	while rem > 0 {
		let mut best = 0usize;
		for i in 1..3 {
			if fracs[i] > fracs[best] + FRAC_TIE_EPS {
				best = i;
			}
		}
		floors[best] += 1;
		fracs[best] = f32::NEG_INFINITY;
		rem -= 1;
	}

	let mut pool = Vec::with_capacity(count);
	for (i, form) in POOL_FORMS.iter().enumerate() {
		for _ in 0..floors[i] {
			pool.push(*form);
		}
	}
	debug_assert_eq!(pool.len(), count);
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
///
/// `loop_weight` is clamped to `[0, 1]` (non-finite → [`DEFAULT_LOOP_WEIGHT`]).
pub fn generate_skeletal_rsp(dice: &mut Dice, subdivisions: i32, loop_weight: f32) -> RspGraph {
	let level = subdivisions.max(0);
	let id = RspNodeId(0);
	let seed = RspGraph {
		nodes: BTreeMap::from([(id, RspNode { id, level })]),
		constraints: Vec::new(),
	};
	subdivide_rsp_fully(seed, dice, clamp_loop_weight(loop_weight))
}

/// Repeatedly run [`subdivide_rsp_pass`] until every node has `level == 0`.
pub fn subdivide_rsp_fully(mut graph: RspGraph, dice: &mut Dice, loop_weight: f32) -> RspGraph {
	let loop_weight = clamp_loop_weight(loop_weight);
	while graph.nodes.values().any(|node| node.level > 0) {
		graph = subdivide_rsp_pass(&graph, dice, loop_weight);
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

/// Unitless parent-edge span used when recomputing inherited angles.
const INHERIT_PARENT_SPAN: f32 = 1.0;
/// Child slot radius relative to [`INHERIT_PARENT_SPAN`] in the local embedding.
const INHERIT_CHILD_RADIUS: f32 = 0.5;
const INHERIT_DEGENERATE_EPS: f32 = 1e-8;

/// One subdivision pass: every node with `level > 0` is replaced by children.
///
/// Forms are allocated from `loop_weight`-weighted per-level pools (not independent rolls).
/// Parent connections are inherited by the **nearest** child only (max dot product
/// of child offset against the parent→other direction). Inherited angles are
/// **recomputed** from a local parent-edge embedding that includes child offsets.
pub fn subdivide_rsp_pass(graph: &RspGraph, dice: &mut Dice, loop_weight: f32) -> RspGraph {
	let loop_weight = clamp_loop_weight(loop_weight);
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
		let forms = allocate_split_forms(parents.len(), dice, loop_weight);
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
		let angle = recompute_inherited_angle(
			constraint.angle,
			constraint.subject,
			constraint.reference,
			subject,
			reference,
			&splits,
		);
		new_constraints.push(RspConstraint {
			subject,
			angle,
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

fn endpoint_embed_pos(
	parent_id: RspNodeId,
	resolved_id: RspNodeId,
	parent_pos: Vector2,
	splits: &BTreeMap<RspNodeId, Split>,
) -> Vector2 {
	match splits.get(&parent_id) {
		Some(split) => {
			let offset = split
				.children
				.iter()
				.find(|child| child.id == resolved_id)
				.map(|child| child.offset)
				.unwrap_or(Vector2::zero());
			parent_pos + offset * INHERIT_CHILD_RADIUS
		}
		None => parent_pos,
	}
}

/// Local parent-edge embedding: reference parent at origin, subject parent at
/// `angle_unit(parent_angle) * INHERIT_PARENT_SPAN`, children at parent ± offset·r.
/// Returns `parent_angle` if the resolved delta is degenerate.
pub(crate) fn recompute_inherited_angle(
	parent_angle: RspAngle,
	subject_parent: RspNodeId,
	reference_parent: RspNodeId,
	subject: RspNodeId,
	reference: RspNodeId,
	splits: &BTreeMap<RspNodeId, Split>,
) -> RspAngle {
	let reference_parent_pos = Vector2::zero();
	let subject_parent_pos = angle_unit(parent_angle) * INHERIT_PARENT_SPAN;
	let subject_pos = endpoint_embed_pos(subject_parent, subject, subject_parent_pos, splits);
	let reference_pos = endpoint_embed_pos(reference_parent, reference, reference_parent_pos, splits);
	let delta = subject_pos - reference_pos;
	let len_sq = delta.x * delta.x + delta.y * delta.y;
	if len_sq < INHERIT_DEGENERATE_EPS * INHERIT_DEGENERATE_EPS {
		return parent_angle;
	}
	delta.y.atan2(delta.x)
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

/// Spanning-tree placement for metric-layout initialization (margen XY, +Y north).
///
/// Cycle-closing edges do not overwrite positions. Bounding-box center at origin.
pub fn layout_rsp_spanning_tree_2d(graph: &RspGraph, spacing: f32) -> Vec<(RspNodeId, Vector2)> {
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

	center_positions(&mut positions);
	positions_vec(graph, &positions)
}

/// Parameters for [`layout_rsp_metric_2d`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspLayoutConfig {
	pub preferred_edge_length: f32,
	pub min_separation: f32,
	pub angle_tolerance: f32,
	pub max_iterations: u32,
}

impl Default for RspLayoutConfig {
	fn default() -> Self {
		let preferred_edge_length = 120.0;
		Self {
			preferred_edge_length,
			min_separation: preferred_edge_length * 0.35,
			angle_tolerance: 5.0_f32.to_radians(),
			max_iterations: 250,
		}
	}
}

/// Quality metrics from [`layout_rsp_metric_2d`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspLayoutStats {
	pub iterations: u32,
	pub min_pairwise_distance: f32,
	pub max_angle_residual: f32,
	pub mean_edge_length: f32,
}

/// Metric realization of an abstract [`RspGraph`] (derived positions, not stored on the graph).
#[derive(Clone, Debug, PartialEq)]
pub struct RspLayout {
	pub positions: Vec<(RspNodeId, Vector2)>,
	pub stats: RspLayoutStats,
}

/// Pipeline layout: hard min-separation, soft angle bands, stretchable edge lengths.
///
/// Initializes from [`layout_rsp_spanning_tree_2d`], then relaxes with a global force model.
/// Always returns positions; inspect [`RspLayoutStats`] for quality.
pub fn layout_rsp_metric_2d(graph: &RspGraph, config: &RspLayoutConfig) -> RspLayout {
	if graph.nodes.is_empty() {
		return RspLayout {
			positions: Vec::new(),
			stats: RspLayoutStats {
				iterations: 0,
				min_pairwise_distance: f32::INFINITY,
				max_angle_residual: 0.0,
				mean_edge_length: 0.0,
			},
		};
	}

	let preferred = config.preferred_edge_length.max(1e-3);
	let min_sep = config.min_separation.max(0.0);
	let angle_tol = config.angle_tolerance.max(0.0);
	let max_iters = config.max_iterations;

	let init = layout_rsp_spanning_tree_2d(graph, preferred);
	let mut positions: BTreeMap<RspNodeId, Vector2> = init.into_iter().collect();

	const ANGLE_STRENGTH: f32 = 0.35;
	const LENGTH_STRENGTH: f32 = 0.08;
	const SEP_STRENGTH: f32 = 0.55;
	const STEP: f32 = 0.45;
	const DAMP: f32 = 0.85;

	let mut velocities: BTreeMap<RspNodeId, Vector2> = BTreeMap::new();
	for &id in graph.nodes.keys() {
		velocities.insert(id, Vector2::zero());
	}

	let mut iterations = 0u32;
	for iter in 0..max_iters {
		iterations = iter + 1;
		let mut forces: BTreeMap<RspNodeId, Vector2> = BTreeMap::new();
		for &id in graph.nodes.keys() {
			forces.insert(id, Vector2::zero());
		}

		for constraint in &graph.constraints {
			let Some(&subject_pos) = positions.get(&constraint.subject) else {
				continue;
			};
			let Some(&reference_pos) = positions.get(&constraint.reference) else {
				continue;
			};
			let delta = subject_pos - reference_pos;
			let dist_sq = delta.x * delta.x + delta.y * delta.y;
			if dist_sq < 1e-10 {
				continue;
			}
			let dist = dist_sq.sqrt();
			let radial = Vector2::new(delta.x / dist, delta.y / dist);
			let tangent = Vector2::new(-radial.y, radial.x);

			let actual = delta.y.atan2(delta.x);
			let err = wrap_angle_to_pi(actual - constraint.angle);
			let abs_err = err.abs();
			if abs_err > angle_tol {
				let excess = (abs_err - angle_tol) * err.signum();
				let force = tangent * (excess * ANGLE_STRENGTH * preferred);
				add_force(&mut forces, constraint.subject, force * -1.0);
				add_force(&mut forces, constraint.reference, force);
			}

			let length_err = dist - preferred;
			let length_force = radial * (length_err * LENGTH_STRENGTH);
			add_force(&mut forces, constraint.subject, length_force * -1.0);
			add_force(&mut forces, constraint.reference, length_force);
		}

		if min_sep > 0.0 {
			let ids: Vec<RspNodeId> = graph.nodes.keys().copied().collect();
			for i in 0..ids.len() {
				for j in (i + 1)..ids.len() {
					let a = ids[i];
					let b = ids[j];
					let pa = positions[&a];
					let pb = positions[&b];
					let delta = pb - pa;
					let dist_sq = delta.x * delta.x + delta.y * delta.y;
					if dist_sq < 1e-12 {
						let push = Vector2::new(min_sep * 0.5, 0.0);
						add_force(&mut forces, a, push * -1.0);
						add_force(&mut forces, b, push);
						continue;
					}
					let dist = dist_sq.sqrt();
					if dist < min_sep {
						let overlap = min_sep - dist;
						let dir = Vector2::new(delta.x / dist, delta.y / dist);
						let force = dir * (overlap * SEP_STRENGTH);
						add_force(&mut forces, a, force * -1.0);
						add_force(&mut forces, b, force);
					}
				}
			}
		}

		for (&id, force) in &forces {
			let vel = velocities.entry(id).or_insert(Vector2::zero());
			*vel = Vector2::new(vel.x * DAMP + force.x * STEP, vel.y * DAMP + force.y * STEP);
			if let Some(pos) = positions.get_mut(&id) {
				*pos = Vector2::new(pos.x + vel.x, pos.y + vel.y);
			}
		}
		center_positions(&mut positions);
	}

	project_min_separation(&mut positions, min_sep, 40);
	center_positions(&mut positions);

	let stats = compute_layout_stats(graph, &positions, angle_tol, iterations);
	RspLayout {
		positions: positions_vec(graph, &positions),
		stats,
	}
}

fn add_force(forces: &mut BTreeMap<RspNodeId, Vector2>, id: RspNodeId, delta: Vector2) {
	let entry = forces.entry(id).or_insert(Vector2::zero());
	*entry = Vector2::new(entry.x + delta.x, entry.y + delta.y);
}

fn wrap_angle_to_pi(angle: f32) -> f32 {
	let mut a = angle % TAU;
	if a > std::f32::consts::PI {
		a -= TAU;
	} else if a < -std::f32::consts::PI {
		a += TAU;
	}
	a
}

fn center_positions(positions: &mut BTreeMap<RspNodeId, Vector2>) {
	if positions.is_empty() {
		return;
	}
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

fn positions_vec(
	graph: &RspGraph,
	positions: &BTreeMap<RspNodeId, Vector2>,
) -> Vec<(RspNodeId, Vector2)> {
	graph
		.nodes
		.keys()
		.map(|id| (*id, positions.get(id).copied().unwrap_or(Vector2::zero())))
		.collect()
}

fn project_min_separation(
	positions: &mut BTreeMap<RspNodeId, Vector2>,
	min_sep: f32,
	passes: u32,
) {
	if min_sep <= 0.0 || positions.len() < 2 {
		return;
	}
	let ids: Vec<RspNodeId> = positions.keys().copied().collect();
	for _ in 0..passes {
		let mut moved = false;
		for i in 0..ids.len() {
			for j in (i + 1)..ids.len() {
				let a = ids[i];
				let b = ids[j];
				let pa = positions[&a];
				let pb = positions[&b];
				let delta = pb - pa;
				let dist_sq = delta.x * delta.x + delta.y * delta.y;
				if dist_sq < 1e-12 {
					let half = min_sep * 0.5;
					positions.insert(a, Vector2::new(pa.x - half, pa.y));
					positions.insert(b, Vector2::new(pb.x + half, pb.y));
					moved = true;
					continue;
				}
				let dist = dist_sq.sqrt();
				if dist < min_sep {
					let push = (min_sep - dist) * 0.5;
					let dir = Vector2::new(delta.x / dist, delta.y / dist);
					positions.insert(a, pa - dir * push);
					positions.insert(b, pb + dir * push);
					moved = true;
				}
			}
		}
		if !moved {
			break;
		}
	}
}

fn compute_layout_stats(
	graph: &RspGraph,
	positions: &BTreeMap<RspNodeId, Vector2>,
	angle_tol: f32,
	iterations: u32,
) -> RspLayoutStats {
	let mut min_pairwise = f32::INFINITY;
	let ids: Vec<RspNodeId> = graph.nodes.keys().copied().collect();
	for i in 0..ids.len() {
		for j in (i + 1)..ids.len() {
			let pa = positions[&ids[i]];
			let pb = positions[&ids[j]];
			let dx = pb.x - pa.x;
			let dy = pb.y - pa.y;
			min_pairwise = min_pairwise.min((dx * dx + dy * dy).sqrt());
		}
	}
	if !min_pairwise.is_finite() {
		min_pairwise = 0.0;
	}

	let mut max_residual = 0.0f32;
	let mut length_sum = 0.0f32;
	let mut length_count = 0u32;
	for constraint in &graph.constraints {
		let Some(&subject_pos) = positions.get(&constraint.subject) else {
			continue;
		};
		let Some(&reference_pos) = positions.get(&constraint.reference) else {
			continue;
		};
		let delta = subject_pos - reference_pos;
		let dist = (delta.x * delta.x + delta.y * delta.y).sqrt();
		if dist > 1e-8 {
			length_sum += dist;
			length_count += 1;
			let actual = delta.y.atan2(delta.x);
			let abs_err = wrap_angle_to_pi(actual - constraint.angle).abs();
			let excess = (abs_err - angle_tol).max(0.0);
			max_residual = max_residual.max(excess);
		}
	}
	let mean_edge_length = if length_count > 0 {
		length_sum / length_count as f32
	} else {
		0.0
	};

	RspLayoutStats {
		iterations,
		min_pairwise_distance: min_pairwise,
		max_angle_residual: max_residual,
		mean_edge_length,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_rsp::opposite_angle;
	use std::f32::consts::FRAC_PI_2;

	fn count_forms(pool: &[RspSplitForm]) -> [usize; 3] {
		let mut counts = [0usize; 3];
		for form in pool {
			let idx = match form {
				RspSplitForm::Binary => 0,
				RspSplitForm::TernaryOpen => 1,
				RspSplitForm::TernaryClosed => 2,
			};
			counts[idx] += 1;
		}
		counts
	}

	#[test]
	fn allocate_split_forms_equal_third_is_balanced() {
		for n in 0..=6 {
			let mut dice = Dice::new(7);
			let pool = allocate_split_forms(n, &mut dice, 1.0 / 3.0);
			assert_eq!(pool.len(), n);
			let counts = count_forms(&pool);
			if n > 0 {
				let min = *counts.iter().min().unwrap();
				let max = *counts.iter().max().unwrap();
				assert!(max - min <= 1, "n={n} counts={counts:?}");
			}
		}
	}

	#[test]
	fn allocate_split_forms_equal_third_remainder_to_binary() {
		let mut dice = Dice::new(0);
		// N=4 with equal thirds: floors 1,1,1 then largest-remainder tie → Binary.
		let pool = allocate_split_forms(4, &mut dice, 1.0 / 3.0);
		assert_eq!(count_forms(&pool), [2, 1, 1]);
	}

	#[test]
	fn allocate_split_forms_default_loop_weight_n6() {
		let mut dice = Dice::new(0);
		let pool = allocate_split_forms(6, &mut dice, DEFAULT_LOOP_WEIGHT);
		assert_eq!(count_forms(&pool), [1, 1, 4]);
	}

	#[test]
	fn allocate_split_forms_extremes() {
		let mut dice = Dice::new(1);
		let none = allocate_split_forms(5, &mut dice, 0.0);
		assert_eq!(count_forms(&none)[2], 0);
		assert_eq!(count_forms(&none)[0] + count_forms(&none)[1], 5);

		let mut dice = Dice::new(1);
		let all = allocate_split_forms(5, &mut dice, 1.0);
		assert_eq!(count_forms(&all), [0, 0, 5]);
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
		let graph = generate_skeletal_rsp(&mut dice, 0, DEFAULT_LOOP_WEIGHT);
		assert_eq!(graph.nodes.len(), 1);
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(graph.constraints.is_empty());
	}

	#[test]
	fn skeletal_graph_subdivides_to_level_zero() {
		let mut dice = Dice::new(0);
		let graph = generate_skeletal_rsp(&mut dice, 2, DEFAULT_LOOP_WEIGHT);
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

		// Equal thirds + N=1 → Binary (largest-remainder tie breaks to first form).
		let mut dice = Dice::new(1);
		let split_angle = dice.get_float_range(0.0, TAU);
		let mut dice = Dice::new(1);
		let out = subdivide_rsp_pass(&graph, &mut dice, 1.0 / 3.0);

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

		let child_offset = if expect_pos {
			u
		} else {
			Vector2::new(-u.x, -u.y)
		};
		let subject_pos = angle_unit(FRAC_PI_2) * INHERIT_PARENT_SPAN + child_offset * INHERIT_CHILD_RADIUS;
		let reference_pos = Vector2::zero();
		let delta = subject_pos - reference_pos;
		let expect_angle = delta.y.atan2(delta.x);
		assert!(
			(c.angle - expect_angle).abs() < 1e-4,
			"inherited angle should follow local embedding, got {} expect {}",
			c.angle,
			expect_angle
		);
		// Off-axis child offset should tilt away from the raw parent angle.
		assert!((c.angle - FRAC_PI_2).abs() > 1e-3 || child_offset.x.abs() < 1e-5);
	}

	#[test]
	fn inherited_angle_reflects_both_endpoint_offsets() {
		let a = RspNodeId(0);
		let b = RspNodeId(1);
		let child_a = RspNodeId(10);
		let child_b = RspNodeId(20);
		let mut splits = BTreeMap::new();
		splits.insert(
			a,
			Split {
				children: vec![ChildSlot {
					id: child_a,
					offset: angle_unit(0.0),
				}],
				sibling_edges: Vec::new(),
			},
		);
		splits.insert(
			b,
			Split {
				children: vec![ChildSlot {
					id: child_b,
					offset: angle_unit(FRAC_PI_2),
				}],
				sibling_edges: Vec::new(),
			},
		);
		let parent_angle = 0.0;
		let got = recompute_inherited_angle(parent_angle, a, b, child_a, child_b, &splits);
		let subject_pos = angle_unit(parent_angle) * INHERIT_PARENT_SPAN
			+ angle_unit(0.0) * INHERIT_CHILD_RADIUS;
		let reference_pos = Vector2::zero() + angle_unit(FRAC_PI_2) * INHERIT_CHILD_RADIUS;
		let delta = subject_pos - reference_pos;
		let expect = delta.y.atan2(delta.x);
		assert!((got - expect).abs() < 1e-5);
		assert!((got - parent_angle).abs() > 1e-3);
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
	fn spanning_tree_layout_offsets_along_angle() {
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
		let layout = layout_rsp_spanning_tree_2d(&graph, 10.0);
		let subject_pos = layout.iter().find(|(id, _)| *id == subject).unwrap().1;
		let reference_pos = layout.iter().find(|(id, _)| *id == reference).unwrap().1;
		let delta = Vector2::new(subject_pos.x - reference_pos.x, subject_pos.y - reference_pos.y);
		assert!(delta.x.abs() < 1e-3, "north offset should be vertical, dx={}", delta.x);
		assert!(delta.y > 0.0, "subject should be north of reference");
		let mid_y = (subject_pos.y + reference_pos.y) * 0.5;
		assert!(mid_y.abs() < 1e-4, "layout should be centered, mid_y={mid_y}");
	}

	#[test]
	fn spanning_tree_layout_cycle_safe_triangle() {
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
		let layout = layout_rsp_spanning_tree_2d(&graph, 10.0);
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

	fn triangle_graph() -> RspGraph {
		let a = RspNodeId(0);
		let b = RspNodeId(1);
		let c = RspNodeId(2);
		RspGraph {
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
		}
	}

	#[test]
	fn metric_layout_two_node_honors_angle_within_tolerance() {
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
		let config = RspLayoutConfig::default();
		let layout = layout_rsp_metric_2d(&graph, &config);
		let subject_pos = layout.positions.iter().find(|(id, _)| *id == subject).unwrap().1;
		let reference_pos = layout.positions.iter().find(|(id, _)| *id == reference).unwrap().1;
		let delta = subject_pos - reference_pos;
		let actual = delta.y.atan2(delta.x);
		let err = wrap_angle_to_pi(actual - FRAC_PI_2).abs();
		assert!(
			err <= config.angle_tolerance + 1e-3,
			"angle err {err} exceeds tolerance {}",
			config.angle_tolerance
		);
		assert!(layout.stats.min_pairwise_distance + 1e-3 >= config.min_separation);
	}

	#[test]
	fn metric_layout_triangle_respects_min_separation() {
		let graph = triangle_graph();
		let config = RspLayoutConfig {
			preferred_edge_length: 10.0,
			min_separation: 3.5,
			angle_tolerance: 5.0_f32.to_radians(),
			max_iterations: 200,
		};
		let layout = layout_rsp_metric_2d(&graph, &config);
		assert_eq!(layout.positions.len(), 3);
		assert!(layout.stats.min_pairwise_distance + 1e-3 >= config.min_separation);
		assert!(layout.stats.max_angle_residual.is_finite());
		assert!(layout.stats.mean_edge_length.is_finite());
	}

	#[test]
	fn metric_layout_skeletal_seed0_sub4_loop06_min_separation() {
		let mut dice = Dice::new(0);
		let graph = generate_skeletal_rsp(&mut dice, 4, 0.6);
		let config = RspLayoutConfig::default();
		let layout = layout_rsp_metric_2d(&graph, &config);
		assert!(!layout.positions.is_empty());
		assert!(
			layout.stats.min_pairwise_distance + 1e-2 >= config.min_separation,
			"min pairwise {} < min_separation {}",
			layout.stats.min_pairwise_distance,
			config.min_separation
		);
		assert!(layout.stats.max_angle_residual.is_finite());
		assert!(layout.stats.mean_edge_length.is_finite());
		assert!(layout.stats.iterations > 0);
	}

	#[test]
	fn opposite_angle_used_for_subject_toward_reference() {
		let toward = angle_unit(opposite_angle(0.0));
		assert!((toward.x + 1.0).abs() < 1e-5);
		assert!(toward.y.abs() < 1e-5);
	}
}
