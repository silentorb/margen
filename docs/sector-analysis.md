# Sector analysis (path-derived partitioning)

Technical reference for **GenerationGraphingAnalysis** (`margen_generation_graphing_analysis`): deriving `SectorData` from an expanded `CellGrid` via path-depth analysis, then optional goal/switch placement.

For the story-graph layout pipeline, see [story-graph.md](story-graph.md). For player-facing progression intent, see Marloth [story progression](../../marloth/docs/game/features/gameplay/story-progression.md).

## Pipeline

```mermaid
flowchart LR
  grid[CellGrid with path cells]
  neighbors[grid_to_neighbors_map]
  depths[map_path_depths]
  group[group_cells]
  sync[sync_sector_cells]
  propagate[propagate_sector_cells]
  headroom[calculate_headroom_clearance]
  goals[generate_goals]
  grid --> neighbors --> depths --> group --> sync --> propagate --> headroom
  headroom --> goals
```

Public composition API:

| Function | Role |
|----------|------|
| `partition_sectors` | Full analysis facade (`margen_generation_graphing_analysis`) |
| `generate_goals` | `sectors_to_dag` → gated waypoints → switches with cell locations (`goals.rs` in the analysis crate) |

## Story vs default paths

| Path | Sector assignment |
|------|-------------------|
| **Story graph** (`generate_story_grid`) | Unreal assigns one sector per story node inline; margen story API does not yet populate `SectorData`. Analysis can still run on story-grid output for testing. |
| **Default** (prefab + `windingPath`, not yet ported) | Unreal calls `partitionSectors` after grid growth. This is the production use case for the analysis layer. |

## Configuration

`PartitionConfig` mirrors optional Unreal DotEnv flags:

| Field | Unreal flag | Default |
|-------|-------------|---------|
| `cell_group_second_pass` | `CELL_GROUP_SECOND_PASS` | `false` |
| `cell_group_third_pass` | `CELL_GROUP_THIRD_PASS` | `false` |

Production Unreal behavior uses **first pass only** plus `connect_neighbor_sectors`.

## Types

- `NeighborsMap` — path cell → neighbor path locations (`margen_generation_structure::querying`)
- `PathDepths` — BFS depth from `player_start` over path connectivity
- `SectorData` — sectors, cell map, origin (`margen_generation::core::sector`)
- `LevelSwitchEntry` — source/target sector pair with switch cell location

## Port status

| Piece | Status |
|-------|--------|
| `grid_to_neighbors_map`, `get_path_locations` | Ported |
| `map_path_depths` | Ported |
| `group_cells` (+ optional 2nd/3rd pass) | Ported |
| `propagate_sector_cells`, headroom clearance, validation | Ported |
| `partition_sectors` | Ported |
| `generate_goals` | Ported |
| C ABI exports for analysis / switches | Deferred |

Unreal reference: `GenerationGraphingAnalysis/` under workspace **`unreal-marloth`**.
