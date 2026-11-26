# Relative Space Partitioning (RSP)

Experimental parallel path beside the hex-grid generation pipeline. Intent comes from the design journal (`docs/journal/2026-11-14.md`): define node graphs by **relative direction constraints**, defer absolute distance/scale until late realization.

## Crates

| Crate | Role |
|-------|------|
| `margen_rsp` | Core model: nodes with levels, radian angles, constraints, graph, partition (cells / points / doorways) |
| `margen_rsp_generation` | Generators (skeletal seed + cell-partition subdivision + contained metric layout + embedding validator) |
| `margen_rsp_skinning` | Floor slabs: rooms and pathways fitted into the partition with margins, plus a per-piece skinner seam ([rsp-skinning.md](rsp-skinning.md)) |

None of these crates depends on `margen_generation*`. Hosts consume results through `margen_ffi` / `include/margen.h`.

## Levels

Each node has an integer `level`: remaining subdivisions until the bottom (**0**). Greater values are coarser / higher level. The skeletal generator’s `subdivisions` argument sets the seed node’s level (default hosts use **2**).

Each node also stores an optional immediate `parent` id: the node it replaced during a split (`None` for the seed / roots). Parents are removed from the active graph after a pass but remain available via ancestry and history.

## Subdivision history

`generate_skeletal_rsp` / `subdivide_rsp_fully` return an [`RspHistory`](crates/margen_rsp/src/lib.rs) whose `layers` vector keeps a full snapshot per stage. Each `RspLayer` is the relative `RspGraph` plus its `RspPartition` (cells, node points, doorways; see [Cells and doorways](#cells-and-doorways)):

| Index | Contents |
|-------|----------|
| `[0]` | Pre-subdivision input (seed for skeletal) |
| `[1..]` | Layer after each subdivision pass |
| last | Level-0 leaves used for layout / FFI |

Later pipeline steps can read coarser layers; hosts that only need the final layer use `history.final_layer()` (or `history.final_graph()` for the graph alone).

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
| **Binary** | Parent cell cut by a line through its centroid at a random angle | 1 |
| **Ternary open** | Parent cell cut by three rays at 120° from its centroid (random base rotation) | 2 (one side omitted at random) |
| **Ternary closed** | Same three wedges | 3 (closed loop) |

### Per-level form pools

Forms are **not** rolled independently per parent. At the start of each pass, parents are grouped by `level`. For each group of size `N`, a pool of `N` forms is built from **`loop_weight`** ∈ `[0, 1]`:

- **Ternary closed** share = `loop_weight`
- **Binary** and **Ternary open** each get `(1 - loop_weight) / 2`

Integer slot counts use largest-remainder (Hamilton) allocation; remainder ties break in Binary → TernaryOpen → TernaryClosed order. The pool is then shuffled and assigned one-to-one to that group’s parents.

Default `loop_weight` is **`2/3`** (`DEFAULT_LOOP_WEIGHT`) — double the original equal-share `1/3`. Hosts such as `MargenRspGraphView` expose this as an inspector property.

### Inheritance

When a parent is replaced, each of its connections is inherited by **exactly one child: the one whose cell holds the edge’s doorway**. When the doorway lies exactly on a new split wall, both children hold it and the lowest child id wins. The doorway itself is unchanged.

Every angle is the direction between the endpoint **node points** (cell centroids during subdivision), so the relative constraints always describe a drawing that actually exists.

## Cells and doorways

Children are confined to bounds inside their parent at every level. The seed owns a unit square **cell**. Each split divides the parent’s cell into 2 or 3 convex child cells, so a leaf’s cell lies inside its parent’s, which lies inside its grandparent’s, and so on up to the seed. Each edge has a **doorway**: a point on the wall shared by its two endpoint cells. The edge is drawn `subject → doorway → reference`.

### Invariants (every layer)

1. **I1 (partition):** cells are convex (counter-clockwise), have positive area and disjoint interiors, and tile the seed square.
2. **I2 (node inside):** each node point lies strictly inside its own cell.
3. **I3 (doorway on wall):** each constraint has one doorway, stored as a vertex of both endpoint cells, and no two constraints share a doorway.

### Why nothing can overlap

If I1–I3 hold, no two nodes coincide, and two edge drawings meet only at a node they share.

- A half-edge `[node point, doorway]` runs from an interior point to a boundary point of a convex cell. Every point of it except the doorway is therefore in the cell’s interior.
- **Halves in different cells:** the interiors are disjoint, so the halves can meet only at boundary points. Those are just their doorways, which differ (I3) unless both halves belong to the same edge.
- **Halves in the same cell:** both start at the node point. They overlap only if they point the same way. A ray from an interior point leaves a convex cell exactly once, so that would mean the same doorway, which I3 rules out.
- **Node points:** each is interior to its own cell (I2). It lies on no other cell’s half-edges, and on its own only as their start point.

### Why splits keep the invariants

The proof is by induction from the seed, which is one cell with no edges.

- **Binary:** a line through the centroid yields two convex halves. Each keeps at least 4/9 of the parent’s area (Grünbaum), so neither is degenerate. The sibling doorway is the centroid itself: it lies on the wall and strictly inside the parent.
- **Ternary:** each child is the parent intersected with a 120° wedge (two half-planes), so it is convex. Its interior is non-empty because the centroid is interior. The sibling doorways are the wall midpoints, inserted as vertices of both adjacent children.
- **Inherited doorways:** these are parent vertices, and splits copy boundary vertices into children verbatim. Each ray’s exit point is computed once and shared by both adjacent wedges. Ownership is therefore decided by construction (a bit-identical vertex), never by a separate point-in-polygon test.
- **New doorways** lie strictly inside the parent. Every older doorway lies on the parent’s boundary or outside it, so the new ones are distinct.
- **Child node points** are cell centroids, which are strictly interior.

The proof is stated in exact arithmetic. Geometry runs in `f64`, and `validate_rsp_embedding` re-checks I1–I3 and edge conflicts independently, using a clearance tolerance of 1e-9 × the partition diagonal. Tests sweep seeds × depths × `loop_weight`.

**Outcome change:** the cell-based splits replace the earlier fixed-offset embedding, so seeded graphs differ from those of the previous algorithm.

## Skeletal generator

`generate_skeletal_rsp(dice, subdivisions, loop_weight)` seeds one node (`id = 0`, `level = subdivisions.max(0)`, `parent = None`), then fully subdivides until all nodes are level 0, returning the full [`RspHistory`](#subdivision-history). Leaf count depends on depth and how many ternary splits appear. The relative `RspGraph` stores no positions; the geometry lives beside it in each layer’s `RspPartition`, in the scale-free seed square.

## Metric layout

`layout_rsp_metric_2d(layer, config)` produces a derived `RspLayout` (the realized layer + stats) for pipeline and preview use. Every step preserves I1–I3:

1. **Scale** the whole partition uniformly so the mean straight edge length equals `preferred_edge_length`.
2. **Relax** node points from their cell centroids:
   - soft angle bands (force only outside `angle_tolerance`);
   - length springs toward `preferred_edge_length`;
   - soft repulsion within `min_separation`.
3. **Clamp** after every step: each point moves toward its cell centroid until it is inside the cell shrunk to `CELL_CLAMP_SCALE` (0.8). That keeps it strictly inside its cell (I2).
4. **Clearance:** scale up uniformly once more, only if needed. Afterwards node points are at least `min_separation` apart, and every node point is at least `min_separation / 2` from any edge drawing it is not part of. This is a hard guarantee, so drawn node markers of radius ≤ `min_separation / 2` never overlap each other or a foreign edge.

Stats report:

- iterations
- min pairwise distance
- min node–foreign-edge distance
- max angle residual (excess outside the deadband)
- mean edge length
- `clearance_scale`
- `crossings` (validator conflicts; always 0)

Defaults: `preferred_edge_length = 120`, `min_separation = 60` (preview markers are radius 27). The C ABI runs this layout when it builds an opaque `MargenRspGraph`. It exports positions through `margen_rsp_graph_bulk`, and cells and doorways through `margen_rsp_graph_geometry`.

## C ABI

- `margen_generate_rsp_skeletal(seed, subdivisions, loop_weight, out_handle)` → opaque `MargenRspGraph` (final layer only)
- Bulk node ids, immediate parent ids (parallel; `UINT32_MAX` = no parent), constraints (`float angle`), and metric layout positions
- `margen_rsp_graph_geometry`:
  - Doorways, parallel to constraints. Hosts draw `subject → doorway → reference`.
  - Cells, parallel to node ids: counter-clockwise polygons addressed by `cell_offsets` / `cell_counts` into `cell_vertices`.
- Debug builds assert `validate_rsp_embedding` on the realized layer before export
- Free with `margen_rsp_graph_free`
- Full layer stacks are not exported over the ABI yet
- Skinned slabs: `margen_generate_rsp_skin` and friends ([rsp-skinning.md](rsp-skinning.md#c-abi))

## Out of scope (for now)

Per-edge / hierarchical angle tolerances, separate Binary vs TernaryOpen weight knobs, connect-all / distance-similarity inheritance thresholds, exposing layout config on `MargenRspGraphView`, 3D angles, and replacing the hex/`MargenWorldMesh` pipeline.
