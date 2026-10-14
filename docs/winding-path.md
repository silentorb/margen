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

## Forbidden neighboring path combinations

Slope sculpting needs each shared upper corner to have one coherent height demand. A path bit with yaw `y` claims hex edge corners `(y+5)%6` and `y`, at destination height Δz ∈ {−1, 0, +1}.

**Geometric rule (runtime):** two path bits on the same cell conflict when their edges **share a corner** and their **destination Z differs** (`CongruenceError::SharedCornerPitch`). Encoded by `paths_conflict_on_shared_corner` in `congruence.rs`.

| # | Combination | Gate / notes |
|---|-------------|--------------|
| 1 | Non-neighbor upward pair (yaw Δ ∈ {2,3}) | `UpwardSlope` (`supports_new_upward_slope`) |
| 2 | New up vs down in proposed yaw neighborhood | `UpwardSlope` |
| 3 | New down vs existing up at yaw ±1 | `NeighboringUpward` (kept; often also caught by #10–11) |
| 4 | Flat attribute + any slope/vertical pitch | `Flat` |
| 5 | Non-path endpoint | `NoPath` |
| 6 | Flat over lower path / path over lower flat | `cell_has_no_flat_slope_conflicts` |
| 7 | Headroom / non-solid floor | `block_space_is_available` |
| 8 | Multi-link connect order | earlier bit can invalidate a later link |
| 9 | Wrap near-miss closing a conflicting slope | must stay disconnected (`InvalidCurving`-class) |
| 10 | Same yaw, different pitch (e.g. `NORTH` + `UP_NORTH`) | `SharedCornerPitch` |
| 11 | Adjacent yaw, different pitch (e.g. `NORTH` + `UP_NORTHEAST`) | `SharedCornerPitch` |
| 12 | Same-yaw up+down | `SharedCornerPitch` (symmetric) |

**Allowed:** adjacent-yaw upward fan (`UP_NORTH` + `UP_NORTHEAST`) — both Δz=+1; shared corner wants high from both sides.

**Sculpting residual (not a placement CongruenceError):** corner-column vertical-gap bleeding on spiral layouts — covered by slope Z congruity suite tests.

Suite coverage: exhaustive same-cell bit-pair matrix in `congruence.rs` tests; slope upper-ring patterns and shared-vertex Z congruity in `slopes.rs` tests.

## Logic fix

Unreal `gatherAvailableLocations` built the solving cell from **`origin`** while the growth loop used **`location`**. Margen gather uses **`location`** (`incorrect-traversal-endpoint`).

## Deferred

| Piece | Notes |
|-------|-------|
| Home/room prefabs | Beyond minimal one-block seed |
| Wall shift / prune / expansion / full `refineWorldGrid` | Path slopes **ported** (`apply_path_slopes`); rest of organic refinement later |
| RoomDistribution shaping | Dormant in Unreal `getCellPriority` |
| Population / Integration facade | Later stages (face C ABI **done** via `margen_generate_world_faces`) |
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
