#ifndef MARGEN_H
#define MARGEN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MargenStoryGrid MargenStoryGrid;

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

const char *margen_version(void);
const char *margen_last_error(void);

MargenStatus margen_generate_story_grid(
	int32_t seed,
	int16_t node_count,
	int32_t origin_x,
	int32_t origin_y,
	int32_t origin_z,
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

#ifdef __cplusplus
}
#endif

#endif /* MARGEN_H */
