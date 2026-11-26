//! Per-piece skinners: footprints → triangles.
//!
//! [`assign_skinners`] is the delegation seam: it picks a skinner for every floor piece.
//! Today every piece becomes a flat slab; region- or biome-specific skinners are added as
//! new [`RspPieceSkinner`] variants and chosen here.

use margen_rsp::RspPoint;
use mythic_math::Vector3;

use crate::{RspFloorPiece, RspFloorPlan, RspMesh, RspMeshPiece, RspSkinConfig};

/// How one floor piece is turned into geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RspPieceSkinner {
	/// Prism with its top at z = 0 and bottom at z = −`thickness`. Side walls are emitted
	/// only on edges that are not joins, so adjoining slabs form one closed surface.
	Slab { thickness: f64 },
}

/// One skinner per piece of `plan` (parallel to `plan.pieces`).
pub fn assign_skinners(plan: &RspFloorPlan, config: &RspSkinConfig) -> Vec<RspPieceSkinner> {
	let thickness = f64::from(config.floor_thickness);
	plan.pieces
		.iter()
		.map(|_| RspPieceSkinner::Slab { thickness })
		.collect()
}

/// Skin every piece with its assigned skinner into one mesh.
pub fn skin_floor_plan(plan: &RspFloorPlan, skinners: &[RspPieceSkinner]) -> RspMesh {
	assert_eq!(
		plan.pieces.len(),
		skinners.len(),
		"one skinner per floor piece"
	);
	let mut mesh = RspMesh::default();
	for (piece, skinner) in plan.pieces.iter().zip(skinners) {
		extrude_piece(&mut mesh, piece, *skinner);
	}
	mesh
}

/// Append `piece` to `mesh` using `skinner`.
pub fn extrude_piece(mesh: &mut RspMesh, piece: &RspFloorPiece, skinner: RspPieceSkinner) {
	let index_begin = mesh.indices.len() as u32;
	match skinner {
		RspPieceSkinner::Slab { thickness } => extrude_slab(mesh, piece, thickness),
	}
	mesh.pieces.push(RspMeshPiece {
		kind: piece.kind,
		index_begin,
		index_count: mesh.indices.len() as u32 - index_begin,
	});
}

fn extrude_slab(mesh: &mut RspMesh, piece: &RspFloorPiece, thickness: f64) {
	let polygon = &piece.polygon;
	let n = polygon.len();
	let top = |p: RspPoint| vertex(p, 0.0);
	let bottom = |p: RspPoint| vertex(p, -thickness);

	// Top (+Z) is counter-clockwise from above; bottom reverses it.
	let top_ring: Vec<Vector3> = polygon.iter().map(|p| top(*p)).collect();
	push_fan(mesh, &top_ring, Vector3::new(0.0, 0.0, 1.0));
	let bottom_ring: Vec<Vector3> = polygon.iter().rev().map(|p| bottom(*p)).collect();
	push_fan(mesh, &bottom_ring, Vector3::new(0.0, 0.0, -1.0));

	for i in 0..n {
		if piece.joins.contains(&i) {
			continue;
		}
		let a = polygon[i];
		let b = polygon[(i + 1) % n];
		let edge = b - a;
		let len = (edge.x * edge.x + edge.y * edge.y).sqrt();
		// Interior is on the left of a counter-clockwise edge, so outward is its right.
		let normal = Vector3::new((edge.y / len) as f32, (-edge.x / len) as f32, 0.0);
		push_fan(mesh, &[top(a), bottom(a), bottom(b), top(b)], normal);
	}
}

/// Convex polygon (counter-clockwise seen from where `normal` points) as a triangle fan.
fn push_fan(mesh: &mut RspMesh, ring: &[Vector3], normal: Vector3) {
	let base = mesh.positions.len() as u32;
	mesh.positions.extend_from_slice(ring);
	mesh.normals.extend(std::iter::repeat(normal).take(ring.len()));
	for k in 1..ring.len() as u32 - 1 {
		mesh.indices.extend_from_slice(&[base, base + k, base + k + 1]);
	}
}

fn vertex(p: RspPoint, z: f64) -> Vector3 {
	Vector3::new(p.x as f32, p.y as f32, z as f32)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::RspPieceKind;
	use margen_rsp::RspNodeId;

	#[test]
	fn slab_triangle_count_matches_its_open_edges() {
		let piece = RspFloorPiece {
			kind: RspPieceKind::Room(RspNodeId(0)),
			polygon: vec![
				RspPoint::new(0.0, 0.0),
				RspPoint::new(2.0, 0.0),
				RspPoint::new(3.0, 1.0),
				RspPoint::new(1.0, 2.0),
				RspPoint::new(-1.0, 1.0),
			],
			joins: vec![1, 3],
		};
		let mut mesh = RspMesh::default();
		extrude_piece(&mut mesh, &piece, RspPieceSkinner::Slab { thickness: 0.5 });
		// Two caps of (n − 2) triangles plus two per non-join side.
		let n = piece.polygon.len();
		let expected = 2 * (n - 2) + 2 * (n - piece.joins.len());
		assert_eq!(mesh.indices.len(), expected * 3);
		assert_eq!(mesh.pieces[0].index_count as usize, expected * 3);
	}

	#[test]
	fn slab_triangles_face_their_normals() {
		let piece = RspFloorPiece {
			kind: RspPieceKind::Room(RspNodeId(0)),
			polygon: vec![
				RspPoint::new(0.0, 0.0),
				RspPoint::new(1.0, 0.0),
				RspPoint::new(1.0, 1.0),
				RspPoint::new(0.0, 1.0),
			],
			joins: vec![],
		};
		let mut mesh = RspMesh::default();
		extrude_piece(&mut mesh, &piece, RspPieceSkinner::Slab { thickness: 1.0 });
		for tri in mesh.indices.chunks(3) {
			let [a, b, c] = [0, 1, 2].map(|k| mesh.positions[tri[k] as usize]);
			let u = [b.x - a.x, b.y - a.y, b.z - a.z];
			let v = [c.x - a.x, c.y - a.y, c.z - a.z];
			let cross = [
				u[1] * v[2] - u[2] * v[1],
				u[2] * v[0] - u[0] * v[2],
				u[0] * v[1] - u[1] * v[0],
			];
			let n = mesh.normals[tri[0] as usize];
			assert!(cross[0] * n.x + cross[1] * n.y + cross[2] * n.z > 0.0);
		}
	}
}
