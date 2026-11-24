#ifndef MARGEN_H
#define MARGEN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MargenStoryGrid MargenStoryGrid;
typedef struct MargenWorldFaces MargenWorldFaces;
typedef struct MargenRspGraph MargenRspGraph;

typedef int MargenStatus;

enum {
	MARGEN_OK = 0,
	MARGEN_ERR_INVALID_ARGUMENT = 1,
	MARGEN_ERR_GENERATION_FAILED = 2
};

typedef struct MargenCell {
	int32_t x;
	int32_t y;
	int32_t z;
	int64_t attributes;
	uint32_t paths;
	uint32_t biome_id;
	int16_t sector;
	int16_t _pad;
} MargenCell;

typedef struct MargenPortal {
	int16_t from;
	int16_t to;
} MargenPortal;

typedef struct MargenVec2 {
	float x;
	float y;
} MargenVec2;

/* Right-handed Z-up: +X right, +Y north/forward, +Z up. Hosts convert at the boundary. */
typedef struct MargenVec3 {
	float x;
	float y;
	float z;
} MargenVec3;

typedef struct MargenIVec3 {
	int32_t x;
	int32_t y;
	int32_t z;
} MargenIVec3;

enum {
	MARGEN_HEX_CELL_CORNER_COUNT = 14
};

typedef struct MargenRenderFaceInfo {
	int32_t location_x;
	int32_t location_y;
	int32_t location_z;
	int32_t material_id;
	uint32_t point_count;
	uint32_t uv_count;
} MargenRenderFaceInfo;

typedef struct MargenMaterialSlot {
	uint32_t placeable_id;
	int32_t slot_index;
} MargenMaterialSlot;

/* Contiguous face descriptor for bulk buffer export (indices into points/uvs). */
typedef struct MargenBulkFaceDesc {
	uint32_t point_begin;
	uint32_t point_count;
	uint32_t uv_begin;
	uint32_t uv_count;
	int32_t location_x;
	int32_t location_y;
	int32_t location_z;
	int32_t material_id;
} MargenBulkFaceDesc;

/* Abstract actor placement (first cell location). Hosts instantiate after mesh apply. */
typedef struct MargenActorPlacement {
	uint32_t placeable_id;
	int32_t location_x;
	int32_t location_y;
	int32_t location_z;
	uint32_t biome_id;
	int16_t sector;
	int16_t _pad;
} MargenActorPlacement;

enum {
	MARGEN_PLACEABLE_PLAYER_SPAWN = 1
};

const char *margen_version(void);
const char *margen_last_error(void);

/* biome_ids may be NULL when biome_count is 0 (defaults to a single biome id 0). */
MargenStatus margen_generate_story_grid(
	int32_t seed,
	int16_t node_count,
	int32_t origin_x,
	int32_t origin_y,
	int32_t origin_z,
	int32_t biome_count,
	const uint32_t *biome_ids,
	MargenStoryGrid **out_handle);

void margen_story_grid_free(MargenStoryGrid *handle);
int margen_story_grid_success(const MargenStoryGrid *handle);
size_t margen_story_grid_cell_count(const MargenStoryGrid *handle);
MargenStatus margen_story_grid_get_cell(
	const MargenStoryGrid *handle,
	size_t index,
	MargenCell *out_cell);
size_t margen_story_grid_portal_count(const MargenStoryGrid *handle);
MargenStatus margen_story_grid_get_portal(
	const MargenStoryGrid *handle,
	size_t index,
	MargenPortal *out_portal);

/* One-shot production grid + Surfacing face IR (empty corner maps). */
MargenStatus margen_generate_world_faces(
	int32_t seed,
	int32_t max_blocks,
	MargenWorldFaces **out_handle);

void margen_world_faces_free(MargenWorldFaces *handle);
size_t margen_world_faces_count(const MargenWorldFaces *handle);
MargenStatus margen_world_faces_get_face(
	const MargenWorldFaces *handle,
	size_t face_index,
	MargenRenderFaceInfo *out_face);
MargenStatus margen_world_faces_get_point(
	const MargenWorldFaces *handle,
	size_t face_index,
	size_t point_index,
	MargenVec3 *out_point);
MargenStatus margen_world_faces_get_uv(
	const MargenWorldFaces *handle,
	size_t face_index,
	size_t uv_index,
	MargenVec2 *out_uv);

/*
 * Bulk export: contiguous points/uvs plus face descriptors. Pointers remain valid
 * until margen_world_faces_free. Prefer this over per-point getters for hosts.
 */
MargenStatus margen_world_faces_bulk(
	const MargenWorldFaces *handle,
	const MargenVec3 **out_points,
	size_t *out_point_count,
	const MargenVec2 **out_uvs,
	size_t *out_uv_count,
	const MargenBulkFaceDesc **out_faces,
	size_t *out_face_count);

size_t margen_world_faces_material_count(const MargenWorldFaces *handle);
MargenStatus margen_world_faces_get_material(
	const MargenWorldFaces *handle,
	size_t material_index,
	MargenMaterialSlot *out_material);
MargenStatus margen_world_faces_player_start(
	const MargenWorldFaces *handle,
	int32_t *out_x,
	int32_t *out_y,
	int32_t *out_z);

/*
 * Bulk export of abstract actor placements. Pointers remain valid until
 * margen_world_faces_free. Prefer this over chatty getters.
 */
MargenStatus margen_world_faces_placements_bulk(
	const MargenWorldFaces *handle,
	const MargenActorPlacement **out_placements,
	size_t *out_placement_count);

/*
 * Bulk export of production grid cells (incl. path bits). Pointers remain valid
 * until margen_world_faces_free.
 */
MargenStatus margen_world_faces_cells_bulk(
	const MargenWorldFaces *handle,
	const MargenCell **out_cells,
	size_t *out_cell_count);

uint32_t margen_placeable_player_spawn(void);

MargenStatus margen_resolve_cell_center(
	int32_t location_x,
	int32_t location_y,
	int32_t location_z,
	float cell_size_x,
	float cell_size_y,
	float cell_size_z,
	MargenVec3 *out_center);

/* Unit face direction for path spokes (RH Z-up). direction in [0, 20). */
MargenStatus margen_resolve_direction_vector(
	int32_t direction,
	MargenVec3 *out_direction);

/*
 * Ideal hex prism corners (empty SharedCellVertexOffsets) for cell-boundary wires.
 * Writes MARGEN_HEX_CELL_CORNER_COUNT absolute positions into out_corners.
 */
MargenStatus margen_resolve_default_cell_corners(
	int32_t location_x,
	int32_t location_y,
	int32_t location_z,
	float cell_size_x,
	float cell_size_y,
	float cell_size_z,
	MargenVec3 *out_corners);

/* Grid neighbor offset for flat directions 0..5 (and full table 0..19). */
MargenStatus margen_resolve_grid_direction(
	int32_t direction,
	MargenIVec3 *out_offset);

/* --- Relative Space Partitioning (RSP); parallel to hex pipeline --- */

/*
 * Planar constraint angle in radians (margen XY): 0 = +X east, pi/2 = +Y north.
 * Subject lies at `angle` relative to reference.
 */
typedef struct MargenRspConstraint {
	uint32_t subject;
	uint32_t reference;
	float angle;
} MargenRspConstraint;

/*
 * Skeletal seed: one node at `subdivisions` level (clamped >= 0), fully
 * subdivided to level-0 leaves. `seed` feeds Dice; leaf count varies with depth
 * and ternary split forms. `loop_weight` in [0, 1] is the TernaryClosed share of
 * each per-level form pool (default hosts use 2/3); Binary and TernaryOpen split
 * the remainder equally. Non-finite values fall back to 2/3 inside the generator.
 */
MargenStatus margen_generate_rsp_skeletal(
	int32_t seed,
	int32_t subdivisions,
	float loop_weight,
	MargenRspGraph **out_handle);
void margen_rsp_graph_free(MargenRspGraph *handle);

/*
 * Bulk node ids (sorted), immediate parent ids (parallel to node ids;
 * UINT32_MAX = no parent / root), constraints, and metric layout positions
 * (margen XY, +Y north; parallel to node ids). Layout relaxes each point with
 * soft angle bands and stretchable lengths while keeping it strictly inside its
 * node's cell (see margen_rsp_graph_geometry). Pointers valid until
 * margen_rsp_graph_free.
 */
MargenStatus margen_rsp_graph_bulk(
	const MargenRspGraph *handle,
	const uint32_t **out_node_ids,
	const uint32_t **out_parent_ids,
	size_t *out_node_count,
	const MargenRspConstraint **out_constraints,
	size_t *out_constraint_count,
	const MargenVec2 **out_layout,
	size_t *out_layout_count);

/*
 * Partition geometry (same margen XY space as the layout):
 * - doorways: parallel to constraints; each edge is drawn
 *   subject -> doorway -> reference, which never overlaps another edge.
 * - cells: parallel to node ids; convex counter-clockwise polygons with disjoint
 *   interiors, read as cell_vertices[cell_offsets[i] .. + cell_counts[i]].
 * Pointers valid until margen_rsp_graph_free.
 */
MargenStatus margen_rsp_graph_geometry(
	const MargenRspGraph *handle,
	const MargenVec2 **out_doorways,
	size_t *out_doorway_count,
	const uint32_t **out_cell_offsets,
	const uint32_t **out_cell_counts,
	size_t *out_cell_count,
	const MargenVec2 **out_cell_vertices,
	size_t *out_cell_vertex_count);

#ifdef __cplusplus
}
#endif

#endif /* MARGEN_H */
