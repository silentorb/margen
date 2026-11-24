//! Metric layout: relax node points inside their cells.
//!
//! The partition is scaled uniformly to the preferred edge length, then node
//! points are relaxed with soft angle bands, length springs and soft repulsion.
//! After every step each point is clamped into its cell shrunk toward the cell
//! centroid, so points stay strictly inside their cells and the edge drawings
//! (`subject → doorway → reference`) stay overlap-free.

use margen_rsp::{length_point, RspCell, RspLayer, RspNodeId, RspPartition, RspPoint};
use std::collections::BTreeMap;
use std::f64::consts::{PI, TAU};

use crate::partition::{clamp_into_cell, polygon_centroid, scale_cell, shrink_toward};
use crate::validate::{count_edge_conflicts, measure_clearance};

/// Parameters for [`layout_rsp_metric_2d`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RspLayoutConfig {
	pub preferred_edge_length: f32,
	/// Hard minimum distance between node points, and twice the hard minimum distance
	/// from a node point to any edge drawing it is not part of. Enforced after relaxation
	/// by uniform scaling (which preserves every partition invariant); also used as the
	/// soft repulsion radius while relaxing.
	pub min_separation: f32,
	pub angle_tolerance: f32,
	pub max_iterations: u32,
}

impl Default for RspLayoutConfig {
	fn default() -> Self {
		let preferred_edge_length = 120.0;
		Self {
			preferred_edge_length,
			min_separation: preferred_edge_length * 0.5,
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
	/// Smallest distance from a node point to an edge drawing it is not part of
	/// (`INFINITY` when no such pair exists; likewise `min_pairwise_distance` for one node).
	pub min_node_edge_distance: f32,
	/// Uniform scale applied after relaxation to meet `min_separation` (`1` when already met).
	pub clearance_scale: f32,
	/// Conflicting edge / node pairs found by the validator (always 0 by construction).
	pub crossings: u32,
}

/// Metric realization: the scaled partition with relaxed node points.
#[derive(Clone, Debug, PartialEq)]
pub struct RspLayout {
	pub layer: RspLayer,
	pub stats: RspLayoutStats,
}

/// Fraction of each cell (about its centroid) that node points may occupy.
pub const CELL_CLAMP_SCALE: f64 = 0.8;

/// Pipeline layout: scale the partition, then relax points inside their cells.
pub fn layout_rsp_metric_2d(layer: &RspLayer, config: &RspLayoutConfig) -> RspLayout {
	let preferred = f64::from(config.preferred_edge_length.max(1e-3));
	let min_sep = f64::from(config.min_separation.max(0.0));
	let angle_tol = f64::from(config.angle_tolerance.max(0.0));

	let mut layer = RspLayer {
		graph: layer.graph.clone(),
		partition: scale_partition(&layer.partition, scale_factor(layer, preferred)),
	};

	let mut centers: BTreeMap<RspNodeId, RspPoint> = BTreeMap::new();
	let mut bounds: BTreeMap<RspNodeId, RspCell> = BTreeMap::new();
	for (id, cell) in &layer.partition.cells {
		let center = polygon_centroid(cell);
		centers.insert(*id, center);
		bounds.insert(*id, shrink_toward(cell, center, CELL_CLAMP_SCALE));
	}

	const ANGLE_STRENGTH: f64 = 0.35;
	const LENGTH_STRENGTH: f64 = 0.08;
	const SEP_STRENGTH: f64 = 0.55;
	const STEP: f64 = 0.45;
	const DAMP: f64 = 0.85;

	let ids: Vec<RspNodeId> = layer.graph.nodes.keys().copied().collect();
	let mut positions = layer.partition.points.clone();
	let mut velocities: BTreeMap<RspNodeId, RspPoint> =
		ids.iter().map(|id| (*id, RspPoint::default())).collect();

	let mut iterations = 0u32;
	for iter in 0..config.max_iterations {
		iterations = iter + 1;
		let mut forces: BTreeMap<RspNodeId, RspPoint> =
			ids.iter().map(|id| (*id, RspPoint::default())).collect();

		for constraint in &layer.graph.constraints {
			let delta = positions[&constraint.subject] - positions[&constraint.reference];
			let dist = length_point(delta);
			if dist < 1e-12 {
				continue;
			}
			let radial = delta * (1.0 / dist);
			let tangent = RspPoint::new(-radial.y, radial.x);

			let err = wrap_angle_to_pi(delta.y.atan2(delta.x) - f64::from(constraint.angle));
			if err.abs() > angle_tol {
				let excess = (err.abs() - angle_tol) * err.signum();
				let force = tangent * (excess * ANGLE_STRENGTH * preferred);
				add_force(&mut forces, constraint.subject, force * -1.0);
				add_force(&mut forces, constraint.reference, force);
			}

			let length_force = radial * ((dist - preferred) * LENGTH_STRENGTH);
			add_force(&mut forces, constraint.subject, length_force * -1.0);
			add_force(&mut forces, constraint.reference, length_force);
		}

		if min_sep > 0.0 {
			for i in 0..ids.len() {
				for j in (i + 1)..ids.len() {
					let delta = positions[&ids[j]] - positions[&ids[i]];
					let dist = length_point(delta);
					if dist > 1e-12 && dist < min_sep {
						let force = delta * ((min_sep - dist) * SEP_STRENGTH / dist);
						add_force(&mut forces, ids[i], force * -1.0);
						add_force(&mut forces, ids[j], force);
					}
				}
			}
		}

		for id in &ids {
			let old = positions[id];
			let vel = velocities[id] * DAMP + forces[id] * STEP;
			let moved = clamp_into_cell(old + vel, centers[id], &bounds[id]);
			velocities.insert(*id, moved - old);
			positions.insert(*id, moved);
		}
	}

	layer.partition.points = positions;

	// Hard clearance: scale up uniformly until nodes are `min_sep` apart and no node is
	// within `min_sep / 2` of a foreign edge drawing. Scaling preserves I1–I3, so the
	// result stays overlap-free while gaining a guaranteed visual margin.
	let clearance = measure_clearance(&layer);
	let mut clearance_scale = 1.0f64;
	if min_sep > 0.0 {
		if clearance.node_node > 0.0 && clearance.node_node.is_finite() {
			clearance_scale = clearance_scale.max(min_sep / clearance.node_node);
		}
		if clearance.node_edge > 0.0 && clearance.node_edge.is_finite() {
			clearance_scale = clearance_scale.max(0.5 * min_sep / clearance.node_edge);
		}
	}
	if clearance_scale > 1.0 {
		layer.partition = scale_partition(&layer.partition, clearance_scale);
	}

	let stats = compute_layout_stats(&layer, angle_tol, iterations, clearance_scale);
	RspLayout { layer, stats }
}

/// Uniform scale that makes the mean straight edge length `preferred`
/// (seed side `preferred` when there are no edges).
fn scale_factor(layer: &RspLayer, preferred: f64) -> f64 {
	let points = &layer.partition.points;
	let lengths: Vec<f64> = layer
		.graph
		.constraints
		.iter()
		.filter_map(|c| Some(length_point(*points.get(&c.subject)? - *points.get(&c.reference)?)))
		.collect();
	let mean = lengths.iter().sum::<f64>() / lengths.len().max(1) as f64;
	if mean > 0.0 {
		preferred / mean
	} else {
		preferred
	}
}

fn scale_partition(partition: &RspPartition, factor: f64) -> RspPartition {
	RspPartition {
		cells: partition
			.cells
			.iter()
			.map(|(id, cell)| (*id, scale_cell(cell, factor)))
			.collect(),
		points: partition
			.points
			.iter()
			.map(|(id, p)| (*id, *p * factor))
			.collect(),
		doorways: partition.doorways.iter().map(|d| *d * factor).collect(),
	}
}

fn add_force(forces: &mut BTreeMap<RspNodeId, RspPoint>, id: RspNodeId, delta: RspPoint) {
	let entry = forces.entry(id).or_default();
	*entry = *entry + delta;
}

fn wrap_angle_to_pi(angle: f64) -> f64 {
	let mut a = angle % TAU;
	if a > PI {
		a -= TAU;
	} else if a < -PI {
		a += TAU;
	}
	a
}

fn compute_layout_stats(
	layer: &RspLayer,
	angle_tol: f64,
	iterations: u32,
	clearance_scale: f64,
) -> RspLayoutStats {
	let clearance = measure_clearance(layer);

	let mut max_residual = 0.0f64;
	let mut length_sum = 0.0f64;
	let mut length_count = 0u32;
	for constraint in &layer.graph.constraints {
		let delta = layer.partition.points[&constraint.subject]
			- layer.partition.points[&constraint.reference];
		let dist = length_point(delta);
		if dist > 1e-12 {
			length_sum += dist;
			length_count += 1;
			let err = wrap_angle_to_pi(delta.y.atan2(delta.x) - f64::from(constraint.angle)).abs();
			max_residual = max_residual.max((err - angle_tol).max(0.0));
		}
	}
	let mean_edge_length = if length_count > 0 {
		length_sum / f64::from(length_count)
	} else {
		0.0
	};

	RspLayoutStats {
		iterations,
		min_pairwise_distance: clearance.node_node as f32,
		max_angle_residual: max_residual as f32,
		mean_edge_length: mean_edge_length as f32,
		min_node_edge_distance: clearance.node_edge as f32,
		clearance_scale: clearance_scale as f32,
		crossings: count_edge_conflicts(layer),
	}
}
