use crate::{
	arrange_face_points, gather_faces, generate_render_faces, materials, FaceLocation,
};
use margen_generation::{
	cell_add_attributes, cell_attribute, directions::directions, Biome, CellGrid,
	HexCellCornersMap, MaterialPlaceable, SharedCellVertexOffsets, WorldCell,
};
use mythic_dice::Dice;
use mythic_math::Vector3i;
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
	grid.insert(Vector3i::new(0, -1, 0), solid_cell_with_materials());
	let hex_map = HexCellCornersMap::new();
	let faces = gather_faces(&grid, &hex_map);
	assert!(!faces.contains_key(&FaceLocation::new(
		Vector3i::zero(),
		directions::NORTH as i32
	)));
	assert!(!faces.contains_key(&FaceLocation::new(
		Vector3i::new(0, -1, 0),
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
