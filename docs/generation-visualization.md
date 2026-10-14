# Generation visualization

Editor tooling visualizes world-generation datasets so developers can diagnose path connectivity, structure, and intermediate samples. Margen stays mostly oblivious: it **gathers** production (and optional debug) data; hosts **draw**.

Unreal reference: `Source/Editing/Visualization/` (especially `WorldVisualizer.cpp`, `AnnotationGraphics.cpp`, `GenerationVisualizationConfig.h`). Host for Godot: sibling **margen-godot**.

## Gather vs draw

| Layer | Role |
|-------|------|
| **margen** | Export cells / path bits / (later) optional debug bags via the C ABI |
| **Host** (margen-godot) | Annotation bitflags UI + viewport drawing (e.g. RenderingServer lines) |

Same split as profiling: samples in the target program, visualization elsewhere.

**Basic path spokes** use production `WorldCell.paths` — not a debug-only bag. Optional Unreal-style bags (`GenerationDebugInfo`, `GenerationStructureDebugInfo`) are for priorities, expansions, slope corners, wall spans, etc., and remain deferred.

## Two corner channels (easy to confuse)

Surfacing resolves faces from **two** independent maps:

| Channel | Type | Filled today? | Unreal viz flag |
|---------|------|---------------|-----------------|
| Shared vertex offsets | `SharedCellVertexOffsets` | Always empty in world-faces (narrow walls / wall shift deferred) | `cornerOffsets` (3D default→actual rays) |
| Path-slope Z scales | `HexCellCornersMap` via `apply_path_slopes` | Yes — deforms mesh slopes | `slopeCornerPaths` (HUD counts; not 3D Z rays) |

Ideal **cellBoundaries** wires intentionally use **empty** shared offsets (undeformed hex prism). They do **not** follow slope-scaled mesh edges.

## Path spokes

Contract (mirrors Unreal `drawPaths`):

1. For each cell, for each direction `i` in `0..18` (`HORIZONTAL_DIRECTION_COUNT`) where `paths & (1 << i)`:
2. Line from cell center to `center + get_actual_direction_vector(i) * cell_size * 0.45`, with a small Z lift.
3. Flat (`i < 6`): green; slopes: teal.
4. Half-stubs toward connections — not full center-to-neighbor chords.

Data path: `margen_generate_world_faces` retains cells; hosts call `margen_world_faces_cells_bulk` plus `margen_resolve_cell_center` / `margen_resolve_direction_vector`.

Godot: `MargenWorldMesh.annotations` bitflags (`PROPERTY_HINT_FLAGS`); `paths` bit toggles spokes drawn via RenderingServer RIDs (no scene-tree mesh node). Host bit layout is independent of Unreal’s enum values (`Paths = 1<<0`).

## Cell boundaries

Contract (mirrors Unreal `drawCellBoundaries`):

1. Resolve 14 absolute corners via `margen_resolve_default_cell_corners` (empty shared offsets — ideal prism).
2. For each flat side `i` in `0..6`:
   - Lower ring edge `corners[i] → corners[(i+1)%6]` if no cell at `location + (0,0,-1)`.
   - Vertical `corners[i] → corners[i+6]` if both `location + gridDir(i)` and `location + gridDir((i+1)%6)` are missing (`margen_resolve_grid_direction`).
3. Do **not** draw the upper ring (commented out in Unreal).
4. Color: blue. No sector offset.
5. Host batches all segments into **one** `PRIMITIVE_LINES` mesh RID (same pattern as path spokes).

Godot bit: `CellBoundaries = 1<<1`.

## Unreal inventory (summary)

| Piece | Role |
|-------|------|
| `FWorldVisualizer` | Editor component visualizer entry |
| `AnnotationGraphics` | 3D drawers (`drawPaths`, boundaries, …) |
| `AnnotationText` | HUD labels |
| `GenerationAnnotation` | Bitflags (`paths`, `pathDepth`, …) |
| `UWorldDebugger` / `AWorldPreview` | Hold bundle + viz config |
| `GenerationDebugInfo` / structure debug | Optional editor-only samples |

## Annotation backlog

| Flag (Unreal) | Needs | Status |
|---------------|-------|--------|
| `paths` | Cell path bits + direction vectors | **Done** |
| `cellBoundaries` | Ideal hex wire + grid neighbor probes | **Done** |
| `cornerOffsets` | Filled `SharedCellVertexOffsets` | Deferred (map empty until wall-shift) |
| `slopeCornerPaths` | Structure debug bag + HUD | Deferred |
| `pathDepth` / HUD text | Depth map + labels | Deferred |
| `pathExpansions` / priorities / wall spans | Optional debug bag | Deferred |
| `cornerColumns` | Structure debug bag | Deferred |
| Filters / `GridCache` / hit selection | Host UX | Deferred |

## Future optional debug bags

When needed, mirror Unreal: allocate a nullable sidecar only when a collect flag is on; append at step ends; export via a separate bulk ABI. Do not require that channel for production path bits.
