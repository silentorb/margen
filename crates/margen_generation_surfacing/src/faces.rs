use crate::types::{
	to_face_column_location, CellFace, CellFaceMap, ColumnSpan, FaceColumnSpans,
	FaceColumnSpansMap, FaceLocation,
};
use margen_generation::{
	cell_attribute, cell_has_attribute, cell_is_solid, directions::directions,
	get_grid_direction_vector, rotate_direction_index, CellGrid, HexCellCornersMap, WorldCell,
};
use mythic_math::{Vector2, Vector3i};

pub fn is_face_a_wall(direction: i32) -> bool {
	direction < 6
}

const CELL_SIDE_DIRECTIONS: [i32; 8] = [
	directions::NORTH as i32,
	directions::NORTHEAST as i32,
	directions::SOUTHEAST as i32,
	directions::SOUTH as i32,
	directions::SOUTHWEST as i32,
	directions::NORTHWEST as i32,
	directions::UP as i32,
	directions::DOWN as i32,
];

fn get_face_corner_heights(
	hex_cell_corners_map: &HexCellCornersMap,
	location: Vector3i,
	direction: i32,
	cell: Option<&WorldCell>,
) -> Vector2 {
	if let Some(cell_corners) = hex_cell_corners_map.get(&location) {
		let left = rotate_direction_index(direction as i8, -1) as usize + 6;
		let right = direction as usize + 6;
		Vector2::new(cell_corners[left].z, cell_corners[right].z)
	} else if cell.map(cell_is_solid).unwrap_or(false) {
		Vector2::new(1.0, 1.0)
	} else {
		Vector2::new(-1.0, -1.0)
	}
}

pub fn is_face_visible(
	grid: &CellGrid,
	hex_cell_corners_map: &HexCellCornersMap,
	cell: &WorldCell,
	location: Vector3i,
	direction: i32,
) -> bool {
	let neighbor_location = location + get_grid_direction_vector(direction as i8);
	if let Some(neighbor) = grid.get(neighbor_location) {
		if !is_face_a_wall(direction) {
			let is_concealed = neighbor.sector == cell.sector
				&& !cell_has_attribute(neighbor, cell_attribute::CUSTOM_MESH)
				&& cell_is_solid(neighbor);
			!is_concealed
		} else {
			let local_heights =
				get_face_corner_heights(hex_cell_corners_map, location, direction, Some(cell));
			let neighbor_heights = get_face_corner_heights(
				hex_cell_corners_map,
				neighbor_location,
				rotate_direction_index(direction as i8, 3) as i32,
				Some(neighbor),
			);
			local_heights.x > neighbor_heights.y || local_heights.y > neighbor_heights.x
		}
	} else {
		true
	}
}

fn gather_solid_cell_faces(
	grid: &CellGrid,
	hex_cell_corners_map: &HexCellCornersMap,
	location: Vector3i,
	cell: &WorldCell,
	faces: &mut CellFaceMap,
) {
	for direction in CELL_SIDE_DIRECTIONS {
		if is_face_visible(grid, hex_cell_corners_map, cell, location, direction) {
			faces.insert(
				FaceLocation::new(location, direction),
				CellFace {
					sector: cell.sector,
					biome: cell.biome.clone(),
				},
			);
		}
	}
}

pub fn gather_faces(grid: &CellGrid, hex_cell_corners_map: &HexCellCornersMap) -> CellFaceMap {
	let mut faces = CellFaceMap::new();
	for (location, cell) in grid.iter() {
		if cell_is_solid(cell) && !cell_has_attribute(cell, cell_attribute::CUSTOM_MESH) {
			gather_solid_cell_faces(grid, hex_cell_corners_map, *location, cell, &mut faces);
		}
	}
	faces
}

fn get_vertical_span_in_direction(
	faces: &CellFaceMap,
	origin: Vector3i,
	direction: i32,
	z_step: i32,
) -> i32 {
	let mut z = origin.z;
	for _ in 0..1000 {
		z += z_step;
		let location = Vector3i::new(origin.x, origin.y, z);
		if !faces.contains_key(&FaceLocation::new(location, direction)) {
			break;
		}
	}
	z - z_step
}

fn new_wall_column_span(faces: &CellFaceMap, face_location: FaceLocation) -> ColumnSpan {
	ColumnSpan {
		bottom: get_vertical_span_in_direction(
			faces,
			face_location.location,
			face_location.direction,
			-1,
		),
		top: get_vertical_span_in_direction(
			faces,
			face_location.location,
			face_location.direction,
			1,
		),
	}
}

fn has_wall_column(column_spans: &FaceColumnSpans, z: i32) -> bool {
	column_spans
		.iter()
		.any(|span| z >= span.bottom && z <= span.top)
}

pub fn gather_wall_columns(faces: &CellFaceMap) -> FaceColumnSpansMap {
	let mut wall_span_map = FaceColumnSpansMap::new();
	for face_location in faces.keys() {
		if !is_face_a_wall(face_location.direction) {
			continue;
		}
		let column = to_face_column_location(*face_location);
		let column_spans = wall_span_map.entry(column).or_default();
		let z = face_location.location.z;
		if !has_wall_column(column_spans, z) {
			column_spans.push(new_wall_column_span(faces, *face_location));
		}
	}
	wall_span_map
}
