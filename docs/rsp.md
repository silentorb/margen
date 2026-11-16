# Relative Space Partitioning (RSP)

Experimental parallel path beside the hex-grid generation pipeline. Intent comes from the design journal (`docs/journal/2026-11-14.md`): define node graphs by **relative direction constraints**, defer absolute distance/scale until late realization.

## Crates

| Crate | Role |
|-------|------|
| `margen_rsp` | Core model: nodes with levels, radian angles, constraints, graph |
| `margen_rsp_generation` | Generators (skeletal seed + multi-form subdivision + metric layout) |

Neither crate depends on `margen_generation*`. Hosts consume results through `margen_ffi` / `include/margen.h`.

## Levels

Each node has an integer `level`: remaining subdivisions until the bottom (**0**). Greater values are coarser / higher level. The skeletal generator’s `subdivisions` argument sets the seed node’s level (default hosts use **2**).

Each node also stores an optional immediate `parent` id: the node it replaced during a split (`None` for the seed / roots). Parents are removed from the active graph after a pass but remain available via ancestry and history.

## Subdivision history

`generate_skeletal_rsp` / `subdivide_rsp_fully` return an [`RspHistory`](crates/margen_rsp/src/lib.rs) whose `layers` vector keeps a full graph snapshot per stage:

| Index | Contents |
|-------|----------|
| `[0]` | Pre-subdivision input (seed for skeletal) |
| `[1..]` | Graph after each subdivision pass |
| last | Level-0 leaves used for layout / FFI |

Later pipeline steps can read coarser layers; hosts that only need the final graph use `history.final_graph()`.

## Angle model (2D)

Constraint directions are planar **radians** in margen XY (RH Z-up):

| Angle | Meaning |
|-------|---------|
| `0` | +X east |
| `π/2` | +Y north |

An `RspConstraint` means **subject lies at `angle` relative to reference**. Connections and constraints are the same thing for now (traversable paths only). 3D / elevation constraints are out of scope.

## Subdivision forms

Each parent with `level > 0` is replaced by children at `level - 1` using one of three forms:

| Form | Children | Sibling edges |
|------|----------|---------------|
| **Binary** | Two on ± a random split axis | 1 |
| **Ternary open** | Three on a triangle (120° offsets, random base rotation) | 2 (one side omitted at random) |
| **Ternary closed** | Same triangle layout | 3 (closed loop) |

### Per-level form pools

Forms are **not** rolled independently per parent. At the start of each pass, parents are grouped by `level`. For each group of size `N`, a pool of `N` forms is built from **`loop_weight`** ∈ `[0, 1]`:

- **Ternary closed** share = `loop_weight`
- **Binary** and **Ternary open** each get `(1 - loop_weight) / 2`

Integer slot counts use largest-remainder (Hamilton) allocation; remainder ties break in Binary → TernaryOpen → TernaryClosed order. The pool is then shuffled and assigned one-to-one to that group’s parents.

Default `loop_weight` is **`2/3`** (`DEFAULT_LOOP_WEIGHT`) — double the original equal-share `1/3`. Hosts such as `MargenRspGraphView` expose this as an inspector property.

### Inheritance

When a parent is replaced, each of its connections is inherited by **only the nearest child**. Nearest is `argmax` of the dot product between the parent→other direction (from the existing constraint angle) and each child’s local unit offset—no global layout. Near-ties break to the lowest child id.

The inherited **angle is recomputed** from a local parent-edge embedding (reference parent at origin, subject parent at unit span along the old angle, children at parent ± offset · 0.5). Degenerate deltas fall back to the parent angle. New sibling edges still use angles derived from child offsets directly.

## Skeletal generator

`generate_skeletal_rsp(dice, subdivisions, loop_weight)` seeds one node (`id = 0`, `level = subdivisions.max(0)`, `parent = None`), then fully subdivides until all nodes are level 0, returning the full [`RspHistory`](#subdivision-history). Leaf count depends on depth and how many ternary splits appear. Absolute positions are **not** stored on the graph.

## Metric layout

`layout_rsp_metric_2d(graph, config)` produces a derived [`RspLayout`] (positions + stats) for pipeline and preview use:

- Initialize from a spanning-tree walk (`layout_rsp_spanning_tree_2d`)
- Soft angle bands: force only outside `angle_tolerance` around each constraint target
- Soft length springs toward `preferred_edge_length` (stretch allowed)
- Hard min-separation via repulsion and a final projection pass
- Stats: iterations, min pairwise distance, max angle residual (excess outside the deadband), mean edge length

Defaults match the historical preview spacing (`preferred_edge_length = 120`). The C ABI runs this layout when building an opaque `MargenRspGraph` and exports positions through `margen_rsp_graph_bulk`.

## C ABI

- `margen_generate_rsp_skeletal(seed, subdivisions, loop_weight, out_handle)` → opaque `MargenRspGraph` (final layer only)
- Bulk node ids, immediate parent ids (parallel; `UINT32_MAX` = no parent), constraints (`float angle`), and metric layout positions
- Free with `margen_rsp_graph_free`
- Full layer stacks are not exported over the ABI yet

## Out of scope (for now)

Per-edge / hierarchical angle tolerances, separate Binary vs TernaryOpen weight knobs, connect-all / distance-similarity inheritance thresholds, exposing layout config on `MargenRspGraphView`, 3D angles, and replacing the hex/`MargenWorldMesh` pipeline.
