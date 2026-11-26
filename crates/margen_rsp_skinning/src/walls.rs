//! Shared walls between adjacent cells and the door slots cut into them.
//!
//! Two cells with disjoint interiors meet in a convex set, so their shared wall is a
//! single segment. Every constraint between the pair gets an equal-length slot on that
//! segment; its door is the middle `width` of the slot.

use margen_rsp::{cross_point, dot_point, length_point, RspCell, RspPoint};

/// A wall segment `origin + t·direction` for `t` in `[t_min, t_max]` (`direction` is unit).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {
	pub origin: RspPoint,
	pub direction: RspPoint,
	pub t_min: f64,
	pub t_max: f64,
}

pub fn wall_point(wall: &Wall, t: f64) -> RspPoint {
	wall.origin + wall.direction * t
}

pub fn wall_param(wall: &Wall, p: RspPoint) -> f64 {
	dot_point(p - wall.origin, wall.direction)
}

/// The wall shared by cells `a` and `b` through `doorway` (a vertex of both cells),
/// or `None` when the cells only touch at a point there.
pub fn shared_wall(a: &RspCell, b: &RspCell, doorway: RspPoint) -> Option<Wall> {
	let a_out = outgoing_directions(a, doorway)?;
	let b_out = outgoing_directions(b, doorway)?;
	let direction = a_out
		.iter()
		.find(|da| b_out.iter().any(|db| parallel_same_way(**da, *db)))
		.copied()?;
	let line = Wall {
		origin: doorway,
		direction,
		t_min: f64::NEG_INFINITY,
		t_max: f64::INFINITY,
	};
	let (a_min, a_max) = clip_line(a, &line)?;
	let (b_min, b_max) = clip_line(b, &line)?;
	let t_min = a_min.max(b_min);
	let t_max = a_max.min(b_max);
	if t_max <= t_min {
		return None;
	}
	Some(Wall {
		t_min,
		t_max,
		..line
	})
}

/// Unit directions of the boundary edges leaving `p`, a vertex of `cell` (collinear
/// duplicates skipped).
fn outgoing_directions(cell: &RspCell, p: RspPoint) -> Option<Vec<RspPoint>> {
	let n = cell.vertices.len();
	let index = cell.vertices.iter().position(|v| *v == p)?;
	let mut out = Vec::new();
	for step in [1, n - 1] {
		let mut k = (index + step) % n;
		while cell.vertices[k] == p && k != index {
			k = (k + step) % n;
		}
		let delta = cell.vertices[k] - p;
		let len = length_point(delta);
		if len > 0.0 {
			out.push(delta * (1.0 / len));
		}
	}
	Some(out)
}

fn parallel_same_way(a: RspPoint, b: RspPoint) -> bool {
	cross_point(a, b).abs() <= 1e-9 && dot_point(a, b) > 0.0
}

/// Parameter interval of `line` inside the convex counter-clockwise `cell`. Edges parallel
/// to the line are skipped: the line runs along one of them, or stays inside its half-plane.
fn clip_line(cell: &RspCell, line: &Wall) -> Option<(f64, f64)> {
	let n = cell.vertices.len();
	let mut t_min = f64::NEG_INFINITY;
	let mut t_max = f64::INFINITY;
	for i in 0..n {
		let a = cell.vertices[i];
		let b = cell.vertices[(i + 1) % n];
		let edge = b - a;
		let len = length_point(edge);
		if len == 0.0 {
			continue;
		}
		let unit = edge * (1.0 / len);
		let slope = cross_point(unit, line.direction);
		if slope.abs() <= 1e-9 {
			continue;
		}
		// Inside when cross(unit, origin + t·dir − a) >= 0.
		let t = -cross_point(unit, line.origin - a) / slope;
		if slope > 0.0 {
			t_min = t_min.max(t);
		} else {
			t_max = t_max.min(t);
		}
	}
	(t_max > t_min).then_some((t_min, t_max))
}

/// Door slot for constraint `rank` of `count` sharing `wall`: the slot's parameter interval.
pub fn slot_interval(wall: &Wall, rank: usize, count: usize) -> (f64, f64) {
	let length = (wall.t_max - wall.t_min) / count as f64;
	let start = wall.t_min + length * rank as f64;
	(start, start + length)
}

/// `p`, or the nearest vertex of either cell when one lies within `eps` of it. Inserting a
/// point that close to an existing vertex would create a near-zero-length edge.
pub fn snap_to_vertex(cells: [&RspCell; 2], p: RspPoint, eps: f64) -> RspPoint {
	cells
		.iter()
		.flat_map(|cell| cell.vertices.iter().copied())
		.map(|v| (length_point(v - p), v))
		.filter(|(distance, _)| *distance <= eps)
		.min_by(|a, b| a.0.total_cmp(&b.0))
		.map_or(p, |(_, v)| v)
}

/// Insert `p` (on the boundary of `cell`) as a vertex of the edge it lies on, unless it
/// already is a vertex. Returns `false` when no edge contains `p` within `eps`.
pub fn insert_boundary_vertex(cell: &mut RspCell, p: RspPoint, eps: f64) -> bool {
	if cell.vertices.contains(&p) {
		return true;
	}
	let n = cell.vertices.len();
	let mut best: Option<(usize, f64)> = None;
	for i in 0..n {
		let a = cell.vertices[i];
		let b = cell.vertices[(i + 1) % n];
		let edge = b - a;
		let len_sq = dot_point(edge, edge);
		if len_sq == 0.0 {
			continue;
		}
		let t = dot_point(p - a, edge) / len_sq;
		if t <= 0.0 || t >= 1.0 {
			continue;
		}
		let distance = cross_point(edge, p - a).abs() / len_sq.sqrt();
		if distance <= eps && best.map_or(true, |(_, d)| distance < d) {
			best = Some((i, distance));
		}
	}
	match best {
		Some((i, _)) => {
			cell.vertices.insert(i + 1, p);
			true
		}
		None => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> RspCell {
		RspCell {
			vertices: vec![
				RspPoint::new(x0, y0),
				RspPoint::new(x1, y0),
				RspPoint::new(x1, y1),
				RspPoint::new(x0, y1),
			],
		}
	}

	#[test]
	fn shared_wall_of_two_squares() {
		let mut a = square(0.0, 0.0, 1.0, 1.0);
		let mut b = square(1.0, 0.0, 2.0, 1.0);
		let door = RspPoint::new(1.0, 0.5);
		assert!(insert_boundary_vertex(&mut a, door, 1e-12));
		assert!(insert_boundary_vertex(&mut b, door, 1e-12));
		let wall = shared_wall(&a, &b, door).expect("wall");
		let ends = [wall_point(&wall, wall.t_min), wall_point(&wall, wall.t_max)];
		let mut ys = [ends[0].y, ends[1].y];
		ys.sort_by(f64::total_cmp);
		assert!((ys[0] - 0.0).abs() < 1e-12 && (ys[1] - 1.0).abs() < 1e-12);
		assert!(ends.iter().all(|p| (p.x - 1.0).abs() < 1e-12));
	}

	#[test]
	fn shared_wall_at_t_junction_is_the_shorter_side() {
		// `a` spans the whole wall x = 1; `b` covers only its lower half.
		let mut a = square(0.0, 0.0, 1.0, 2.0);
		let mut b = square(1.0, 0.0, 2.0, 1.0);
		let door = RspPoint::new(1.0, 0.25);
		assert!(insert_boundary_vertex(&mut a, door, 1e-12));
		assert!(insert_boundary_vertex(&mut b, door, 1e-12));
		// The T-junction corner of `b` is not a vertex of `a`.
		assert!(!a.vertices.contains(&RspPoint::new(1.0, 1.0)));
		let wall = shared_wall(&a, &b, door).expect("wall");
		assert!((wall.t_max - wall.t_min - 1.0).abs() < 1e-12);
	}

	#[test]
	fn corner_contact_has_no_wall() {
		let a = square(0.0, 0.0, 1.0, 1.0);
		let b = square(1.0, 1.0, 2.0, 2.0);
		assert_eq!(shared_wall(&a, &b, RspPoint::new(1.0, 1.0)), None);
	}

	#[test]
	fn slots_split_the_wall_evenly() {
		let wall = Wall {
			origin: RspPoint::new(0.0, 0.0),
			direction: RspPoint::new(1.0, 0.0),
			t_min: 2.0,
			t_max: 8.0,
		};
		assert_eq!(slot_interval(&wall, 0, 1), (2.0, 8.0));
		assert_eq!(slot_interval(&wall, 0, 2), (2.0, 5.0));
		assert_eq!(slot_interval(&wall, 1, 2), (5.0, 8.0));
	}
}
