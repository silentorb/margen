# Winding path (production grid growth)

Technical reference for Unreal’s **default production** layout path in **GenerationStructure**: minimal prefab seed → `windingPath` growth → `CellGrid`, then sector analysis.

For the story-graph alternate path, see [story-graph.md](story-graph.md). For path-derived sectors, see [sector-analysis.md](sector-analysis.md).

## Pipeline

```mermaid
flowchart LR
  seed[minimal_seed_block]
  grow[winding_path]
  grid[CellGrid plus player_start]
  analysis[partition_sectors]
  seed --> grow --> grid --> analysis
```

| Stage | Role |
|-------|------|
| Minimal seed | One `winding_path_block` at origin via `add_connected_block` (Unreal `UMinimalGenerator`). |
| `winding_path` | Grow by placing pathway blocks from location pools until `step > max_blocks`. |
| `generate_world_grid` | Facade: seed + growth; returns `{ grid, player_start, step }`. |

Public API: `generate_world_grid` / `generate_world_grid_default` in `crates/margen_generation_structure/src/structure/generate_world_grid.rs`. Growth loop: `winding_path` in `crates/margen_generation_structure/src/winding_path/growth.rs`.

Default config: `max_blocks=40`, `headroom=3`, single `default_biome()`. Multiple biomes use spatial `BiomeDistribution` (same pattern as story grid).

## Placement rules

Congruence gates (`can_place_block`):

1. **Space** — solid (or empty) below floor; empty headroom column of height `headroom`.
2. **Viable paths** — at least one neighbor connection that passes congruence.
3. **Flat conflicts** — no flat/slope adjacency violations.

Priority (`get_cell_priority`): biome-junction rule only (`LIMIT_CELL_PATHS` default off; RoomDistribution unused in current Unreal).

## Logic fix

Unreal `gatherAvailableLocations` built the solving cell from **`origin`** while the growth loop used **`location`**. Margen gather uses **`location`** (`incorrect-traversal-endpoint`).

## Deferred

| Piece | Notes |
|-------|-------|
| Home/room prefabs | Beyond minimal one-block seed |
| `GridRefinement` / `refineWorldGrid` | Organic Surfacing inputs (empty maps still yield prism faces) |
| RoomDistribution shaping | Dormant in Unreal `getCellPriority` |
| Population / face C ABI / Integration facade | Later stages (face C ABI **done** via `margen_generate_world_faces`) |
| Nested sub-biome sampling | Deferred with BiomeDistribution |

## Port status

| Piece | Status |
|-------|--------|
| `Block` / `add_connected_block` / connect-side congruence | Ported |
| Placement gates (`can_place_block`, biome `SolvingCell`) | Ported |
| Location pools / path management / biome-junction priority | Ported |
| `winding_path` growth loop | Ported |
| `generate_world_grid` (minimal seed) | Ported |
| Analysis smoke on winding grids | Ported (in analysis crate tests) |

Unreal reference: `GenerationStructure/WindingPath/`, `GenerationStructure/Structure/GenerateWorldGrid.cpp`, `GenerationIntegration/Specialization/RoomGenerator.cpp` (`UMinimalGenerator`).
