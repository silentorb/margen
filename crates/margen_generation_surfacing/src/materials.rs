use crate::types::{
	to_face_location, CellFaceMap, FaceAppearance, FaceAppearanceMap, FaceColumnSpansMap,
	FaceLocation, FaceMetadataMap, FaceStructureMap, SurfacePoints, SurfaceUvs,
};
use margen_generation::{
	directions::directions, resolve_cell_coordinates, Biome, MaterialPlaceable,
	MaterialPlaceableId,
};
use mythic_dice::Dice;
use mythic_math::{Vector2, Vector2i, Vector3};
use std::collections::{BTreeMap, BTreeSet};

pub const FACE_TAG_WALL: &str = "Face.Wall";
pub const FACE_TAG_FLOOR: &str = "Face.Floor";
pub const FACE_TAG_CEILING: &str = "Face.Ceiling";
pub const FACE_TAG_SLOPE_TOP: &str = "Face.SlopeTop";

/// Maps placeable id → sequential material slot index (Unreal `UsedMaterials`).
pub type UsedMaterials = BTreeMap<MaterialPlaceableId, i32>;

pub fn use_material(id: MaterialPlaceableId, used_materials: &mut UsedMaterials) -> i32 {
	if let Some(&existing) = used_materials.get(&id) {
		return existing;
	}
	let index = used_materials.len() as i32;
	used_materials.insert(id, index);
	index
}

fn has_all_tags(required: &BTreeSet<String>, tags: &[String]) -> bool {
	required.iter().all(|r| tags.iter().any(|t| t == r))
}

fn has_no_tags(prevented: &BTreeSet<String>, tags: &[String]) -> bool {
	!tags.iter().any(|t| prevented.contains(t))
}

pub fn select_face_material<'a>(
	dice: &mut Dice,
	face_tags: &[String],
	span_dimensions: Vector2i,
	materials: &'a [MaterialPlaceable],
) -> Option<&'a MaterialPlaceable> {
	let options: Vec<&MaterialPlaceable> = materials
		.iter()
		.filter(|material| {
			has_all_tags(&material.requirements, face_tags)
				&& has_no_tags(&material.prevented_tags, face_tags)
				&& material.minimum_length.y <= span_dimensions.y
		})
		.collect();
	if options.is_empty() {
		None
	} else {
		Some(dice.from_slice(&options))
	}
}

pub fn get_face_biome<'a>(
	cell_faces: &'a CellFaceMap,
	face_location: FaceLocation,
) -> Option<&'a Biome> {
	cell_faces
		.get(&face_location)
		.and_then(|face| face.biome.as_deref())
}

/// UV extents as (lower U, upper U, lower V, upper V).
#[derive(Clone, Copy, Debug)]
pub struct UvExtents {
	pub u0: f32,
	pub u1: f32,
	pub v0: f32,
	pub v1: f32,
}

impl Default for UvExtents {
	fn default() -> Self {
		Self {
			u0: 0.0,
			u1: 1.0,
			v0: 0.0,
			v1: 1.0,
		}
	}
}

pub fn arrange_snapped_quad_uvs(
	collapsed_point: i32,
	uvs: &mut SurfaceUvs,
	uv_extents: UvExtents,
) {
	let uv_quad = [
		Vector2::new(uv_extents.u0, uv_extents.v1),
		Vector2::new(uv_extents.u1, uv_extents.v1),
		Vector2::new(uv_extents.u1, uv_extents.v0),
		Vector2::new(uv_extents.u0, uv_extents.v0),
	];
	for (index, uv) in uv_quad.iter().enumerate() {
		if index as i32 != collapsed_point {
			uvs.push(*uv);
		}
	}
}

pub fn arrange_global_uvs(
	points: &SurfacePoints,
	offset: Vector3,
	uv_scale: f32,
	uvs: &mut SurfaceUvs,
) {
	let wrapped_origin = Vector3::new(
		offset.x - offset.x.floor(),
		offset.y - offset.y.floor(),
		0.0,
	);
	uvs.clear();
	for point in points {
		let location = *point - wrapped_origin;
		uvs.push(Vector2::new(location.y * uv_scale, location.x * uv_scale));
	}
}

fn get_span_extents(division_length: f32, step: f32) -> Vector2 {
	let lower = step * division_length;
	let upper = (lower + division_length).min(1.0);
	Vector2::new(lower, upper)
}

fn are_sloped_points(points: &SurfacePoints) -> bool {
	if points.len() == 3 {
		return true;
	}
	if points.len() < 4 {
		return false;
	}
	points[2].z != points[3].z
}

fn calculate_wall_uv_extents(texture_divisions: f32, division_height: f32, step: f32) -> UvExtents {
	let initial_wrapped_step = step % texture_divisions;
	let wrapped_step = texture_divisions - 1.0 - initial_wrapped_step;
	let v_extents = get_span_extents(division_height, wrapped_step);
	UvExtents {
		u0: 0.0,
		u1: 1.0,
		v0: v_extents.x,
		v1: v_extents.y,
	}
}

fn set_wall_span_appearance(
	dice: &mut Dice,
	column: crate::types::FaceColumnLocation,
	span: crate::types::ColumnSpan,
	biome: &Biome,
	face_structure_map: &FaceStructureMap,
	used_materials: &mut UsedMaterials,
	appearance_map: &mut FaceAppearanceMap,
	face_metadata_map: &mut FaceMetadataMap,
) {
	if biome.materials.is_empty() {
		return;
	}
	let mut face_tags = vec![FACE_TAG_WALL.to_string()];
	let top_location = to_face_location(column, span.top);
	if let Some(top_structure) = face_structure_map.get(&top_location) {
		if are_sloped_points(&top_structure.points) {
			face_tags.push(FACE_TAG_SLOPE_TOP.to_string());
		}
	}
	let span_height = span.top - span.bottom + 1;
	let Some(material) = select_face_material(
		dice,
		&face_tags,
		Vector2i::new(1, span_height),
		&biome.materials,
	) else {
		return;
	};
	let material_id = use_material(material.id, used_materials);
	let texture_divisions = material.face_span.y;
	let division_height = 1.0 / texture_divisions;
	let mut step = -1.0_f32;
	for h in span.bottom..=span.top {
		step += 1.0;
		let face_location = to_face_location(column, h);
		face_metadata_map.insert(face_location, material.id);
		if let Some(structure) = face_structure_map.get(&face_location) {
			let mut appearance = FaceAppearance::new();
			appearance.material_id = material_id;
			let uv_extents = calculate_wall_uv_extents(texture_divisions, division_height, step);
			arrange_snapped_quad_uvs(structure.collapsed_point, &mut appearance.uvs, uv_extents);
			appearance_map.insert(face_location, appearance);
		}
	}
}

pub fn set_wall_face_appearances(
	dice: &mut Dice,
	cell_faces: &CellFaceMap,
	wall_span_map: &FaceColumnSpansMap,
	face_structure_map: &FaceStructureMap,
	used_materials: &mut UsedMaterials,
	appearance_map: &mut FaceAppearanceMap,
	face_metadata_map: &mut FaceMetadataMap,
) {
	for (column, spans) in wall_span_map {
		for span in spans {
			if let Some(biome) = get_face_biome(cell_faces, to_face_location(*column, span.bottom)) {
				set_wall_span_appearance(
					dice,
					*column,
					*span,
					biome,
					face_structure_map,
					used_materials,
					appearance_map,
					face_metadata_map,
				);
			}
		}
	}
}

fn set_floor_and_ceiling_face_appearance(
	dice: &mut Dice,
	face_location: FaceLocation,
	biome: &Biome,
	face_structure_map: &FaceStructureMap,
	used_materials: &mut UsedMaterials,
	appearance_map: &mut FaceAppearanceMap,
	face_metadata_map: &mut FaceMetadataMap,
) {
	if biome.materials.is_empty() {
		return;
	}
	let face_tags = if face_location.direction == directions::UP as i32 {
		vec![FACE_TAG_FLOOR.to_string()]
	} else {
		vec![FACE_TAG_CEILING.to_string()]
	};
	let Some(material) =
		select_face_material(dice, &face_tags, Vector2i::new(1, 1), &biome.materials)
	else {
		return;
	};
	let material_id = use_material(material.id, used_materials);
	let Some(structure) = face_structure_map.get(&face_location) else {
		return;
	};
	face_metadata_map.insert(face_location, material.id);
	let mut appearance = FaceAppearance::new();
	appearance.material_id = material_id;
	// Effective Unreal path: overwrite with global UVs at scale 0.75.
	let center = resolve_cell_coordinates(face_location.location);
	arrange_global_uvs(&structure.points, center, 0.75, &mut appearance.uvs);
	appearance_map.insert(face_location, appearance);
}

pub fn set_floor_and_ceiling_face_appearances(
	dice: &mut Dice,
	cell_faces: &CellFaceMap,
	face_structure_map: &FaceStructureMap,
	used_materials: &mut UsedMaterials,
	appearance_map: &mut FaceAppearanceMap,
	face_metadata_map: &mut FaceMetadataMap,
) {
	for (face_location, cell_face) in cell_faces {
		if crate::faces::is_face_a_wall(face_location.direction) {
			continue;
		}
		if let Some(biome) = cell_face.biome.as_deref() {
			set_floor_and_ceiling_face_appearance(
				dice,
				*face_location,
				biome,
				face_structure_map,
				used_materials,
				appearance_map,
				face_metadata_map,
			);
		}
	}
}
