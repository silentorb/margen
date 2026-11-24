//! Relative Space Partitioning (RSP) generation.
//!
//! Skeletal path: seed one high-level node, then subdivide until level 0 using
//! per-level pools of binary / ternary-open / ternary-closed forms weighted by
//! [`DEFAULT_LOOP_WEIGHT`] (or a caller-supplied `loop_weight`).
//!
//! Every node owns a convex cell and children split their parent's cell, so
//! each subtree stays inside its ancestor's bounds. Each edge passes through a
//! doorway on the wall shared by its endpoint cells. Together these rule out
//! node and edge overlaps at every layer (proof in `docs/rsp.md`).

mod layout;
mod partition;
mod validate;

pub use layout::{
	layout_rsp_metric_2d, RspLayout, RspLayoutConfig, RspLayoutStats, CELL_CLAMP_SCALE,
};
pub use partition::{polygon_area, polygon_centroid, seed_cell};
pub use validate::{
	count_edge_conflicts, measure_clearance, validate_rsp_embedding, RspClearance,
	RspEmbeddingError,
};

use margen_rsp::{
	angle_between_points, RspCell, RspConstraint, RspGraph, RspHistory, RspLayer, RspNode,
	RspNodeId, RspPartition, RspPoint,
};
use mythic_dice::Dice;
use partition::split_by_rays;
use std::collections::BTreeMap;
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

/// Seed layer: one node at `level` owning the [`seed_cell`] (unit square), no edges.
pub fn seed_rsp_layer(level: i32) -> RspLayer {
	let id = RspNodeId(0);
	let cell = seed_cell();
	let point = polygon_centroid(&cell);
	RspLayer {
		graph: RspGraph {
			nodes: BTreeMap::from([(id, RspNode {
				id,
				level: level.max(0),
				parent: None,
			})]),
			constraints: Vec::new(),
		},
		partition: RspPartition {
			cells: BTreeMap::from([(id, cell)]),
			points: BTreeMap::from([(id, point)]),
			doorways: Vec::new(),
		},
	}
}

/// One node at `level = subdivisions` (clamped `>= 0`), fully subdivided to level 0.
///
/// Returns an [`RspHistory`] whose first layer is the seed and whose last layer is leaves.
/// `loop_weight` is clamped to `[0, 1]` (non-finite → [`DEFAULT_LOOP_WEIGHT`]).
pub fn generate_skeletal_rsp(dice: &mut Dice, subdivisions: i32, loop_weight: f32) -> RspHistory {
	subdivide_rsp_fully(seed_rsp_layer(subdivisions), dice, clamp_loop_weight(loop_weight))
}

/// Repeatedly run [`subdivide_rsp_pass`] until every node has `level == 0`.
///
/// Layer `[0]` is the input; each subsequent layer is the result of one pass.
pub fn subdivide_rsp_fully(mut layer: RspLayer, dice: &mut Dice, loop_weight: f32) -> RspHistory {
	let loop_weight = clamp_loop_weight(loop_weight);
	let mut layers = vec![layer.clone()];
	while layer.graph.nodes.values().any(|node| node.level > 0) {
		layer = subdivide_rsp_pass(&layer, dice, loop_weight);
		layers.push(layer.clone());
	}
	RspHistory { layers }
}

/// One child of a split: its cell and its point (the cell centroid).
#[derive(Clone, Debug)]
pub struct ChildCell {
	pub id: RspNodeId,
	pub cell: RspCell,
	pub point: RspPoint,
}

/// New sibling edge: subject, reference, and its doorway on their shared wall.
#[derive(Clone, Copy, Debug)]
pub struct SiblingEdge {
	pub subject: RspNodeId,
	pub reference: RspNodeId,
	pub doorway: RspPoint,
}

/// Children and sibling edges produced by splitting one parent cell.
#[derive(Clone, Debug)]
pub struct Split {
	pub children: Vec<ChildCell>,
	pub sibling_edges: Vec<SiblingEdge>,
}

/// One subdivision pass: every node with `level > 0` is replaced by children
/// whose cells tile the parent's cell.
///
/// Forms are allocated from `loop_weight`-weighted per-level pools (not independent rolls).
/// Each parent connection is inherited by the child whose cell holds the edge's doorway
/// (lowest child id when the doorway sits on a split wall). Inherited and sibling angles
/// are the directions between the resulting node points.
pub fn subdivide_rsp_pass(layer: &RspLayer, dice: &mut Dice, loop_weight: f32) -> RspLayer {
	let loop_weight = clamp_loop_weight(loop_weight);
	let graph = &layer.graph;
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
		return layer.clone();
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
			let cell = &layer.partition.cells[parent_id];
			let split = materialize_split(cell, form, &mut next_id, dice);
			splits.insert(*parent_id, split);
		}
	}

	let mut nodes: BTreeMap<RspNodeId, RspNode> = BTreeMap::new();
	let mut cells: BTreeMap<RspNodeId, RspCell> = BTreeMap::new();
	let mut points: BTreeMap<RspNodeId, RspPoint> = BTreeMap::new();
	for node in graph.nodes.values() {
		if let Some(split) = splits.get(&node.id) {
			for child in &split.children {
				nodes.insert(child.id, RspNode {
					id: child.id,
					level: node.level - 1,
					parent: Some(node.id),
				});
				cells.insert(child.id, child.cell.clone());
				points.insert(child.id, child.point);
			}
		} else {
			nodes.insert(node.id, *node);
			cells.insert(node.id, layer.partition.cells[&node.id].clone());
			points.insert(node.id, layer.partition.points[&node.id]);
		}
	}

	let mut constraints: Vec<RspConstraint> = Vec::new();
	let mut doorways: Vec<RspPoint> = Vec::new();
	for (constraint, doorway) in graph.constraints.iter().zip(&layer.partition.doorways) {
		let subject = doorway_owner(constraint.subject, *doorway, &splits);
		let reference = doorway_owner(constraint.reference, *doorway, &splits);
		constraints.push(RspConstraint {
			subject,
			angle: angle_between_points(points[&reference], points[&subject]),
			reference,
		});
		doorways.push(*doorway);
	}

	for split in splits.values() {
		for edge in &split.sibling_edges {
			constraints.push(RspConstraint {
				subject: edge.subject,
				angle: angle_between_points(points[&edge.reference], points[&edge.subject]),
				reference: edge.reference,
			});
			doorways.push(edge.doorway);
		}
	}

	RspLayer {
		graph: RspGraph { nodes, constraints },
		partition: RspPartition {
			cells,
			points,
			doorways,
		},
	}
}

/// Split one parent cell.
///
/// - **Binary:** a line through the parent centroid at a random angle; the sibling
///   doorway is the centroid itself (on the wall, strictly inside the parent).
/// - **Ternary:** three rays at 120° from the centroid; sibling doorways are wall midpoints.
///   TernaryOpen drops one sibling edge (and its doorway) at random.
///
/// Dice usage matches the forms: one angle per split, plus one drop roll for TernaryOpen.
pub fn materialize_split(
	parent_cell: &RspCell,
	form: RspSplitForm,
	next_id: &mut u32,
	dice: &mut Dice,
) -> Split {
	let center = polygon_centroid(parent_cell);
	match form {
		RspSplitForm::Binary => {
			let split_angle = f64::from(dice.get_float_range(0.0, TAU));
			let u = RspPoint::new(split_angle.cos(), split_angle.sin());
			let rays = split_by_rays(parent_cell, center, &[u, u * -1.0], &[false, false]);
			let children = make_children(rays.cells, next_id);
			let edge = SiblingEdge {
				subject: children[0].id,
				reference: children[1].id,
				doorway: center,
			};
			Split {
				children,
				sibling_edges: vec![edge],
			}
		}
		RspSplitForm::TernaryOpen | RspSplitForm::TernaryClosed => {
			let base = f64::from(dice.get_float_range(0.0, TAU));
			let directions: Vec<RspPoint> = (0..3)
				.map(|k| {
					let angle = base + (k as f64) * std::f64::consts::TAU / 3.0;
					RspPoint::new(angle.cos(), angle.sin())
				})
				.collect();
			// Side (i, j = i + 1) joins wedges i and j across wall (ray) j.
			let mut sides = vec![(0usize, 1usize), (1, 2), (2, 0)];
			if form == RspSplitForm::TernaryOpen {
				let drop_index = dice.get_int_range(0, 2) as usize;
				sides.remove(drop_index);
			}
			let mut wall_doorway = [false; 3];
			for &(_, j) in &sides {
				wall_doorway[j] = true;
			}
			let rays = split_by_rays(parent_cell, center, &directions, &wall_doorway);
			let children = make_children(rays.cells, next_id);
			let sibling_edges = sides
				.into_iter()
				.map(|(i, j)| SiblingEdge {
					subject: children[j].id,
					reference: children[i].id,
					doorway: rays.wall_doorways[j].expect("kept side has a wall doorway"),
				})
				.collect();
			Split {
				children,
				sibling_edges,
			}
		}
	}
}

fn make_children(cells: Vec<RspCell>, next_id: &mut u32) -> Vec<ChildCell> {
	cells
		.into_iter()
		.map(|cell| {
			let id = RspNodeId(*next_id);
			*next_id = next_id.saturating_add(1);
			let point = polygon_centroid(&cell);
			ChildCell { id, cell, point }
		})
		.collect()
}

/// Endpoint after a split: the lowest-id child whose cell has `doorway` as a vertex.
///
/// Doorways are stored as cell vertices and splits copy boundary vertices verbatim,
/// so exactly the owning child (or both children, when the doorway lies on a split
/// wall) contains a bit-identical copy.
fn doorway_owner(
	endpoint: RspNodeId,
	doorway: RspPoint,
	splits: &BTreeMap<RspNodeId, Split>,
) -> RspNodeId {
	match splits.get(&endpoint) {
		Some(split) => split
			.children
			.iter()
			.find(|child| child.cell.vertices.contains(&doorway))
			.map(|child| child.id)
			.expect("parent doorway must be a vertex of some child cell"),
		None => endpoint,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_rsp::{angle_unit, opposite_angle};
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
	fn skeletal_subdivisions_zero_is_single_leaf() {
		let mut dice = Dice::new(0);
		let history = generate_skeletal_rsp(&mut dice, 0, DEFAULT_LOOP_WEIGHT);
		assert_eq!(history.layers.len(), 1);
		let graph = history.final_graph();
		assert_eq!(graph.nodes.len(), 1);
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(graph.nodes.values().all(|node| node.parent.is_none()));
		assert!(graph.constraints.is_empty());
		assert_eq!(validate_rsp_embedding(history.final_layer()), Ok(()));
	}

	#[test]
	fn skeletal_graph_subdivides_to_level_zero() {
		let mut dice = Dice::new(0);
		let history = generate_skeletal_rsp(&mut dice, 2, DEFAULT_LOOP_WEIGHT);
		assert_eq!(history.layers.len(), 3);
		let graph = history.final_graph();
		assert!(graph.nodes.values().all(|node| node.level == 0));
		assert!(!graph.nodes.is_empty());
		for constraint in &graph.constraints {
			assert!(constraint.angle.is_finite());
			assert!(graph.nodes.contains_key(&constraint.subject));
			assert!(graph.nodes.contains_key(&constraint.reference));
		}
		for (i, layer) in history.layers.iter().enumerate().skip(1) {
			let prev = &history.layers[i - 1].graph;
			for node in layer.graph.nodes.values() {
				let Some(parent) = node.parent else {
					panic!("non-root leaf ancestry missing at layer {i}");
				};
				assert!(
					prev.nodes.contains_key(&parent),
					"parent {parent:?} missing from previous layer"
				);
				assert!(!layer.graph.nodes.contains_key(&parent));
			}
		}
	}

	#[test]
	fn children_tile_parent_cell() {
		let mut dice = Dice::new(4);
		let history = generate_skeletal_rsp(&mut dice, 3, DEFAULT_LOOP_WEIGHT);
		for i in 1..history.layers.len() {
			let prev = &history.layers[i - 1].partition;
			let layer = &history.layers[i];
			let mut child_area: BTreeMap<RspNodeId, f64> = BTreeMap::new();
			for node in layer.graph.nodes.values() {
				let parent = node.parent.expect("every node below the seed has a parent");
				*child_area.entry(parent).or_default() +=
					polygon_area(&layer.partition.cells[&node.id]);
			}
			for (parent, area) in child_area {
				let parent_area = polygon_area(&prev.cells[&parent]);
				assert!((area - parent_area).abs() < 1e-12, "layer {i} parent {parent:?}");
			}
		}
	}

	#[test]
	fn binary_halves_keep_at_least_four_ninths() {
		for seed in 0..50 {
			let mut dice = Dice::new(seed);
			let mut next_id = 1;
			let parent = seed_cell();
			let split = materialize_split(&parent, RspSplitForm::Binary, &mut next_id, &mut dice);
			assert_eq!(split.children.len(), 2);
			assert_eq!(split.sibling_edges.len(), 1);
			for child in &split.children {
				// Grünbaum: any line through the centroid keeps ≥ 4/9 of the area.
				assert!(polygon_area(&child.cell) >= 4.0 / 9.0 - 1e-12);
			}
		}
	}

	#[test]
	fn ternary_open_and_closed_sibling_edge_counts() {
		let parent = seed_cell();
		let mut next_id = 1u32;
		let mut dice = Dice::new(3);
		let closed = materialize_split(&parent, RspSplitForm::TernaryClosed, &mut next_id, &mut dice);
		assert_eq!(closed.children.len(), 3);
		assert_eq!(closed.sibling_edges.len(), 3);

		let mut next_id = 1u32;
		let mut dice = Dice::new(3);
		let open = materialize_split(&parent, RspSplitForm::TernaryOpen, &mut next_id, &mut dice);
		assert_eq!(open.children.len(), 3);
		assert_eq!(open.sibling_edges.len(), 2);

		for edge in closed.sibling_edges.iter().chain(&open.sibling_edges) {
			let split = if closed.sibling_edges.iter().any(|e| e.doorway == edge.doorway) {
				&closed
			} else {
				&open
			};
			for id in [edge.subject, edge.reference] {
				let child = split.children.iter().find(|c| c.id == id).unwrap();
				assert!(child.cell.vertices.contains(&edge.doorway));
			}
		}
	}

	/// Two nodes side by side (west / east halves of the seed) joined through `(0, 0)`.
	fn two_cell_layer(level: i32) -> RspLayer {
		let west = RspNodeId(0);
		let east = RspNodeId(1);
		let p = RspPoint::new;
		let west_cell = RspCell {
			vertices: vec![p(-0.5, -0.5), p(0.0, -0.5), p(0.0, 0.0), p(0.0, 0.5), p(-0.5, 0.5)],
		};
		let east_cell = RspCell {
			vertices: vec![p(0.0, -0.5), p(0.5, -0.5), p(0.5, 0.5), p(0.0, 0.5), p(0.0, 0.0)],
		};
		RspLayer {
			graph: RspGraph {
				nodes: BTreeMap::from([
					(west, RspNode { id: west, level, parent: None }),
					(east, RspNode { id: east, level: 0, parent: None }),
				]),
				constraints: vec![RspConstraint {
					subject: east,
					angle: 0.0,
					reference: west,
				}],
			},
			partition: RspPartition {
				points: BTreeMap::from([
					(west, polygon_centroid(&west_cell)),
					(east, polygon_centroid(&east_cell)),
				]),
				cells: BTreeMap::from([(west, west_cell), (east, east_cell)]),
				doorways: vec![p(0.0, 0.0)],
			},
		}
	}

	#[test]
	fn split_inherits_connection_through_doorway_owner_only() {
		let layer = two_cell_layer(1);
		assert_eq!(validate_rsp_embedding(&layer), Ok(()));
		let east = RspNodeId(1);
		for seed in 0..20 {
			let mut dice = Dice::new(seed);
			let out = subdivide_rsp_pass(&layer, &mut dice, 1.0 / 3.0);
			assert_eq!(validate_rsp_embedding(&out), Ok(()), "seed {seed}");
			let inherited: Vec<(usize, &RspConstraint)> = out
				.graph
				.constraints
				.iter()
				.enumerate()
				.filter(|(_, c)| c.subject == east || c.reference == east)
				.collect();
			assert_eq!(inherited.len(), 1, "only one child inherits the edge");
			let (index, c) = inherited[0];
			assert_eq!(out.partition.doorways[index], RspPoint::new(0.0, 0.0));
			let child = c.reference;
			assert_eq!(out.graph.nodes[&child].parent, Some(RspNodeId(0)));
			assert!(out.partition.cells[&child]
				.vertices
				.contains(&RspPoint::new(0.0, 0.0)));
			let expect = angle_between_points(out.partition.points[&child], out.partition.points[&east]);
			assert_eq!(c.angle, expect);
		}
	}

	#[test]
	fn doorway_on_split_wall_goes_to_lowest_child() {
		// The west cell's centroid is (-0.25, 0); a binary split along y = 0 ends exactly
		// at the doorway (0, 0), so both children hold it and the lowest id owns it.
		let layer = two_cell_layer(1);
		let mut next_id = 2;
		let center = polygon_centroid(&layer.partition.cells[&RspNodeId(0)]);
		let rays = split_by_rays(
			&layer.partition.cells[&RspNodeId(0)],
			center,
			&[RspPoint::new(1.0, 0.0), RspPoint::new(-1.0, 0.0)],
			&[false, false],
		);
		let children = make_children(rays.cells, &mut next_id);
		assert!(children[0].cell.vertices.contains(&RspPoint::new(0.0, 0.0)));
		assert!(children[1].cell.vertices.contains(&RspPoint::new(0.0, 0.0)));
		let splits = BTreeMap::from([(
			RspNodeId(0),
			Split {
				children,
				sibling_edges: Vec::new(),
			},
		)]);
		assert_eq!(
			doorway_owner(RspNodeId(0), RspPoint::new(0.0, 0.0), &splits),
			RspNodeId(2)
		);
	}

	#[test]
	fn validator_rejects_crossing_edges() {
		// Valid layer, then move the east point across the shared wall so its edge
		// half overlaps the west cell's drawing.
		let mut layer = two_cell_layer(0);
		let west = RspNodeId(0);
		let east = RspNodeId(1);
		layer.partition.points.insert(east, RspPoint::new(-0.25, 0.1));
		assert!(validate_rsp_embedding(&layer).is_err());

		let mut layer = two_cell_layer(0);
		// Two edges between the same pair crossing at their doorways' drawings.
		layer.graph.constraints.push(RspConstraint {
			subject: east,
			angle: 0.0,
			reference: west,
		});
		layer.partition.doorways.push(RspPoint::new(0.0, 0.0));
		assert!(matches!(
			validate_rsp_embedding(&layer),
			Err(RspEmbeddingError::SharedDoorway(0, 1))
		));
		assert!(count_edge_conflicts(&layer) > 0);
	}

	#[test]
	fn validator_rejects_overlapping_cells() {
		let mut layer = two_cell_layer(0);
		let east = RspNodeId(1);
		let p = RspPoint::new;
		layer.partition.cells.insert(
			east,
			RspCell {
				vertices: vec![p(-0.1, -0.5), p(0.5, -0.5), p(0.5, 0.5), p(-0.1, 0.5)],
			},
		);
		assert!(validate_rsp_embedding(&layer).is_err());
	}

	#[test]
	fn property_every_layer_is_overlap_free() {
		for seed in 0..100 {
			for subdivisions in 1..=5 {
				for loop_weight in [0.0, 1.0 / 3.0, DEFAULT_LOOP_WEIGHT, 1.0] {
					let mut dice = Dice::new(seed);
					let history = generate_skeletal_rsp(&mut dice, subdivisions, loop_weight);
					for (i, layer) in history.layers.iter().enumerate() {
						assert_eq!(
							validate_rsp_embedding(layer),
							Ok(()),
							"seed {seed} subdivisions {subdivisions} loop {loop_weight} layer {i}"
						);
					}
				}
			}
		}
	}

	#[test]
	fn property_metric_layout_is_overlap_free() {
		let config = RspLayoutConfig::default();
		for seed in 0..12 {
			for subdivisions in 1..=5 {
				for loop_weight in [0.0, DEFAULT_LOOP_WEIGHT, 1.0] {
					let mut dice = Dice::new(seed);
					let history = generate_skeletal_rsp(&mut dice, subdivisions, loop_weight);
					let layout = layout_rsp_metric_2d(history.final_layer(), &config);
					let label = format!("seed {seed} subdivisions {subdivisions} loop {loop_weight}");
					assert_eq!(validate_rsp_embedding(&layout.layer), Ok(()), "{label}");
					assert_eq!(layout.stats.crossings, 0, "{label}");
					let tol = config.min_separation * 1e-4;
					assert!(
						layout.stats.min_pairwise_distance >= config.min_separation - tol,
						"{label}: {:?}",
						layout.stats
					);
					assert!(
						layout.stats.min_node_edge_distance >= 0.5 * config.min_separation - tol,
						"{label}: {:?}",
						layout.stats
					);
					assert!(layout.stats.max_angle_residual.is_finite(), "{label}");
				}
			}
		}
	}

	#[test]
	fn metric_layout_scales_to_preferred_length() {
		let mut dice = Dice::new(0);
		let history = generate_skeletal_rsp(&mut dice, 3, DEFAULT_LOOP_WEIGHT);
		let config = RspLayoutConfig {
			max_iterations: 0,
			min_separation: 0.0,
			..RspLayoutConfig::default()
		};
		let layout = layout_rsp_metric_2d(history.final_layer(), &config);
		assert!((layout.stats.mean_edge_length - config.preferred_edge_length).abs() < 1e-2);
		assert_eq!(layout.stats.max_angle_residual, 0.0);
	}

	#[test]
	fn angle_helpers_round_trip() {
		let toward = angle_unit(opposite_angle(0.0));
		assert!((toward.x + 1.0).abs() < 1e-5);
		let north = angle_between_points(RspPoint::new(0.0, 0.0), RspPoint::new(0.0, 2.0));
		assert!((north - FRAC_PI_2).abs() < 1e-6);
	}
}
