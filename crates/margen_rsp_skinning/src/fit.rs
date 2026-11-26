//! Fit room and pathway footprints into a scaled RSP partition with margins.
//!
//! At a fixed scale the construction is deterministic and either yields every footprint
//! or names the first margin requirement it cannot meet ([`RspSkinBlocker`]). Every
//! requirement is eventually met as the scale grows (`docs/rsp-skinning.md`), so
//! [`fit_floor_plan`] doubles the scale until one fits, then bisects toward the smallest
//! fitting scale. Only scales whose construction succeeded are ever returned.

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use margen_rsp::{cross_point, dot_point, length_point, RspCell, RspLayer, RspNodeId, RspPoint};

use crate::walls::{
	insert_boundary_vertex, shared_wall, slot_interval, snap_to_vertex, wall_param, wall_point,
};
use crate::{
	RspFloorPiece, RspFloorPlan, RspPieceKind, RspRoom, RspSkinBlocker, RspSkinConfig,
	RspSkinError, MAX_SKIN_SCALE,
};

/// Bisection steps after the doubling search has bracketed the smallest fitting scale.
const BISECTION_STEPS: u32 = 8;

/// Relative tolerance (fraction of the layout diagonal) for on-wall tests.
const RELATIVE_EPS: f64 = 1e-9;

/// Smallest scale `s ≥ 1` (up to bisection precision) at which every footprint fits.
///
/// Node points are first moved to the centres of their cells' largest inscribed circles
/// ([`center_rooms`]), which gives every room the most space its cell allows.
pub fn fit_floor_plan(
	layer: &RspLayer,
	config: &RspSkinConfig,
) -> Result<RspFloorPlan, RspSkinError> {
	let centered = center_rooms(layer)?;
	let layer = &centered;
	let mut blocker = match fit_floor_plan_at_scale(layer, config, 1.0)? {
		Ok(plan) => return Ok(plan),
		Err(blocker) => blocker,
	};
	let mut low = 1.0;
	let mut high = 2.0;
	let mut best = loop {
		if high > MAX_SKIN_SCALE {
			return Err(RspSkinError::ScaleLimit { blocker });
		}
		match fit_floor_plan_at_scale(layer, config, high)? {
			Ok(plan) => break plan,
			Err(next) => {
				blocker = next;
				low = high;
				high *= 2.0;
			}
		}
	};
	for _ in 0..BISECTION_STEPS {
		let mid = 0.5 * (low + high);
		match fit_floor_plan_at_scale(layer, config, mid)? {
			Ok(plan) => {
				best = plan;
				high = mid;
			}
			Err(_) => low = mid,
		}
	}
	Ok(best)
}

/// Move every node point to the centre of the largest circle inscribed in its cell.
///
/// That centre is strictly inside the convex cell (its radius is positive), so the
/// partition invariants still hold and the edge drawings stay overlap-free.
pub fn center_rooms(layer: &RspLayer) -> Result<RspLayer, RspSkinError> {
	let mut centered = layer.clone();
	for (id, cell) in &layer.partition.cells {
		let point = centered
			.partition
			.points
			.get_mut(id)
			.ok_or(RspSkinError::MissingGeometry(*id))?;
		if let Some((center, _)) = inscribed_circle(cell) {
			*point = center;
		}
	}
	Ok(centered)
}

/// Largest circle inside a convex counter-clockwise polygon: `(centre, radius)`.
///
/// This is the linear program "maximise r subject to every edge line being at least r
/// away". It is bounded, so some optimum has three linearly independent active
/// constraints; every circle tangent to three edge lines is tried and the largest
/// feasible one kept. `None` only for a degenerate polygon.
pub fn inscribed_circle(cell: &RspCell) -> Option<(RspPoint, f64)> {
	// Edge lines as (unit inward normal n, offset c): distance of p is n·p − c.
	let mut lines: Vec<(RspPoint, f64)> = Vec::new();
	let n = cell.vertices.len();
	for i in 0..n {
		let a = cell.vertices[i];
		let b = cell.vertices[(i + 1) % n];
		let len = length_point(b - a);
		if len == 0.0 {
			continue;
		}
		let normal = RspPoint::new(-(b.y - a.y) / len, (b.x - a.x) / len);
		let line = (normal, dot_point(normal, a));
		// Collinear runs (doorway vertices) repeat the same line.
		let duplicate = lines.iter().any(|(m, c)| {
			cross_point(*m, normal).abs() <= 1e-12
				&& dot_point(*m, normal) > 0.0
				&& (c - line.1).abs() <= 1e-9 * len.max(1.0)
		});
		if !duplicate {
			lines.push(line);
		}
	}
	let distance = |p: RspPoint| {
		lines
			.iter()
			.map(|(m, c)| dot_point(*m, p) - c)
			.fold(f64::INFINITY, f64::min)
	};
	let mut best: Option<(RspPoint, f64)> = None;
	for i in 0..lines.len() {
		for j in (i + 1)..lines.len() {
			for k in (j + 1)..lines.len() {
				// Solve m·p − r = c for the three lines.
				let rows = [lines[i], lines[j], lines[k]];
				let Some((p, r)) = solve_tangent(rows) else {
					continue;
				};
				if r <= 0.0 {
					continue;
				}
				// Feasible when no other line is closer than r (relative slack for rounding).
				let actual = distance(p);
				if actual < r * (1.0 - 1e-9) {
					continue;
				}
				if best.map_or(true, |(_, radius)| actual > radius) {
					best = Some((p, actual));
				}
			}
		}
	}
	best
}

fn solve_tangent(rows: [(RspPoint, f64); 3]) -> Option<(RspPoint, f64)> {
	// [mx my -1] [x y r]^T = c, by Cramer's rule.
	let det3 = |m: [[f64; 3]; 3]| {
		m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
			- m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
			+ m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
	};
	let a = rows.map(|(m, _)| [m.x, m.y, -1.0]);
	let c = rows.map(|(_, c)| c);
	let det = det3(a);
	if det.abs() <= 1e-12 {
		return None;
	}
	let column = |col: usize| {
		let mut m = a;
		for row in 0..3 {
			m[row][col] = c[row];
		}
		det3(m) / det
	};
	Some((RspPoint::new(column(0), column(1)), column(2)))
}

/// One door: the middle `pathway_width` of a constraint's slot on its shared wall.
#[derive(Clone, Copy)]
struct Door {
	center: RspPoint,
	ends: [RspPoint; 2],
}

/// A pathway leaving a room, seen from the room centre.
struct Exit {
	constraint: usize,
	door: Door,
	/// Direction of the door centre.
	theta: f64,
	/// Angular extent of the door (`lo ≤ theta ≤ hi`, unwrapped around `theta`).
	lo: f64,
	hi: f64,
}

/// Build footprints at `scale`. The outer error is scale-independent; the inner one names
/// the margin that fails at this scale.
pub fn fit_floor_plan_at_scale(
	layer: &RspLayer,
	config: &RspSkinConfig,
	scale: f64,
) -> Result<Result<RspFloorPlan, RspSkinBlocker>, RspSkinError> {
	for id in layer.graph.nodes.keys() {
		if !layer.partition.cells.contains_key(id) || !layer.partition.points.contains_key(id) {
			return Err(RspSkinError::MissingGeometry(*id));
		}
	}
	let mut scaled = scale_layer(layer, scale);
	let eps = tolerance(&scaled);
	let width = f64::from(config.pathway_width);
	let gap = f64::from(config.gap);

	let doors = match place_doors(&scaled, width, gap)? {
		Ok(doors) => doors,
		Err(blocker) => return Ok(Err(blocker)),
	};
	let mut doors = doors;
	for (index, door) in doors.iter_mut().enumerate() {
		let constraint = scaled.graph.constraints[index];
		let cells = &scaled.partition.cells;
		door.center = snap_to_vertex(
			[&cells[&constraint.subject], &cells[&constraint.reference]],
			door.center,
			eps,
		);
		for node in [constraint.subject, constraint.reference] {
			let cell = scaled
				.partition
				.cells
				.get_mut(&node)
				.expect("cell presence checked above");
			if !insert_boundary_vertex(cell, door.center, eps) {
				return Err(RspSkinError::NoSharedWall { constraint: index });
			}
		}
		scaled.partition.doorways[index] = door.center;
	}

	let mut exits: BTreeMap<RspNodeId, Vec<Exit>> =
		scaled.graph.nodes.keys().map(|id| (*id, Vec::new())).collect();
	for (index, constraint) in scaled.graph.constraints.iter().enumerate() {
		for node in [constraint.subject, constraint.reference] {
			let center = scaled.partition.points[&node];
			exits
				.get_mut(&node)
				.expect("every constraint endpoint is a node")
				.push(exit_from(center, index, doors[index]));
		}
	}

	let mut rooms = Vec::new();
	let mut pieces = Vec::new();
	for (node, mut node_exits) in exits {
		let center = scaled.partition.points[&node];
		let cell = &scaled.partition.cells[&node];
		node_exits.sort_by(|a, b| a.theta.total_cmp(&b.theta).then(a.constraint.cmp(&b.constraint)));
		match fit_room(node, center, cell, &node_exits, config) {
			Ok((room, room_piece, pathway_pieces)) => {
				rooms.push(room);
				pieces.push(room_piece);
				pieces.extend(pathway_pieces);
			}
			Err(blocker) => return Ok(Err(blocker)),
		}
	}

	Ok(Ok(RspFloorPlan {
		layer: scaled,
		scale,
		rooms,
		pieces,
	}))
}

/// Door for every constraint: shared walls split into equal slots per cell pair.
fn place_doors(
	layer: &RspLayer,
	width: f64,
	gap: f64,
) -> Result<Result<Vec<Door>, RspSkinBlocker>, RspSkinError> {
	let constraints = &layer.graph.constraints;
	let mut pairs: BTreeMap<(RspNodeId, RspNodeId), Vec<usize>> = BTreeMap::new();
	for (index, c) in constraints.iter().enumerate() {
		let key = (c.subject.min(c.reference), c.subject.max(c.reference));
		pairs.entry(key).or_default().push(index);
	}

	let mut doors: Vec<Option<Door>> = vec![None; constraints.len()];
	for ((a, b), mut members) in pairs {
		let first = members[0];
		let wall = shared_wall(
			&layer.partition.cells[&a],
			&layer.partition.cells[&b],
			layer.partition.doorways[first],
		)
		.ok_or(RspSkinError::NoSharedWall { constraint: first })?;
		members.sort_by(|x, y| {
			let tx = wall_param(&wall, layer.partition.doorways[*x]);
			let ty = wall_param(&wall, layer.partition.doorways[*y]);
			tx.total_cmp(&ty).then(x.cmp(y))
		});
		for (rank, index) in members.iter().enumerate() {
			let (start, end) = slot_interval(&wall, rank, members.len());
			if end - start < width + 2.0 * gap {
				return Ok(Err(RspSkinBlocker::SlotTooShort { constraint: *index }));
			}
			let mid = 0.5 * (start + end);
			doors[*index] = Some(Door {
				center: wall_point(&wall, mid),
				ends: [
					wall_point(&wall, mid - 0.5 * width),
					wall_point(&wall, mid + 0.5 * width),
				],
			});
		}
	}
	Ok(Ok(doors
		.into_iter()
		.map(|door| door.expect("every constraint belongs to a pair"))
		.collect()))
}

fn exit_from(center: RspPoint, constraint: usize, door: Door) -> Exit {
	let theta = direction_angle(door.center - center);
	let offsets = door
		.ends
		.map(|end| wrap_to_pi(direction_angle(end - center) - theta));
	Exit {
		constraint,
		door,
		theta,
		lo: theta + offsets[0].min(offsets[1]),
		hi: theta + offsets[0].max(offsets[1]),
	}
}

/// Room circle, room polygon and the half-pathways leaving it, or the failing margin.
fn fit_room(
	node: RspNodeId,
	center: RspPoint,
	cell: &RspCell,
	exits: &[Exit],
	config: &RspSkinConfig,
) -> Result<(RspRoom, RspFloorPiece, Vec<RspFloorPiece>), RspSkinBlocker> {
	let width = f64::from(config.pathway_width);
	let gap = f64::from(config.gap);
	let angular_gap = f64::from(config.angular_gap);
	let step = TAU / f64::from(config.target_sides.max(3));

	let sectors = exit_sectors(exits, angular_gap).ok_or(RspSkinBlocker::DoorsTooClose { node })?;

	// Smallest radius whose side of width `w` fits every sector.
	let half_angle_cap = sectors
		.iter()
		.map(|(lo, hi)| 0.5 * (hi - lo))
		.fold(FRAC_PI_2, f64::min);
	let required = f64::from(config.room_min_radius).max(0.5 * width / half_angle_cap.sin());
	let available = boundary_distance(cell, center) - gap;
	let radius = required.max(f64::from(config.room_preferred_radius).min(available));
	if radius > available {
		return Err(RspSkinBlocker::RoomTooLarge { node });
	}
	let half_angle = (0.5 * width / radius).asin();

	let mut faces = Vec::with_capacity(exits.len());
	for (exit, (lo, hi)) in exits.iter().zip(&sectors) {
		let phi = if lo + half_angle <= hi - half_angle {
			exit.theta.clamp(lo + half_angle, hi - half_angle)
		} else {
			0.5 * (lo + hi)
		};
		let normal = unit(phi);
		let apothem = radius * half_angle.cos();
		for end in exit.door.ends {
			if dot_point(end - center, normal) < apothem + gap {
				return Err(RspSkinBlocker::DoorBehindFace {
					node,
					constraint: exit.constraint,
				});
			}
		}
		let wedge = (phi + half_angle).max(exit.hi) - (phi - half_angle).min(exit.lo);
		if wedge >= PI {
			return Err(RspSkinBlocker::WedgeTooWide {
				node,
				constraint: exit.constraint,
			});
		}
		faces.push(phi);
	}

	let room = RspRoom {
		node,
		center,
		radius,
	};
	let (room_piece, face_points) = room_polygon(&room, &faces, half_angle, step);
	let mut pathways = Vec::with_capacity(exits.len());
	for (exit, [minus, plus]) in exits.iter().zip(face_points) {
		let polygon = pathway_quad(plus, minus, exit.door.ends).ok_or(
			RspSkinBlocker::DoorBehindFace {
				node,
				constraint: exit.constraint,
			},
		)?;
		pathways.push(RspFloorPiece {
			kind: RspPieceKind::Pathway {
				constraint: exit.constraint,
				node,
			},
			polygon,
			joins: vec![0, 2],
		});
	}
	Ok((room, room_piece, pathways))
}

/// Angular sector `(lo, hi)` reserved for each exit (sorted by `theta`): the circle is cut
/// at the middle of every gap between consecutive doors, and `angular_gap / 2` is kept
/// free on each side of a cut. `None` when two doors are closer than `angular_gap`.
fn exit_sectors(exits: &[Exit], angular_gap: f64) -> Option<Vec<(f64, f64)>> {
	let n = exits.len();
	if n == 1 {
		let theta = exits[0].theta;
		return Some(vec![(theta - PI + 0.5 * angular_gap, theta + PI - 0.5 * angular_gap)]);
	}
	// cuts[i] lies between exit i and exit i + 1 (the last one wraps past 2π).
	let mut cuts = Vec::with_capacity(n);
	for i in 0..n {
		let start = exits[i].hi;
		let end = if i + 1 < n {
			exits[i + 1].lo
		} else {
			exits[0].lo + TAU
		};
		if end - start < angular_gap {
			return None;
		}
		cuts.push(0.5 * (start + end));
	}
	Some(
		(0..n)
			.map(|i| {
				let before = if i == 0 { cuts[n - 1] - TAU } else { cuts[i - 1] };
				(before + 0.5 * angular_gap, cuts[i] - 0.5 * angular_gap)
			})
			.collect(),
	)
}

/// Room outline on its circle: every face is the chord `phi ± half_angle` (width exactly
/// `pathway_width`, perpendicular to `phi`), and arc points fill the gaps at most `step`
/// apart. Returns the piece and each face's `[minus, plus]` endpoints.
fn room_polygon(
	room: &RspRoom,
	faces: &[f64],
	half_angle: f64,
	step: f64,
) -> (RspFloorPiece, Vec<[RspPoint; 2]>) {
	let on_circle = |angle: f64| room.center + unit(angle) * room.radius;
	let mut polygon = Vec::new();
	let mut joins = Vec::with_capacity(faces.len());
	let mut face_points = Vec::with_capacity(faces.len());
	if faces.is_empty() {
		let sides = (TAU / step).round() as usize;
		polygon.extend((0..sides).map(|k| on_circle(TAU * k as f64 / sides as f64)));
	}
	for (i, phi) in faces.iter().enumerate() {
		let minus = on_circle(phi - half_angle);
		let plus = on_circle(phi + half_angle);
		joins.push(polygon.len());
		polygon.push(minus);
		polygon.push(plus);
		face_points.push([minus, plus]);

		let gap_start = phi + half_angle;
		let gap_end = if i + 1 < faces.len() {
			faces[i + 1] - half_angle
		} else {
			faces[0] - half_angle + TAU
		};
		let span = gap_end - gap_start;
		let fill = ((span / step).ceil() as usize).saturating_sub(1);
		for k in 1..=fill {
			polygon.push(on_circle(gap_start + span * k as f64 / (fill + 1) as f64));
		}
	}
	(
		RspFloorPiece {
			kind: RspPieceKind::Room(room.node),
			polygon,
			joins,
		},
		face_points,
	)
}

/// Convex quad `plus → minus → door end → door end` (counter-clockwise), or `None` if the
/// four points are not in convex position.
fn pathway_quad(plus: RspPoint, minus: RspPoint, ends: [RspPoint; 2]) -> Option<Vec<RspPoint>> {
	for [first, second] in [[ends[0], ends[1]], [ends[1], ends[0]]] {
		let quad = vec![plus, minus, first, second];
		let convex = (0..4).all(|i| {
			let a = quad[i];
			let b = quad[(i + 1) % 4];
			let c = quad[(i + 2) % 4];
			cross_point(b - a, c - b) > 0.0
		});
		if convex {
			return Some(quad);
		}
	}
	None
}

/// Distance from an interior point to the nearest edge line of a convex cell.
fn boundary_distance(cell: &RspCell, p: RspPoint) -> f64 {
	let n = cell.vertices.len();
	let mut best = f64::INFINITY;
	for i in 0..n {
		let a = cell.vertices[i];
		let b = cell.vertices[(i + 1) % n];
		let edge = b - a;
		let len = length_point(edge);
		if len > 0.0 {
			best = best.min(cross_point(edge, p - a) / len);
		}
	}
	best
}

fn scale_layer(layer: &RspLayer, factor: f64) -> RspLayer {
	let mut scaled = layer.clone();
	for cell in scaled.partition.cells.values_mut() {
		for v in &mut cell.vertices {
			*v = *v * factor;
		}
	}
	for p in scaled.partition.points.values_mut() {
		*p = *p * factor;
	}
	for d in &mut scaled.partition.doorways {
		*d = *d * factor;
	}
	scaled
}

fn tolerance(layer: &RspLayer) -> f64 {
	let mut min = RspPoint::new(f64::INFINITY, f64::INFINITY);
	let mut max = RspPoint::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
	for v in layer.partition.cells.values().flat_map(|c| &c.vertices) {
		min = RspPoint::new(min.x.min(v.x), min.y.min(v.y));
		max = RspPoint::new(max.x.max(v.x), max.y.max(v.y));
	}
	RELATIVE_EPS * length_point(max - min).max(1.0)
}

fn unit(angle: f64) -> RspPoint {
	RspPoint::new(angle.cos(), angle.sin())
}

fn direction_angle(v: RspPoint) -> f64 {
	v.y.atan2(v.x)
}

fn wrap_to_pi(angle: f64) -> f64 {
	let mut a = angle % TAU;
	if a > PI {
		a -= TAU;
	} else if a < -PI {
		a += TAU;
	}
	a
}

#[cfg(test)]
mod tests {
	use super::*;

	fn exit_at(theta: f64, half: f64) -> Exit {
		Exit {
			constraint: 0,
			door: Door {
				center: unit(theta),
				ends: [unit(theta - half), unit(theta + half)],
			},
			theta,
			lo: theta - half,
			hi: theta + half,
		}
	}

	#[test]
	fn inscribed_circle_of_right_triangle_and_rectangle() {
		let triangle = RspCell {
			vertices: vec![RspPoint::new(0.0, 0.0), RspPoint::new(4.0, 0.0), RspPoint::new(0.0, 3.0)],
		};
		let (center, radius) = inscribed_circle(&triangle).expect("circle");
		assert!((radius - 1.0).abs() < 1e-12);
		assert!(length_point(center - RspPoint::new(1.0, 1.0)) < 1e-12);

		// Collinear extra vertex on the bottom edge must not change the answer.
		let rectangle = RspCell {
			vertices: vec![
				RspPoint::new(0.0, 0.0),
				RspPoint::new(1.5, 0.0),
				RspPoint::new(4.0, 0.0),
				RspPoint::new(4.0, 2.0),
				RspPoint::new(0.0, 2.0),
			],
		};
		let (center, radius) = inscribed_circle(&rectangle).expect("circle");
		assert!((radius - 1.0).abs() < 1e-12);
		assert!((center.y - 1.0).abs() < 1e-12 && center.x >= 1.0 && center.x <= 3.0);
	}

	#[test]
	fn sectors_cut_mid_gap_and_keep_the_angular_gap() {
		let exits = [exit_at(0.0, 0.1), exit_at(1.0, 0.1)];
		let sectors = exit_sectors(&exits, 0.02).expect("fits");
		assert!((sectors[0].1 - (0.5 - 0.01)).abs() < 1e-12);
		assert!((sectors[1].0 - (0.5 + 0.01)).abs() < 1e-12);
		// Wrap-around cut sits opposite, halfway through the long gap.
		let wrap_cut = 0.5 * (1.1 + (TAU - 0.1));
		assert!((sectors[1].1 - (wrap_cut - 0.01)).abs() < 1e-12);
		assert!((sectors[0].0 - (wrap_cut - TAU + 0.01)).abs() < 1e-12);
	}

	#[test]
	fn overlapping_doors_have_no_sectors() {
		let exits = [exit_at(0.0, 0.1), exit_at(0.15, 0.1)];
		assert!(exit_sectors(&exits, 0.02).is_none());
	}

	#[test]
	fn narrow_sector_forces_a_larger_room() {
		let config = RspSkinConfig::default();
		let cell = RspCell {
			vertices: vec![
				RspPoint::new(-1000.0, -1000.0),
				RspPoint::new(1000.0, -1000.0),
				RspPoint::new(1000.0, 1000.0),
				RspPoint::new(-1000.0, 1000.0),
			],
		};
		let center = RspPoint::default();
		let far = |theta: f64| {
			let door_center = unit(theta) * 900.0;
			let along = unit(theta + FRAC_PI_2) * 12.0;
			exit_from(
				center,
				0,
				Door {
					center: door_center,
					ends: [door_center - along, door_center + along],
				},
			)
		};
		let wide = [far(0.0), far(PI)];
		let (wide_room, _, _) = fit_room(RspNodeId(0), center, &cell, &wide, &config).expect("fits");
		// The middle exit's sector is squeezed by both neighbours.
		let narrow = [far(0.0), far(0.3), far(0.6)];
		let (narrow_room, _, _) =
			fit_room(RspNodeId(0), center, &cell, &narrow, &config).expect("fits");
		assert_eq!(wide_room.radius, f64::from(config.room_preferred_radius));
		assert!(narrow_room.radius > wide_room.radius);
	}

	#[test]
	fn faces_are_pathway_wide_and_perpendicular() {
		let room = RspRoom {
			node: RspNodeId(3),
			center: RspPoint::new(5.0, -2.0),
			radius: 20.0,
		};
		let width = 24.0;
		let half = (0.5 * width / room.radius).asin();
		let faces = [0.3, 2.0, 4.0];
		let (piece, points) = room_polygon(&room, &faces, half, TAU / 6.0);
		for (phi, [minus, plus]) in faces.iter().zip(&points) {
			let chord = *plus - *minus;
			assert!((length_point(chord) - width).abs() < 1e-9);
			assert!(dot_point(chord, unit(*phi)).abs() < 1e-9);
		}
		for join in &piece.joins {
			let a = piece.polygon[*join];
			assert!(points.iter().any(|[minus, _]| *minus == a));
		}
	}

	#[test]
	fn isolated_room_is_a_regular_polygon() {
		let room = RspRoom {
			node: RspNodeId(0),
			center: RspPoint::default(),
			radius: 10.0,
		};
		let (piece, _) = room_polygon(&room, &[], 0.0, TAU / 6.0);
		assert_eq!(piece.polygon.len(), 6);
		assert!(piece.joins.is_empty());
	}

	#[test]
	fn crowded_room_has_no_vertex_cap() {
		let room = RspRoom {
			node: RspNodeId(0),
			center: RspPoint::default(),
			radius: 100.0,
		};
		let faces: Vec<f64> = (0..7).map(|k| TAU * k as f64 / 7.0).collect();
		let (piece, _) = room_polygon(&room, &faces, (12.0f64 / 100.0).asin(), TAU / 6.0);
		assert!(piece.polygon.len() > 8);
		assert_eq!(piece.joins.len(), 7);
	}

	#[test]
	fn pathway_quad_orders_door_ends() {
		let plus = RspPoint::new(1.0, 1.0);
		let minus = RspPoint::new(1.0, -1.0);
		for ends in [
			[RspPoint::new(5.0, -1.0), RspPoint::new(5.0, 1.0)],
			[RspPoint::new(5.0, 1.0), RspPoint::new(5.0, -1.0)],
		] {
			let quad = pathway_quad(plus, minus, ends).expect("convex");
			assert_eq!(quad[2], RspPoint::new(5.0, -1.0));
			assert_eq!(quad[3], RspPoint::new(5.0, 1.0));
		}
		// Door collinear with the face: not a quad.
		let collinear = [RspPoint::new(1.0, 3.0), RspPoint::new(1.0, 5.0)];
		assert!(pathway_quad(plus, minus, collinear).is_none());
	}
}
