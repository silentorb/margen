//! Face IR pipeline: gather visible faces → arrange points → paint → RenderFace.

pub mod convert;
pub mod faces;
pub mod materials;
pub mod structure;
pub mod types;

#[cfg(test)]
mod tests;

pub use convert::convert_faces;
pub use faces::{gather_faces, gather_wall_columns, is_face_a_wall};
pub use materials::{
	set_floor_and_ceiling_face_appearances, set_wall_face_appearances, UsedMaterials,
};
pub use structure::arrange_face_points;
pub use types::*;

use margen_generation::{
	CellGrid, HexCellCornersMap, SharedCellVertexOffsets,
};
use mythic_dice::Dice;

/// Run the Surfacing face IR pipeline through `RenderFace` (no mesh apply).
pub fn generate_render_faces(
	dice: &mut Dice,
	grid: &CellGrid,
	corner_offsets: &SharedCellVertexOffsets,
	hex_cell_corners_map: &HexCellCornersMap,
) -> (RenderFaces, UsedMaterials) {
	let cell_faces = gather_faces(grid, hex_cell_corners_map);
	let wall_span_map = gather_wall_columns(&cell_faces);
	let face_structures = arrange_face_points(corner_offsets, hex_cell_corners_map, &cell_faces);

	let mut used_materials = UsedMaterials::new();
	let mut appearance_map = FaceAppearanceMap::new();
	let mut face_metadata = FaceMetadataMap::new();

	set_wall_face_appearances(
		dice,
		&cell_faces,
		&wall_span_map,
		&face_structures,
		&mut used_materials,
		&mut appearance_map,
		&mut face_metadata,
	);
	set_floor_and_ceiling_face_appearances(
		dice,
		&cell_faces,
		&face_structures,
		&mut used_materials,
		&mut appearance_map,
		&mut face_metadata,
	);

	let render_faces = convert_faces(&cell_faces, &face_structures, &appearance_map);
	(render_faces, used_materials)
}
