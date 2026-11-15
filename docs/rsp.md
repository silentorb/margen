# Relative Space Partitioning (RSP)

Experimental parallel path beside the hex-grid generation pipeline. Intent comes from the design journal (`docs/journal/2026-11-14.md`): define node graphs by **relative direction constraints**, defer absolute distance/scale until late realization.

## Crates

| Crate | Role |
|-------|------|
| `margen_rsp` | Core model: nodes with levels, radian angles, constraints, graph |
| `margen_rsp_generation` | Generators (skeletal seed + binary subdivision + debug layout) |

Neither crate depends on `margen_generation*`. Hosts consume results through `margen_ffi` / `include/margen.h`.

## Levels

Each node has an integer `level`: remaining subdivisions until the bottom (**0**). Greater values are coarser / higher level.

## Angle model (2D)

Constraint directions are planar **radians** in margen XY (RH Z-up):

| Angle | Meaning |
|-------|---------|
| `0` | +X east |
| `π/2` | +Y north |

An `RspConstraint` means **subject lies at `angle` relative to reference**. Connections and constraints are the same thing for now (traversable paths only). 3D / elevation constraints are out of scope.

## Subdivision

`subdivide_rsp_pass` replaces every node with `level > 0` by **two** children at `level - 1`, linked by a new sibling constraint at a **random** angle. `subdivide_rsp_fully` repeats passes until all nodes are level 0.

When a parent is replaced, each of its connections is inherited by **only the nearest child**. Nearest is chosen with a local **dot product** (split axis vs parent→other direction implied by the existing constraint angle)—no global layout. Rewired edges keep the parent constraint angle; only the new sibling edge uses the random split angle.

## Skeletal generator

`generate_skeletal_rsp(dice)` seeds one node (`id = 0`, `level = 2`), then fully subdivides → **four** level-0 nodes. Absolute placement is not part of the graph; the C ABI also exports a trivial **debug layout** (`MargenVec2` per node) for editor drawing only.

## C ABI

- `margen_generate_rsp_skeletal` → opaque `MargenRspGraph` (fixed `Dice` seed `0`)
- Bulk node ids, constraints (`float angle`), and debug layout positions
- Free with `margen_rsp_graph_free`

## Out of scope (for now)

Absolute metric realization, loops, connect-all / distance-similarity inheritance thresholds, 3D angles, and replacing the hex/`MargenWorldMesh` pipeline.
