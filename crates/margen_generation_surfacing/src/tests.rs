use crate::{
	arrange_face_points, gather_faces, generate_render_faces, materials, FaceLocation,
	FaceStructureMap,
};
use margen_generation::{
	cell_add_attributes, cell_attribute, directions::directions, get_grid_direction_vector, Biome,
	CellGrid, HexCellCornersMap, MaterialPlaceable, SharedCellVertexOffsets, WorldCell,
};
use mythic_dice::Dice;
use mythic_math::{Vector3, Vector3i};
use std::sync::Arc;

fn solid_cell_with_materials() -> WorldCell {
	let mut wall = MaterialPlaceable::new(1);
	wall.requirements
		.insert(materials::FACE_TAG_WALL.to_string());
	let mut floor = MaterialPlaceable::new(2);
	floor
		.requirements
		.insert(materials::FACE_TAG_FLOOR.to_string());
	let mut ceiling = MaterialPlaceable::new(3);
	ceiling
		.requirements
		.insert(materials::FACE_TAG_CEILING.to_string());
	let mut biome = Biome::new(0);
	biome.materials = vec![wall, floor, ceiling];
	let mut cell = WorldCell {
		biome: Some(Arc::new(biome)),
		..Default::default()
	};
	cell_add_attributes(&mut cell, cell_attribute::SOLID);
	cell
}

#[test]
fn single_solid_cell_exposes_all_sides() {
	let mut grid = CellGrid::new();
	grid.insert(Vector3i::zero(), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let faces = gather_faces(&grid, &hex_map);
	assert_eq!(faces.len(), 8);
	assert!(faces.contains_key(&FaceLocation::new(
		Vector3i::zero(),
		directions::NORTH as i32
	)));
	assert!(faces.contains_key(&FaceLocation::new(
		Vector3i::zero(),
		directions::UP as i32
	)));
	assert!(faces.contains_key(&FaceLocation::new(
		Vector3i::zero(),
		directions::DOWN as i32
	)));
}

#[test]
fn adjacent_solids_conceal_shared_wall() {
	let mut grid = CellGrid::new();
	grid.insert(Vector3i::zero(), solid_cell_with_materials());
	grid.insert(Vector3i::new(0, 1, 0), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let faces = gather_faces(&grid, &hex_map);
	assert!(!faces.contains_key(&FaceLocation::new(
		Vector3i::zero(),
		directions::NORTH as i32
	)));
	assert!(!faces.contains_key(&FaceLocation::new(
		Vector3i::new(0, 1, 0),
		directions::SOUTH as i32
	)));
}

#[test]
fn arrange_wall_has_quad_points_floor_has_ring() {
	let mut grid = CellGrid::new();
	grid.insert(Vector3i::zero(), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let offsets = SharedCellVertexOffsets::new();
	let faces = gather_faces(&grid, &hex_map);
	let structures = arrange_face_points(&offsets, &hex_map, &faces);
	let wall = structures
		.get(&FaceLocation::new(
			Vector3i::zero(),
			directions::NORTH as i32,
		))
		.expect("wall");
	assert_eq!(wall.points.len(), 4);
	let floor = structures
		.get(&FaceLocation::new(
			Vector3i::zero(),
			directions::DOWN as i32,
		))
		.expect("floor");
	assert_eq!(floor.points.len(), 8);
}

fn polygon_normal(points: &[Vector3]) -> Vector3 {
	let a = points[0];
	let b = points[1];
	let c = points[2];
	let ab = Vector3::new(b.x - a.x, b.y - a.y, b.z - a.z);
	let ac = Vector3::new(c.x - a.x, c.y - a.y, c.z - a.z);
	Vector3::new(
		ab.y * ac.z - ab.z * ac.y,
		ab.z * ac.x - ab.x * ac.z,
		ab.x * ac.y - ab.y * ac.x,
	)
}

fn dot(a: Vector3, b: Vector3) -> f32 {
	a.x * b.x + a.y * b.y + a.z * b.z
}

/// Godot RH Y-up map used by margen-godot: (x, y, z) -> (x, z, -y).
fn margen_to_godot(v: Vector3) -> Vector3 {
	Vector3::new(v.x, v.z, -v.y)
}

const OUTWARD_FACE_DIRECTIONS: [i32; 8] = [
	directions::NORTH as i32,
	directions::NORTHEAST as i32,
	directions::SOUTHEAST as i32,
	directions::SOUTH as i32,
	directions::SOUTHWEST as i32,
	directions::NORTHWEST as i32,
	directions::UP as i32,
	directions::DOWN as i32,
];

fn arranged_origin_faces() -> (FaceStructureMap, Vec<(i32, Vector3)>) {
	let mut grid = CellGrid::new();
	grid.insert(Vector3i::zero(), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let offsets = SharedCellVertexOffsets::new();
	let faces = gather_faces(&grid, &hex_map);
	let structures = arrange_face_points(&offsets, &hex_map, &faces);
	let expected: Vec<(i32, Vector3)> = OUTWARD_FACE_DIRECTIONS
		.iter()
		.map(|&dir| {
			let v = get_grid_direction_vector(dir as i8);
			(dir, Vector3::new(v.x as f32, v.y as f32, v.z as f32))
		})
		.collect();
	(structures, expected)
}

#[test]
fn face_normals_point_outward() {
	let (structures, expected) = arranged_origin_faces();
	for (dir, outward) in expected {
		let points = &structures
			.get(&FaceLocation::new(Vector3i::zero(), dir))
			.unwrap_or_else(|| panic!("missing face direction {dir}"))
			.points;
		let normal = polygon_normal(points);
		assert!(
			dot(normal, outward) > 0.0,
			"direction {dir}: normal {normal:?} should point along outward {outward:?}"
		);
	}
}

#[test]
fn face_normals_stay_outward_in_godot_space() {
	let (structures, expected) = arranged_origin_faces();
	for (dir, outward_margen) in expected {
		let points: Vec<Vector3> = structures
			.get(&FaceLocation::new(Vector3i::zero(), dir))
			.unwrap_or_else(|| panic!("missing face direction {dir}"))
			.points
			.iter()
			.copied()
			.map(margen_to_godot)
			.collect();
		let normal = polygon_normal(&points);
		let outward_godot = margen_to_godot(outward_margen);
		assert!(
			dot(normal, outward_godot) > 0.0,
			"direction {dir}: Godot normal {normal:?} should point along remapped outward {outward_godot:?}"
		);
	}
}

#[test]
fn generate_render_faces_assigns_material_ids() {
	let mut grid = CellGrid::new();
	grid.insert(Vector3i::zero(), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let offsets = SharedCellVertexOffsets::new();
	let mut dice = Dice::new(1);
	let (render_faces, used) = generate_render_faces(&mut dice, &grid, &offsets, &hex_map);
	assert_eq!(render_faces.len(), 8);
	assert!(!used.is_empty());
	assert!(render_faces.iter().any(|f| f.material_id >= 0));
	assert!(render_faces.iter().all(|f| !f.points.is_empty()));
}
