//! C ABI for margen world generation.

use margen_generation::{
	biome_from_id, get_actual_direction_vector, get_grid_direction_vector,
	resolve_absolute_cell_corner_coordinates, resolve_cell_coordinates_scaled,
	surface_default_biome, MaterialPlaceableId, SharedCellVertexOffsets, WorldCell,
	ALL_DIRECTIONS_COUNT, HEX_CELL_CORNER_COUNT,
};
use margen_generation_graphing::DagNodeId;
use margen_generation_population::{
	populate_world, ActorPlacements, PopulateInput, PLACEABLE_PLAYER_SPAWN,
};
use margen_generation_structure::{
	apply_path_slopes, generate_story_grid_with_config, generate_world_grid, StoryGridConfig,
	StoryGridResult, WorldGridConfig,
};
use margen_generation_surfacing::{generate_render_faces, RenderFaces, UsedMaterials};
use mythic_dice::Dice;
use mythic_math::{Vector3, Vector3i};
use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::os::raw::c_int;
use std::ptr;
use std::slice;

thread_local! {
	static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(message: &str) {
	LAST_ERROR.with(|slot| {
		*slot.borrow_mut() = Some(CString::new(message).unwrap_or_else(|_| {
			CString::new("invalid error message").expect("static")
		}));
	});
}

fn clear_last_error() {
	LAST_ERROR.with(|slot| {
		*slot.borrow_mut() = None;
	});
}

/// Opaque story-grid handle.
pub struct MargenStoryGrid {
	result: StoryGridResult,
	cells: Vec<(Vector3i, WorldCell)>,
	portals: Vec<(DagNodeId, DagNodeId)>,
}

impl MargenStoryGrid {
	fn from_result(result: StoryGridResult) -> Self {
		let mut cells: Vec<_> = result
			.grid
			.iter()
			.map(|(loc, cell)| (*loc, cell.clone()))
			.collect();
		cells.sort_by_key(|(loc, _)| (loc.x, loc.y, loc.z));
		let portals: Vec<_> = result.portal_connections.iter().copied().collect();
		Self {
			result,
			cells,
			portals,
		}
	}
}

#[repr(C)]
pub struct MargenCell {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub attributes: i64,
	pub paths: u32,
	pub biome_id: u32,
	pub sector: i16,
	pub _pad: i16,
}

#[repr(C)]
pub struct MargenPortal {
	pub from: i16,
	pub to: i16,
}

#[repr(C)]
pub struct MargenVec2 {
	pub x: f32,
	pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MargenVec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MargenIVec3 {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

#[repr(C)]
pub struct MargenRenderFaceInfo {
	pub location_x: i32,
	pub location_y: i32,
	pub location_z: i32,
	pub material_id: i32,
	pub point_count: u32,
	pub uv_count: u32,
}

#[repr(C)]
pub struct MargenMaterialSlot {
	pub placeable_id: u32,
	pub slot_index: i32,
}

#[repr(C)]
pub struct MargenBulkFaceDesc {
	pub point_begin: u32,
	pub point_count: u32,
	pub uv_begin: u32,
	pub uv_count: u32,
	pub location_x: i32,
	pub location_y: i32,
	pub location_z: i32,
	pub material_id: i32,
}

/// Abstract actor placement for host instantiation (first location only in this pass).
#[repr(C)]
pub struct MargenActorPlacement {
	pub placeable_id: u32,
	pub location_x: i32,
	pub location_y: i32,
	pub location_z: i32,
	pub biome_id: u32,
	pub sector: i16,
	pub _pad: i16,
}

struct WorldFacesBulk {
	points: Vec<MargenVec3>,
	uvs: Vec<MargenVec2>,
	faces: Vec<MargenBulkFaceDesc>,
}

/// Opaque world-faces handle (production grid + RenderFace IR + placements + cells).
pub struct MargenWorldFaces {
	render_faces: RenderFaces,
	materials: Vec<(MaterialPlaceableId, i32)>,
	player_start: Vector3i,
	placements: Vec<MargenActorPlacement>,
	cells: Vec<MargenCell>,
	bulk: WorldFacesBulk,
}

fn placements_to_c(placements: &ActorPlacements) -> Vec<MargenActorPlacement> {
	placements
		.iter()
		.map(|placement| {
			let location = placement
				.locations
				.first()
				.copied()
				.unwrap_or_else(Vector3i::zero);
			MargenActorPlacement {
				placeable_id: placement.placeable_id,
				location_x: location.x,
				location_y: location.y,
				location_z: location.z,
				biome_id: placement.biome_id,
				sector: placement.sector,
				_pad: 0,
			}
		})
		.collect()
}

fn cells_to_c(grid: &margen_generation::CellGrid) -> Vec<MargenCell> {
	let mut cells: Vec<_> = grid
		.iter()
		.map(|(loc, cell)| MargenCell {
			x: loc.x,
			y: loc.y,
			z: loc.z,
			attributes: cell.attributes,
			paths: cell.paths,
			biome_id: cell.biome_id(),
			sector: cell.sector,
			_pad: 0,
		})
		.collect();
	cells.sort_by_key(|c| (c.x, c.y, c.z));
	cells
}

impl MargenWorldFaces {
	fn from_parts(
		render_faces: RenderFaces,
		used_materials: UsedMaterials,
		player_start: Vector3i,
		placements: ActorPlacements,
		cells: Vec<MargenCell>,
	) -> Self {
		let mut materials: Vec<_> = used_materials.into_iter().collect();
		materials.sort_by_key(|(_, slot)| *slot);
		let bulk = Self::build_bulk(&render_faces);
		Self {
			render_faces,
			materials,
			player_start,
			placements: placements_to_c(&placements),
			cells,
			bulk,
		}
	}

	fn build_bulk(render_faces: &RenderFaces) -> WorldFacesBulk {
		let mut points = Vec::new();
		let mut uvs = Vec::new();
		let mut faces = Vec::with_capacity(render_faces.len());
		for face in render_faces {
			let point_begin = points.len() as u32;
			for p in &face.points {
				points.push(MargenVec3 {
					x: p.x,
					y: p.y,
					z: p.z,
				});
			}
			let uv_begin = uvs.len() as u32;
			for uv in &face.uvs {
				uvs.push(MargenVec2 { x: uv.x, y: uv.y });
			}
			faces.push(MargenBulkFaceDesc {
				point_begin,
				point_count: face.points.len() as u32,
				uv_begin,
				uv_count: face.uvs.len() as u32,
				location_x: face.location.x,
				location_y: face.location.y,
				location_z: face.location.z,
				material_id: face.material_id,
			});
		}
		WorldFacesBulk { points, uvs, faces }
	}
}

pub type MargenStatus = c_int;

pub const MARGEN_OK: MargenStatus = 0;
pub const MARGEN_ERR_INVALID_ARGUMENT: MargenStatus = 1;
pub const MARGEN_ERR_GENERATION_FAILED: MargenStatus = 2;

fn biomes_from_ids(biome_count: i32, biome_ids: *const u32) -> Result<StoryGridConfig, &'static str> {
	let mut config = StoryGridConfig::default();
	if biome_count < 0 {
		return Err("biome_count must be non-negative");
	}
	if biome_count == 0 {
		return Ok(config);
	}
	if biome_ids.is_null() {
		return Err("biome_ids is null while biome_count > 0");
	}
	let ids = unsafe { slice::from_raw_parts(biome_ids, biome_count as usize) };
	config.biomes = ids.iter().copied().map(biome_from_id).collect();
	Ok(config)
}

#[no_mangle]
pub extern "C" fn margen_version() -> *const c_char {
	static VERSION: &[u8] = b"0.1.0\0";
	VERSION.as_ptr().cast()
}

#[no_mangle]
pub extern "C" fn margen_last_error() -> *const c_char {
	LAST_ERROR.with(|slot| match slot.borrow().as_ref() {
		Some(msg) => msg.as_ptr(),
		None => ptr::null(),
	})
}

/// Generate a story grid. On success writes a non-null handle and returns `MARGEN_OK`.
///
/// Pass `biome_count == 0` and a null `biome_ids` for the default single biome.
#[no_mangle]
pub unsafe extern "C" fn margen_generate_story_grid(
	seed: i32,
	node_count: i16,
	origin_x: i32,
	origin_y: i32,
	origin_z: i32,
	biome_count: i32,
	biome_ids: *const u32,
	out_handle: *mut *mut MargenStoryGrid,
) -> MargenStatus {
	clear_last_error();
	if out_handle.is_null() {
		set_last_error("out_handle is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	if node_count < 0 {
		set_last_error("node_count must be non-negative");
		*out_handle = ptr::null_mut();
		return MARGEN_ERR_INVALID_ARGUMENT;
	}

	let config = match biomes_from_ids(biome_count, biome_ids) {
		Ok(config) => config,
		Err(message) => {
			set_last_error(message);
			*out_handle = ptr::null_mut();
			return MARGEN_ERR_INVALID_ARGUMENT;
		}
	};

	let mut dice = Dice::new(seed);
	let origin = Vector3i::new(origin_x, origin_y, origin_z);
	let result = generate_story_grid_with_config(&mut dice, node_count, origin, &config);
	if !result.success {
		set_last_error("story grid generation failed");
		*out_handle = ptr::null_mut();
		return MARGEN_ERR_GENERATION_FAILED;
	}

	let handle = Box::new(MargenStoryGrid::from_result(result));
	*out_handle = Box::into_raw(handle);
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_free(handle: *mut MargenStoryGrid) {
	if !handle.is_null() {
		drop(Box::from_raw(handle));
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_success(handle: *const MargenStoryGrid) -> c_int {
	if handle.is_null() {
		return 0;
	}
	if (*handle).result.success {
		1
	} else {
		0
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_cell_count(handle: *const MargenStoryGrid) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).cells.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_get_cell(
	handle: *const MargenStoryGrid,
	index: usize,
	out_cell: *mut MargenCell,
) -> MargenStatus {
	if handle.is_null() || out_cell.is_null() {
		set_last_error("null handle or out_cell");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let grid = &*handle;
	let Some((loc, cell)) = grid.cells.get(index) else {
		set_last_error("cell index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_cell = MargenCell {
		x: loc.x,
		y: loc.y,
		z: loc.z,
		attributes: cell.attributes,
		paths: cell.paths,
		biome_id: cell.biome_id(),
		sector: cell.sector,
		_pad: 0,
	};
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_portal_count(handle: *const MargenStoryGrid) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).portals.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_story_grid_get_portal(
	handle: *const MargenStoryGrid,
	index: usize,
	out_portal: *mut MargenPortal,
) -> MargenStatus {
	if handle.is_null() || out_portal.is_null() {
		set_last_error("null handle or out_portal");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let grid = &*handle;
	let Some(&(from, to)) = grid.portals.get(index) else {
		set_last_error("portal index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_portal = MargenPortal { from, to };
	MARGEN_OK
}

fn generate_world_faces_internal(seed: i32, max_blocks: i32) -> Result<MargenWorldFaces, &'static str> {
	if max_blocks < 1 {
		return Err("max_blocks must be at least 1");
	}
	let mut dice = Dice::new(seed);
	let config = WorldGridConfig {
		max_blocks,
		biomes: vec![surface_default_biome()],
		..Default::default()
	};
	let world = generate_world_grid(&mut dice, &config);
	let placements = populate_world(&PopulateInput {
		player_start: world.player_start,
	});
	let cells = cells_to_c(&world.grid);
	let corner_offsets = SharedCellVertexOffsets::new();
	let hex_map = apply_path_slopes(&world.grid);
	let (render_faces, used_materials) =
		generate_render_faces(&mut dice, &world.grid, &corner_offsets, &hex_map);
	if render_faces.is_empty() {
		return Err("world faces generation produced no faces");
	}
	Ok(MargenWorldFaces::from_parts(
		render_faces,
		used_materials,
		world.player_start,
		placements,
		cells,
	))
}

/// Generate a production world grid and Surfacing face IR (path slopes applied).
#[no_mangle]
pub unsafe extern "C" fn margen_generate_world_faces(
	seed: i32,
	max_blocks: i32,
	out_handle: *mut *mut MargenWorldFaces,
) -> MargenStatus {
	clear_last_error();
	if out_handle.is_null() {
		set_last_error("out_handle is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	match generate_world_faces_internal(seed, max_blocks) {
		Ok(faces) => {
			*out_handle = Box::into_raw(Box::new(faces));
			MARGEN_OK
		}
		Err(message) => {
			set_last_error(message);
			*out_handle = ptr::null_mut();
			if message == "max_blocks must be at least 1" {
				MARGEN_ERR_INVALID_ARGUMENT
			} else {
				MARGEN_ERR_GENERATION_FAILED
			}
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_free(handle: *mut MargenWorldFaces) {
	if !handle.is_null() {
		drop(Box::from_raw(handle));
	}
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_count(handle: *const MargenWorldFaces) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).render_faces.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_get_face(
	handle: *const MargenWorldFaces,
	face_index: usize,
	out_face: *mut MargenRenderFaceInfo,
) -> MargenStatus {
	if handle.is_null() || out_face.is_null() {
		set_last_error("null handle or out_face");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let faces = &(*handle).render_faces;
	let Some(face) = faces.get(face_index) else {
		set_last_error("face index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_face = MargenRenderFaceInfo {
		location_x: face.location.x,
		location_y: face.location.y,
		location_z: face.location.z,
		material_id: face.material_id,
		point_count: face.points.len() as u32,
		uv_count: face.uvs.len() as u32,
	};
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_get_point(
	handle: *const MargenWorldFaces,
	face_index: usize,
	point_index: usize,
	out_point: *mut MargenVec3,
) -> MargenStatus {
	if handle.is_null() || out_point.is_null() {
		set_last_error("null handle or out_point");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let faces = &(*handle).render_faces;
	let Some(face) = faces.get(face_index) else {
		set_last_error("face index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	let Some(point) = face.points.get(point_index) else {
		set_last_error("point index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_point = MargenVec3 {
		x: point.x,
		y: point.y,
		z: point.z,
	};
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_bulk(
	handle: *const MargenWorldFaces,
	out_points: *mut *const MargenVec3,
	out_point_count: *mut usize,
	out_uvs: *mut *const MargenVec2,
	out_uv_count: *mut usize,
	out_faces: *mut *const MargenBulkFaceDesc,
	out_face_count: *mut usize,
) -> MargenStatus {
	clear_last_error();
	if handle.is_null()
		|| out_points.is_null()
		|| out_point_count.is_null()
		|| out_uvs.is_null()
		|| out_uv_count.is_null()
		|| out_faces.is_null()
		|| out_face_count.is_null()
	{
		set_last_error("null bulk export argument");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let bulk = &(*handle).bulk;
	*out_points = bulk.points.as_ptr();
	*out_point_count = bulk.points.len();
	*out_uvs = bulk.uvs.as_ptr();
	*out_uv_count = bulk.uvs.len();
	*out_faces = bulk.faces.as_ptr();
	*out_face_count = bulk.faces.len();
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_get_uv(
	handle: *const MargenWorldFaces,
	face_index: usize,
	uv_index: usize,
	out_uv: *mut MargenVec2,
) -> MargenStatus {
	if handle.is_null() || out_uv.is_null() {
		set_last_error("null handle or out_uv");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let faces = &(*handle).render_faces;
	let Some(face) = faces.get(face_index) else {
		set_last_error("face index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	let Some(uv) = face.uvs.get(uv_index) else {
		set_last_error("uv index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_uv = MargenVec2 { x: uv.x, y: uv.y };
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_material_count(handle: *const MargenWorldFaces) -> usize {
	if handle.is_null() {
		return 0;
	}
	(*handle).materials.len()
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_get_material(
	handle: *const MargenWorldFaces,
	material_index: usize,
	out_material: *mut MargenMaterialSlot,
) -> MargenStatus {
	if handle.is_null() || out_material.is_null() {
		set_last_error("null handle or out_material");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let materials = &(*handle).materials;
	let Some(&(placeable_id, slot_index)) = materials.get(material_index) else {
		set_last_error("material index out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	};
	*out_material = MargenMaterialSlot {
		placeable_id,
		slot_index,
	};
	MARGEN_OK
}

#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_player_start(
	handle: *const MargenWorldFaces,
	out_x: *mut i32,
	out_y: *mut i32,
	out_z: *mut i32,
) -> MargenStatus {
	if handle.is_null() || out_x.is_null() || out_y.is_null() || out_z.is_null() {
		set_last_error("null handle or player_start output");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	*out_x = (*handle).player_start.x;
	*out_y = (*handle).player_start.y;
	*out_z = (*handle).player_start.z;
	MARGEN_OK
}

/// Bulk export of abstract actor placements. Pointers remain valid until
/// `margen_world_faces_free`. Hosts instantiate actors after mesh realization.
#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_placements_bulk(
	handle: *const MargenWorldFaces,
	out_placements: *mut *const MargenActorPlacement,
	out_placement_count: *mut usize,
) -> MargenStatus {
	clear_last_error();
	if handle.is_null() || out_placements.is_null() || out_placement_count.is_null() {
		set_last_error("null placements bulk export argument");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let placements = &(*handle).placements;
	*out_placements = placements.as_ptr();
	*out_placement_count = placements.len();
	MARGEN_OK
}

/// Bulk export of production grid cells (incl. path bits). Pointers remain valid
/// until `margen_world_faces_free`.
#[no_mangle]
pub unsafe extern "C" fn margen_world_faces_cells_bulk(
	handle: *const MargenWorldFaces,
	out_cells: *mut *const MargenCell,
	out_cell_count: *mut usize,
) -> MargenStatus {
	clear_last_error();
	if handle.is_null() || out_cells.is_null() || out_cell_count.is_null() {
		set_last_error("null cells bulk export argument");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let cells = &(*handle).cells;
	*out_cells = cells.as_ptr();
	*out_cell_count = cells.len();
	MARGEN_OK
}

/// Well-known placeable id for the player spawn location marker.
#[no_mangle]
pub extern "C" fn margen_placeable_player_spawn() -> u32 {
	PLACEABLE_PLAYER_SPAWN
}

#[no_mangle]
pub unsafe extern "C" fn margen_resolve_cell_center(
	location_x: i32,
	location_y: i32,
	location_z: i32,
	cell_size_x: f32,
	cell_size_y: f32,
	cell_size_z: f32,
	out_center: *mut MargenVec3,
) -> MargenStatus {
	if out_center.is_null() {
		set_last_error("out_center is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let location = Vector3i::new(location_x, location_y, location_z);
	let cell_size = Vector3::new(cell_size_x, cell_size_y, cell_size_z);
	let center = resolve_cell_coordinates_scaled(cell_size, location);
	*out_center = MargenVec3 {
		x: center.x,
		y: center.y,
		z: center.z,
	};
	MARGEN_OK
}

/// Unit face direction for path connection spokes (RH Z-up). `direction` in `[0, 20)`.
#[no_mangle]
pub unsafe extern "C" fn margen_resolve_direction_vector(
	direction: i32,
	out_direction: *mut MargenVec3,
) -> MargenStatus {
	if out_direction.is_null() {
		set_last_error("out_direction is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	if direction < 0 || direction >= ALL_DIRECTIONS_COUNT as i32 {
		set_last_error("direction out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let v = get_actual_direction_vector(direction as i8);
	*out_direction = MargenVec3 {
		x: v.x,
		y: v.y,
		z: v.z,
	};
	MARGEN_OK
}

/// Ideal hex prism corners (empty shared offsets) for cell-boundary visualization.
///
/// Writes [`HEX_CELL_CORNER_COUNT`] absolute positions into `out_corners`.
#[no_mangle]
pub unsafe extern "C" fn margen_resolve_default_cell_corners(
	location_x: i32,
	location_y: i32,
	location_z: i32,
	cell_size_x: f32,
	cell_size_y: f32,
	cell_size_z: f32,
	out_corners: *mut MargenVec3,
) -> MargenStatus {
	if out_corners.is_null() {
		set_last_error("out_corners is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let location = Vector3i::new(location_x, location_y, location_z);
	let cell_size = Vector3::new(cell_size_x, cell_size_y, cell_size_z);
	let empty = SharedCellVertexOffsets::new();
	let corners =
		resolve_absolute_cell_corner_coordinates(cell_size, cell_size * 0.5, &empty, location);
	let out = slice::from_raw_parts_mut(out_corners, HEX_CELL_CORNER_COUNT);
	for (dst, src) in out.iter_mut().zip(corners.iter()) {
		*dst = MargenVec3 {
			x: src.x,
			y: src.y,
			z: src.z,
		};
	}
	MARGEN_OK
}

/// Integer grid neighbor offset. `direction` in `[0, 20)`.
#[no_mangle]
pub unsafe extern "C" fn margen_resolve_grid_direction(
	direction: i32,
	out_offset: *mut MargenIVec3,
) -> MargenStatus {
	if out_offset.is_null() {
		set_last_error("out_offset is null");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	if direction < 0 || direction >= ALL_DIRECTIONS_COUNT as i32 {
		set_last_error("direction out of range");
		return MARGEN_ERR_INVALID_ARGUMENT;
	}
	let v = get_grid_direction_vector(direction as i8);
	*out_offset = MargenIVec3 {
		x: v.x,
		y: v.y,
		z: v.z,
	};
	MARGEN_OK
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ffi_story_grid_smoke() {
		unsafe {
			let mut handle: *mut MargenStoryGrid = ptr::null_mut();
			let status = margen_generate_story_grid(1, 5, 0, 0, 0, 0, ptr::null(), &mut handle);
			assert_eq!(status, MARGEN_OK);
			assert!(!handle.is_null());
			assert_eq!(margen_story_grid_success(handle), 1);
			assert!(margen_story_grid_cell_count(handle) > 5);
			assert_eq!(margen_story_grid_portal_count(handle), 0);

			let mut cell = MargenCell {
				x: 0,
				y: 0,
				z: 0,
				attributes: 0,
				paths: 0,
				biome_id: 0,
				sector: 0,
				_pad: 0,
			};
			assert_eq!(margen_story_grid_get_cell(handle, 0, &mut cell), MARGEN_OK);

			margen_story_grid_free(handle);
		}
	}

	#[test]
	fn ffi_multi_biome_smoke() {
		unsafe {
			let ids = [1u32, 2, 3];
			let mut handle: *mut MargenStoryGrid = ptr::null_mut();
			let status =
				margen_generate_story_grid(42, 8, 0, 0, 0, ids.len() as i32, ids.as_ptr(), &mut handle);
			assert_eq!(status, MARGEN_OK);
			assert!(!handle.is_null());
			let count = margen_story_grid_cell_count(handle);
			assert!(count > 0);
			let mut seen = std::collections::BTreeSet::new();
			let mut cell = MargenCell {
				x: 0,
				y: 0,
				z: 0,
				attributes: 0,
				paths: 0,
				biome_id: 0,
				sector: 0,
				_pad: 0,
			};
			for i in 0..count {
				assert_eq!(margen_story_grid_get_cell(handle, i, &mut cell), MARGEN_OK);
				seen.insert(cell.biome_id);
			}
			assert!(seen.len() > 1, "expected multiple biome ids, got {seen:?}");
			margen_story_grid_free(handle);
		}
	}

	#[test]
	fn version_is_nul_terminated() {
		let ptr = margen_version();
		assert!(!ptr.is_null());
	}

	#[test]
	fn ffi_world_faces_smoke() {
		unsafe {
			let mut handle: *mut MargenWorldFaces = ptr::null_mut();
			let status = margen_generate_world_faces(7, 12, &mut handle);
			assert_eq!(status, MARGEN_OK);
			assert!(!handle.is_null());
			let face_count = margen_world_faces_count(handle);
			assert!(face_count > 0);

			let mut face = MargenRenderFaceInfo {
				location_x: 0,
				location_y: 0,
				location_z: 0,
				material_id: -1,
				point_count: 0,
				uv_count: 0,
			};
			let mut has_material = false;
			let mut has_points = false;
			for i in 0..face_count {
				assert_eq!(margen_world_faces_get_face(handle, i, &mut face), MARGEN_OK);
				if face.material_id >= 0 {
					has_material = true;
				}
				if face.point_count > 0 {
					has_points = true;
					let mut point = MargenVec3 {
						x: 0.0,
						y: 0.0,
						z: 0.0,
					};
					assert_eq!(
						margen_world_faces_get_point(handle, i, 0, &mut point),
						MARGEN_OK
					);
				}
			}
			assert!(has_material);
			assert!(has_points);

			let mut points_ptr: *const MargenVec3 = ptr::null();
			let mut uvs_ptr: *const MargenVec2 = ptr::null();
			let mut faces_ptr: *const MargenBulkFaceDesc = ptr::null();
			let mut point_count = 0usize;
			let mut uv_count = 0usize;
			let mut bulk_face_count = 0usize;
			assert_eq!(
				margen_world_faces_bulk(
					handle,
					&mut points_ptr,
					&mut point_count,
					&mut uvs_ptr,
					&mut uv_count,
					&mut faces_ptr,
					&mut bulk_face_count,
				),
				MARGEN_OK
			);
			assert_eq!(bulk_face_count, face_count);
			assert!(!points_ptr.is_null());
			assert!(point_count > 0);
			assert!(!faces_ptr.is_null());

			let material_count = margen_world_faces_material_count(handle);
			assert!(material_count > 0);
			let mut slot = MargenMaterialSlot {
				placeable_id: 0,
				slot_index: -1,
			};
			assert_eq!(
				margen_world_faces_get_material(handle, 0, &mut slot),
				MARGEN_OK
			);
			assert!(slot.slot_index >= 0);

			let mut px = 0;
			let mut py = 0;
			let mut pz = 0;
			assert_eq!(
				margen_world_faces_player_start(handle, &mut px, &mut py, &mut pz),
				MARGEN_OK
			);

			let mut center = MargenVec3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			};
			assert_eq!(
				margen_resolve_cell_center(px, py, pz, 2.0, 2.0, 2.0, &mut center),
				MARGEN_OK
			);

			let mut placements_ptr: *const MargenActorPlacement = ptr::null();
			let mut placement_count = 0usize;
			assert_eq!(
				margen_world_faces_placements_bulk(
					handle,
					&mut placements_ptr,
					&mut placement_count,
				),
				MARGEN_OK
			);
			assert_eq!(placement_count, 1);
			assert!(!placements_ptr.is_null());
			let placement = &*placements_ptr;
			assert_eq!(placement.placeable_id, margen_placeable_player_spawn());
			assert_eq!(placement.location_x, px);
			assert_eq!(placement.location_y, py);
			assert_eq!(placement.location_z, pz);

			let mut cells_ptr: *const MargenCell = ptr::null();
			let mut cell_count = 0usize;
			assert_eq!(
				margen_world_faces_cells_bulk(handle, &mut cells_ptr, &mut cell_count),
				MARGEN_OK
			);
			assert!(cell_count > 0);
			assert!(!cells_ptr.is_null());
			let mut has_paths = false;
			for i in 0..cell_count {
				if (*cells_ptr.add(i)).paths != 0 {
					has_paths = true;
					break;
				}
			}
			assert!(has_paths, "expected at least one cell with path bits");

			let mut dir = MargenVec3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			};
			assert_eq!(margen_resolve_direction_vector(0, &mut dir), MARGEN_OK);
			assert!((dir.y - 1.0).abs() < 1e-5);

			let mut grid_dir = MargenIVec3 { x: 0, y: 0, z: 0 };
			assert_eq!(margen_resolve_grid_direction(0, &mut grid_dir), MARGEN_OK);
			assert_eq!(grid_dir, MargenIVec3 { x: 0, y: 1, z: 0 });

			let mut corners = [MargenVec3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			}; HEX_CELL_CORNER_COUNT];
			assert_eq!(
				margen_resolve_default_cell_corners(0, 0, 0, 2.0, 2.0, 2.0, corners.as_mut_ptr()),
				MARGEN_OK
			);
			let mut center = MargenVec3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			};
			assert_eq!(
				margen_resolve_cell_center(0, 0, 0, 2.0, 2.0, 2.0, &mut center),
				MARGEN_OK
			);
			assert!((corners[12].x - center.x).abs() < 1e-5);
			assert!((corners[12].y - center.y).abs() < 1e-5);
			assert!(corners[12].z < center.z);
			assert!(corners[13].z > center.z);
			for i in 0..6 {
				assert!((corners[i].z - corners[12].z).abs() < 1e-5);
				assert!((corners[i + 6].z - corners[13].z).abs() < 1e-5);
			}

			margen_world_faces_free(handle);
		}
	}

	#[test]
	fn ffi_grid_direction_rejects_out_of_range() {
		unsafe {
			let mut grid_dir = MargenIVec3 { x: 0, y: 0, z: 0 };
			assert_eq!(
				margen_resolve_grid_direction(-1, &mut grid_dir),
				MARGEN_ERR_INVALID_ARGUMENT
			);
			assert_eq!(
				margen_resolve_grid_direction(20, &mut grid_dir),
				MARGEN_ERR_INVALID_ARGUMENT
			);
		}
	}

	#[test]
	fn ffi_world_faces_rejects_invalid_max_blocks() {
		unsafe {
			let mut handle: *mut MargenWorldFaces = ptr::null_mut();
			let status = margen_generate_world_faces(1, 0, &mut handle);
			assert_eq!(status, MARGEN_ERR_INVALID_ARGUMENT);
			assert!(handle.is_null());
		}
	}
}
