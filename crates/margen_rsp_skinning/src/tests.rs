use std::collections::BTreeMap;

use margen_rsp::{RspLayer, RspNodeId, RspPoint};
use margen_rsp_generation::{
	generate_skeletal_rsp, layout_rsp_metric_2d, validate_rsp_embedding, RspLayoutConfig,
};
use mythic_dice::Dice;

use crate::*;

fn skin(seed: i32, depth: i32, loop_weight: f32) -> (RspFloorPlan, RspMesh) {
	let mut dice = Dice::new(seed);
	let history = generate_skeletal_rsp(&mut dice, depth, loop_weight);
	let layout = layout_rsp_metric_2d(history.final_layer(), &RspLayoutConfig::default());
	skin_rsp_layer(&layout.layer, &RspSkinConfig::default())
		.unwrap_or_else(|e| panic!("seed {seed} depth {depth} weight {loop_weight}: {e:?}"))
}

/// Every directed edge appears exactly once and its reverse exactly once: a closed,
/// consistently oriented surface.
fn assert_closed_manifold(mesh: &RspMesh, label: &str) {
	let key = |i: u32| {
		let p = mesh.positions[i as usize];
		(p.x.to_bits(), p.y.to_bits(), p.z.to_bits())
	};
	let mut directed: BTreeMap<_, u32> = BTreeMap::new();
	for tri in mesh.indices.chunks(3) {
		for k in 0..3 {
			*directed
				.entry((key(tri[k]), key(tri[(k + 1) % 3])))
				.or_default() += 1;
		}
	}
	// Fan diagonals and polygon edges alike must pair up with their reverse.
	for ((a, b), count) in &directed {
		assert_eq!(*count, 1, "{label}: directed edge used {count} times");
		assert_eq!(
			directed.get(&(*b, *a)),
			Some(&1),
			"{label}: edge without its reverse"
		);
	}
}

#[test]
fn skins_are_overlap_free_and_closed_across_seeds() {
	let mut max_scale = 0.0f64;
	for depth in 1..=5 {
		// Layout relaxation dominates the cost in debug builds, so deep graphs get fewer seeds
		// (each still has hundreds of rooms) and the loop weight rotates across seeds.
		let seeds = if depth >= 5 { 15 } else { 60 };
		for seed in 0..seeds {
			let loop_weight = [0.0, 2.0 / 3.0, 1.0][seed as usize % 3];
			let label = format!("seed {seed} depth {depth} weight {loop_weight}");
			let (plan, mesh) = skin(seed, depth, loop_weight);
			assert_eq!(validate_floor_plan(&plan), Ok(()), "{label}");
			assert_eq!(validate_rsp_embedding(&plan.layer), Ok(()), "{label}");
			assert_eq!(plan.rooms.len(), plan.layer.graph.nodes.len(), "{label}");
			assert_eq!(
				plan.pieces.len(),
				plan.rooms.len() + 2 * plan.layer.graph.constraints.len(),
				"{label}"
			);
			assert_closed_manifold(&mesh, &label);
			max_scale = max_scale.max(plan.scale);
		}
	}
	assert!(max_scale < MAX_SKIN_SCALE);
}

#[test]
fn pathways_join_their_room_side_and_partner_door() {
	let (plan, _) = skin(3, 4, 2.0 / 3.0);
	let rooms: BTreeMap<RspNodeId, &RspFloorPiece> = plan
		.pieces
		.iter()
		.filter_map(|p| match p.kind {
			RspPieceKind::Room(node) => Some((node, p)),
			_ => None,
		})
		.collect();
	let edge = |piece: &RspFloorPiece, i: usize| {
		(piece.polygon[i], piece.polygon[(i + 1) % piece.polygon.len()])
	};
	for piece in &plan.pieces {
		let RspPieceKind::Pathway { constraint, node } = piece.kind else {
			continue;
		};
		// Edge 0 is the room side, traversed in reverse.
		let (plus, minus) = edge(piece, 0);
		let room = rooms[&node];
		assert!(room.joins.iter().any(|j| edge(room, *j) == (minus, plus)));
		// Edge 2 is the door, shared reversed with the other half of the same pathway.
		let (a, b) = edge(piece, 2);
		let partner = plan.pieces.iter().find(|other| {
			matches!(other.kind, RspPieceKind::Pathway { constraint: c, node: n } if c == constraint && n != node)
		});
		let partner = partner.expect("both halves exist");
		assert_eq!(edge(partner, 2), (b, a));
	}
}

#[test]
fn single_node_graph_is_one_regular_room() {
	let (plan, mesh) = skin(0, 0, 2.0 / 3.0);
	assert_eq!(plan.pieces.len(), 1);
	assert_eq!(plan.pieces[0].polygon.len(), 6);
	assert_closed_manifold(&mesh, "single room");
}

fn piece(kind: RspPieceKind, corners: &[(f64, f64)], joins: Vec<usize>) -> RspFloorPiece {
	RspFloorPiece {
		kind,
		polygon: corners.iter().map(|(x, y)| RspPoint::new(*x, *y)).collect(),
		joins,
	}
}

fn plan_of(pieces: Vec<RspFloorPiece>) -> RspFloorPlan {
	RspFloorPlan {
		layer: RspLayer::default(),
		scale: 1.0,
		rooms: Vec::new(),
		pieces,
	}
}

#[test]
fn validator_accepts_joined_and_separate_pieces() {
	let path = RspPieceKind::Pathway {
		constraint: 0,
		node: RspNodeId(0),
	};
	let plan = plan_of(vec![
		// Room edge 1 (2,0)→(2,2) is joined to pathway edge 3 (2,2)→(2,0).
		piece(
			RspPieceKind::Room(RspNodeId(0)),
			&[(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)],
			vec![1],
		),
		piece(path, &[(2.0, 0.0), (5.0, 0.0), (5.0, 2.0), (2.0, 2.0)], vec![3]),
		piece(
			RspPieceKind::Room(RspNodeId(1)),
			&[(0.0, 3.0), (2.0, 3.0), (2.0, 5.0), (0.0, 5.0)],
			vec![],
		),
	]);
	assert_eq!(validate_floor_plan(&plan), Ok(()));
}

#[test]
fn validator_rejects_overlaps_and_bad_pieces() {
	let a = RspPieceKind::Room(RspNodeId(0));
	let b = RspPieceKind::Room(RspNodeId(1));
	let square = [(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];

	let overlapping = plan_of(vec![
		piece(a, &square, vec![]),
		piece(b, &[(1.0, 1.0), (3.0, 1.0), (3.0, 3.0), (1.0, 3.0)], vec![]),
	]);
	assert_eq!(
		validate_floor_plan(&overlapping),
		Err(RspFloorPlanError::Overlap(0, 1))
	);

	// Touching along an edge counts as an overlap unless that edge is a declared join.
	let touching = plan_of(vec![
		piece(a, &square, vec![]),
		piece(b, &[(2.0, 0.0), (4.0, 0.0), (4.0, 2.0), (2.0, 2.0)], vec![]),
	]);
	assert_eq!(
		validate_floor_plan(&touching),
		Err(RspFloorPlanError::Overlap(0, 1))
	);

	let unmatched = plan_of(vec![piece(a, &square, vec![2])]);
	assert_eq!(
		validate_floor_plan(&unmatched),
		Err(RspFloorPlanError::UnmatchedJoin { piece: 0, edge: 2 })
	);

	let clockwise = plan_of(vec![piece(
		a,
		&[(0.0, 0.0), (0.0, 2.0), (2.0, 2.0), (2.0, 0.0)],
		vec![],
	)]);
	assert_eq!(
		validate_floor_plan(&clockwise),
		Err(RspFloorPlanError::InvalidPiece(0))
	);
}

#[test]
fn too_wide_pathway_is_blocked_by_its_slot() {
	let mut dice = Dice::new(5);
	let history = generate_skeletal_rsp(&mut dice, 3, 2.0 / 3.0);
	let layout = layout_rsp_metric_2d(history.final_layer(), &RspLayoutConfig::default());
	let config = RspSkinConfig {
		pathway_width: 1.0e6,
		..RspSkinConfig::default()
	};
	let config = sanitize_config(&config);
	let at_one = fit_floor_plan_at_scale(&layout.layer, &config, 1.0).expect("walls exist");
	assert!(matches!(at_one, Err(RspSkinBlocker::SlotTooShort { .. })));
}

