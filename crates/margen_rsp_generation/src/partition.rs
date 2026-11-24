//! Convex-cell geometry for RSP partitioning.
//!
//! Every node owns a convex cell. A split divides the parent cell into wedges
//! cast from an interior center, so child cells are convex, have disjoint
//! interiors, and exactly tile the parent. Points are copied verbatim from the
//! parent boundary into children, so any doorway stored as a parent vertex is
//! a bit-identical vertex of the child (or children) that own it.

use margen_rsp::{cross_point, dot_point, RspCell, RspPoint};

/// Seed cell: unit square centered on the origin (counter-clockwise).
pub fn seed_cell() -> RspCell {
	RspCell {
		vertices: vec![
			RspPoint::new(-0.5, -0.5),
			RspPoint::new(0.5, -0.5),
			RspPoint::new(0.5, 0.5),
			RspPoint::new(-0.5, 0.5),
		],
	}
}

/// Signed area (counter-clockwise positive).
pub fn polygon_area(cell: &RspCell) -> f64 {
	let n = cell.vertices.len();
	let mut twice = 0.0;
	for i in 0..n {
		let a = cell.vertices[i];
		let b = cell.vertices[(i + 1) % n];
		twice += cross_point(a, b);
	}
	twice * 0.5
}

/// Area centroid. Strictly interior for a convex cell with positive area.
pub fn polygon_centroid(cell: &RspCell) -> RspPoint {
	let n = cell.vertices.len();
	// Shift by the first vertex to keep the products well conditioned.
	let origin = cell.vertices[0];
	let mut twice_area = 0.0;
	let mut cx = 0.0;
	let mut cy = 0.0;
	for i in 0..n {
		let a = cell.vertices[i] - origin;
		let b = cell.vertices[(i + 1) % n] - origin;
		let c = cross_point(a, b);
		twice_area += c;
		cx += (a.x + b.x) * c;
		cy += (a.y + b.y) * c;
	}
	assert!(
		twice_area > 0.0,
		"RSP cell must have positive counter-clockwise area"
	);
	origin + RspPoint::new(cx / (3.0 * twice_area), cy / (3.0 * twice_area))
}

/// Uniformly scale a cell about the origin.
pub fn scale_cell(cell: &RspCell, factor: f64) -> RspCell {
	RspCell {
		vertices: cell.vertices.iter().map(|v| *v * factor).collect(),
	}
}

/// Cell shrunk toward `center` by `factor` in `(0, 1)`: convex and strictly inside `cell`
/// when `center` is interior.
pub fn shrink_toward(cell: &RspCell, center: RspPoint, factor: f64) -> RspCell {
	RspCell {
		vertices: cell
			.vertices
			.iter()
			.map(|v| center + (*v - center) * factor)
			.collect(),
	}
}

/// Move `point` along the segment toward `center` until it lies inside the convex `shrunk`
/// cell (`center` must be strictly inside `shrunk`). Points already inside are unchanged.
pub fn clamp_into_cell(point: RspPoint, center: RspPoint, shrunk: &RspCell) -> RspPoint {
	let n = shrunk.vertices.len();
	let mut t = 1.0f64;
	for i in 0..n {
		let a = shrunk.vertices[i];
		let b = shrunk.vertices[(i + 1) % n];
		let edge = b - a;
		let f0 = cross_point(edge, center - a);
		let f1 = cross_point(edge, point - a);
		if f1 < 0.0 && f0 > 0.0 {
			t = t.min(f0 / (f0 - f1));
		}
	}
	center + (point - center) * t
}

/// Result of [`split_by_rays`].
pub struct RaySplit {
	/// Child `k` is the wedge from ray `k` to ray `k + 1` (counter-clockwise).
	pub cells: Vec<RspCell>,
	/// Doorway on wall `k` (segment from the center to ray `k`'s boundary hit), when requested.
	pub wall_doorways: Vec<Option<RspPoint>>,
}

/// Where a ray leaves the cell: on vertex `index` (`param == 0`) or inside edge
/// `index → index + 1` at `param` in `(0, 1)`.
struct RayHit {
	index: usize,
	param: f64,
	point: RspPoint,
}

/// Split a convex counter-clockwise `cell` into wedges cast from the interior `center`.
///
/// `directions` must be in counter-clockwise order with consecutive gaps of at most π,
/// so every wedge ∩ cell is convex. `wall_doorway[k]` places a doorway at the midpoint
/// of wall `k` and inserts it as a vertex of both cells sharing that wall.
pub fn split_by_rays(
	cell: &RspCell,
	center: RspPoint,
	directions: &[RspPoint],
	wall_doorway: &[bool],
) -> RaySplit {
	assert_eq!(directions.len(), wall_doorway.len());
	assert!(directions.len() >= 2);
	let hits: Vec<RayHit> = directions
		.iter()
		.map(|d| ray_exit(cell, center, *d))
		.collect();

	// Boundary walk: original vertices plus edge-interior hits, tagged with the ray that hits them.
	let n = cell.vertices.len();
	let mut walk: Vec<(RspPoint, Option<usize>)> = Vec::with_capacity(n + hits.len());
	for i in 0..n {
		let vertex_ray = hits
			.iter()
			.position(|hit| hit.index == i && hit.param == 0.0);
		walk.push((cell.vertices[i], vertex_ray));
		let mut on_edge: Vec<usize> = (0..hits.len())
			.filter(|k| hits[*k].index == i && hits[*k].param > 0.0)
			.collect();
		on_edge.sort_by(|a, b| hits[*a].param.total_cmp(&hits[*b].param));
		for k in on_edge {
			walk.push((hits[k].point, Some(k)));
		}
	}
	let walk_pos: Vec<usize> = (0..hits.len())
		.map(|k| {
			walk.iter()
				.position(|(_, ray)| *ray == Some(k))
				.expect("every ray hit is on the boundary walk")
		})
		.collect();

	let wall_doorways: Vec<Option<RspPoint>> = (0..hits.len())
		.map(|k| wall_doorway[k].then(|| (center + hits[k].point) * 0.5))
		.collect();

	let count = hits.len();
	let mut cells = Vec::with_capacity(count);
	for k in 0..count {
		let next = (k + 1) % count;
		let mut vertices = vec![center];
		if let Some(door) = wall_doorways[k] {
			vertices.push(door);
		}
		let mut i = walk_pos[k];
		loop {
			vertices.push(walk[i].0);
			if i == walk_pos[next] {
				break;
			}
			i = (i + 1) % walk.len();
		}
		if let Some(door) = wall_doorways[next] {
			vertices.push(door);
		}
		vertices.dedup();
		cells.push(RspCell { vertices });
	}
	RaySplit {
		cells,
		wall_doorways,
	}
}

/// The single boundary point where the ray `center + t·direction` (`t > 0`) leaves the cell.
///
/// Uses one sign per vertex: the forward exit is the only place where
/// `cross(direction, v − center)` turns from negative to non-negative while ahead of the center.
fn ray_exit(cell: &RspCell, center: RspPoint, direction: RspPoint) -> RayHit {
	let n = cell.vertices.len();
	let side: Vec<f64> = cell
		.vertices
		.iter()
		.map(|v| cross_point(direction, *v - center))
		.collect();
	for i in 0..n {
		let a = cell.vertices[i];
		if side[i] == 0.0 && dot_point(direction, a - center) > 0.0 {
			return RayHit {
				index: i,
				param: 0.0,
				point: a,
			};
		}
		let j = (i + 1) % n;
		if side[i] < 0.0 && side[j] > 0.0 {
			let param = side[i] / (side[i] - side[j]);
			let point = a + (cell.vertices[j] - a) * param;
			if dot_point(direction, point - center) > 0.0 {
				return RayHit {
					index: i,
					param,
					point,
				};
			}
		}
	}
	panic!("ray from an interior center must leave a convex RSP cell exactly once");
}

#[cfg(test)]
mod tests {
	use super::*;
	use margen_rsp::RspPoint as P;

	fn unit(angle: f64) -> P {
		P::new(angle.cos(), angle.sin())
	}

	#[test]
	fn seed_cell_area_and_centroid() {
		let cell = seed_cell();
		assert!((polygon_area(&cell) - 1.0).abs() < 1e-12);
		let c = polygon_centroid(&cell);
		assert!(c.x.abs() < 1e-12 && c.y.abs() < 1e-12);
	}

	#[test]
	fn binary_axis_split_halves_square() {
		let cell = seed_cell();
		let split = split_by_rays(
			&cell,
			P::new(0.0, 0.0),
			&[unit(0.0), unit(std::f64::consts::PI)],
			&[false, false],
		);
		assert_eq!(split.cells.len(), 2);
		for child in &split.cells {
			assert!((polygon_area(child) - 0.5).abs() < 1e-12);
		}
	}

	#[test]
	fn binary_diagonal_split_through_vertices() {
		// The rays hit the corners exactly: hits must reuse the corner vertices verbatim.
		let cell = seed_cell();
		let d = P::new(1.0, 1.0);
		let split = split_by_rays(
			&cell,
			P::new(0.0, 0.0),
			&[d, P::new(-1.0, -1.0)],
			&[false, false],
		);
		for child in &split.cells {
			assert!((polygon_area(child) - 0.5).abs() < 1e-12);
			assert!(child.vertices.contains(&P::new(0.5, 0.5)));
			assert!(child.vertices.contains(&P::new(-0.5, -0.5)));
		}
	}

	#[test]
	fn ternary_split_tiles_parent_and_inserts_wall_doorways() {
		let cell = seed_cell();
		let base = 0.3;
		let dirs: Vec<P> = (0..3)
			.map(|k| unit(base + k as f64 * std::f64::consts::TAU / 3.0))
			.collect();
		let split = split_by_rays(&cell, P::new(0.0, 0.0), &dirs, &[true, false, true]);
		let total: f64 = split.cells.iter().map(polygon_area).sum();
		assert!((total - 1.0).abs() < 1e-12);
		for child in &split.cells {
			assert!(polygon_area(child) > 0.0);
		}
		let d0 = split.wall_doorways[0].unwrap();
		let d2 = split.wall_doorways[2].unwrap();
		assert!(split.wall_doorways[1].is_none());
		// Wall 0 is shared by children 2 and 0; wall 2 by children 1 and 2.
		assert!(split.cells[0].vertices.contains(&d0));
		assert!(split.cells[2].vertices.contains(&d0));
		assert!(split.cells[1].vertices.contains(&d2));
		assert!(split.cells[2].vertices.contains(&d2));
		assert!(!split.cells[1].vertices.contains(&d0));
	}

	#[test]
	fn parent_vertex_on_split_ray_goes_to_both_children() {
		// A doorway-like collinear vertex sits exactly on the ray at +X.
		let mut cell = seed_cell();
		cell.vertices.insert(2, P::new(0.5, 0.0));
		let split = split_by_rays(
			&cell,
			P::new(0.0, 0.0),
			&[unit(0.0), unit(std::f64::consts::PI)],
			&[false, false],
		);
		assert!(split.cells[0].vertices.contains(&P::new(0.5, 0.0)));
		assert!(split.cells[1].vertices.contains(&P::new(0.5, 0.0)));
	}

	#[test]
	fn clamp_keeps_point_inside_shrunk_cell() {
		let cell = seed_cell();
		let center = polygon_centroid(&cell);
		let shrunk = shrink_toward(&cell, center, 0.8);
		let inside = P::new(0.1, -0.2);
		assert_eq!(clamp_into_cell(inside, center, &shrunk), inside);
		let clamped = clamp_into_cell(P::new(3.0, 1.0), center, &shrunk);
		assert!(clamped.x <= 0.4 + 1e-12 && clamped.y <= 0.4 + 1e-12);
		assert!((clamped.x - 0.4).abs() < 1e-12);
	}
}
