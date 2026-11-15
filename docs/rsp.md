# Relative Space Partitioning (RSP)

Experimental parallel path beside the hex-grid generation pipeline. Intent comes from the design journal (`docs/journal/2026-11-14.md`): define node graphs by **relative direction constraints**, defer absolute distance/scale until late realization.

## Crates

| Crate | Role |
|-------|------|
| `margen_rsp` | Core model: nodes with levels, radian angles, constraints, graph |
| `margen_rsp_generation` | Generators (skeletal seed + multi-form subdivision + debug layout) |

Neither crate depends on `margen_generation*`. Hosts consume results through `margen_ffi` / `include/margen.h`.

## Levels

Each node has an integer `level`: remaining subdivisions until the bottom (**0**). Greater values are coarser / higher level. The skeletal generator’s `subdivisions` argument sets the seed node’s level (default hosts use **2**).

## Angle model (2D)

Constraint directions are planar **radians** in margen XY (RH Z-up):

| Angle | Meaning |
|-------|---------|
| `0` | +X east |
| `π/2` | +Y north |

An `RspConstraint` means **subject lies at `angle` relative to reference**. Connections and constraints are the same thing for now (traversable paths only). 3D / elevation constraints are out of scope.

## Subdivision forms

Each parent with `level > 0` is replaced by children at `level - 1` using one of three forms (equal weight for now):

| Form | Children | Sibling edges |
|------|----------|---------------|
| **Binary** | Two on ± a random split axis | 1 |
| **Ternary open** | Three on a triangle (120° offsets, random base rotation) | 2 (one side omitted at random) |
| **Ternary closed** | Same triangle layout | 3 (closed loop) |

### Per-level form pools

Forms are **not** rolled independently per parent. At the start of each pass, parents are grouped by `level`. For each group of size `N`, a pool of `N` forms is built with equal weights (counts differ by at most one; remainder goes to forms in Binary → TernaryOpen → TernaryClosed order), shuffled, and assigned one-to-one to that group’s parents.

### Inheritance

When a parent is replaced, each of its connections is inherited by **only the nearest child**. Nearest is `argmax` of the dot product between the parent→other direction (from the existing constraint angle) and each child’s local unit offset—no global layout. Near-ties break to the lowest child id. Rewired edges keep the parent constraint angle; only new sibling edges use angles derived from child offsets.

## Skeletal generator

`generate_skeletal_rsp(dice, subdivisions)` seeds one node (`id = 0`, `level = subdivisions.max(0)`), then fully subdivides until all nodes are level 0. Leaf count depends on depth and how many ternary splits appear. Absolute placement is not part of the graph; the C ABI also exports a trivial **debug layout** (`MargenVec2` per node) for editor drawing only (spanning-tree placement so cycles do not thrash positions).

## C ABI

- `margen_generate_rsp_skeletal(seed, subdivisions, out_handle)` → opaque `MargenRspGraph`
- Bulk node ids, constraints (`float angle`), and debug layout positions
- Free with `margen_rsp_graph_free`

## Out of scope (for now)

Absolute metric realization, configurable form weights / inspector UI for weights, connect-all / distance-similarity inheritance thresholds, 3D angles, and replacing the hex/`MargenWorldMesh` pipeline.
