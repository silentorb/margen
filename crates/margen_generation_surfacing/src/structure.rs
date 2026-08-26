use crate::types::{
	CellFaceMap, FaceLocation, FaceStructure, FaceStructureMap, SurfacePoints,
};
use margen_generation::{
	apply_relative_cell_offsets, directions::directions,
	resolve_relative_cell_corner_coordinates, HexCellCorners, HexCellCornersMap,
	SharedCellVertexOffsets,
};
use mythic_math::Vector3;

fn add_polygon_point(points: &mut SurfacePoints, point: Vector3, collapsed_point: &mut i32) {
	if points.iter().any(|p| *p == point) {
		*collapsed_point = points.len() as i32;
	} else {
		points.push(point);
	}
}

pub fn get_surface_points(
	corners: &HexCellCorners,
	direction: i32,
	collapsed_point: &mut i32,
) -> SurfacePoints {
	let mut points = SurfacePoints::new();
	*collapsed_point = -1;

	if direction == directions::UP as i32 {
		points.push(corners[13]);
		for i in (0..6).rev() {
			points.push(corners[i + 6]);
		}
		points.push(corners[5 + 6]);
	} else if direction == directions::DOWN as i32 {
		points.push(corners[12]);
		for i in 0..6 {
			points.push(corners[i]);
		}
		points.push(corners[0]);
	} else {
		let i = ((direction + 5) % 6) as usize;
		let next = direction as usize;
		let lower1 = corners[i];
		let lower2 = corners[next];
		let upper1 = corners[i + 6];
		let upper2 = corners[next + 6];
		add_polygon_point(&mut points, lower2, collapsed_point);
		add_polygon_point(&mut points, lower1, collapsed_point);
		add_polygon_point(&mut points, upper1, collapsed_point);
		add_polygon_point(&mut points, upper2, collapsed_point);
	}

	points
}

fn arrange_one_face(
	corner_offsets: &SharedCellVertexOffsets,
	hex_cell_corners_map: &HexCellCornersMap,
	face_location: FaceLocation,
) -> FaceStructure {
	let mut corners = resolve_relative_cell_corner_coordinates(corner_offsets, face_location.location);
	if let Some(cell_offsets) = hex_cell_corners_map.get(&face_location.location) {
		apply_relative_cell_offsets(cell_offsets, &mut corners);
	}
	let mut collapsed_point = -1;
	let points = get_surface_points(&corners, face_location.direction, &mut collapsed_point);
	FaceStructure {
		points,
		collapsed_point,
	}
}

pub fn arrange_face_points(
	corner_offsets: &SharedCellVertexOffsets,
	hex_cell_corners_map: &HexCellCornersMap,
	cell_faces: &CellFaceMap,
) -> FaceStructureMap {
	let mut faces = FaceStructureMap::new();
	for face_location in cell_faces.keys() {
		faces.insert(
			*face_location,
			arrange_one_face(corner_offsets, hex_cell_corners_map, *face_location),
		);
	}
	faces
}
