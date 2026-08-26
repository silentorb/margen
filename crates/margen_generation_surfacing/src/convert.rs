use crate::types::{
	CellFaceMap, FaceAppearanceMap, FaceStructureMap, RenderFace, RenderFaceMap, RenderFaces,
};

pub fn convert_faces(
	cell_faces: &CellFaceMap,
	face_structures: &FaceStructureMap,
	appearance_map: &FaceAppearanceMap,
) -> RenderFaces {
	let mut render_faces = RenderFaces::new();
	for (face_location, _cell_face) in cell_faces {
		let mut record = RenderFace {
			location: face_location.location,
			..Default::default()
		};
		if let Some(structure) = face_structures.get(face_location) {
			record.points = structure.points.clone();
		}
		if let Some(appearance) = appearance_map.get(face_location) {
			record.uvs = appearance.uvs.clone();
			record.material_id = appearance.material_id;
		}
		render_faces.push(record);
	}
	render_faces
}

/// Group faces by sector (one face each; corrects Unreal's accidental full-list copy).
pub fn convert_faces_by_sector(
	cell_faces: &CellFaceMap,
	face_structures: &FaceStructureMap,
	appearance_map: &FaceAppearanceMap,
) -> RenderFaceMap {
	let mut render_face_map = RenderFaceMap::new();
	for (face_location, cell_face) in cell_faces {
		let mut record = RenderFace {
			location: face_location.location,
			..Default::default()
		};
		if let Some(structure) = face_structures.get(face_location) {
			record.points = structure.points.clone();
		}
		if let Some(appearance) = appearance_map.get(face_location) {
			record.uvs = appearance.uvs.clone();
			record.material_id = appearance.material_id;
		}
		render_face_map
			.entry(cell_face.sector)
			.or_default()
			.push(record);
	}
	render_face_map
}
