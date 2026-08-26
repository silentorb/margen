#ifndef MARGEN_H
#define MARGEN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MargenStoryGrid MargenStoryGrid;
typedef struct MargenWorldFaces MargenWorldFaces;

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

typedef struct MargenVec3 {
	float x;
	float y;
	float z;
} MargenVec3;

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

MargenStatus margen_resolve_cell_center(
	int32_t location_x,
	int32_t location_y,
	int32_t location_z,
	float cell_size_x,
	float cell_size_y,
	float cell_size_z,
	MargenVec3 *out_center);

#ifdef __cplusplus
}
#endif

#endif /* MARGEN_H */
