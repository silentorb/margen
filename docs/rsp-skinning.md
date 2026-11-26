# RSP skinning

`margen_rsp_skinning` turns a laid-out RSP layer ([rsp.md](rsp.md)) into floor slabs:

- every node becomes a **room**: a low-poly cylinder, flat on top;
- every constraint becomes a **pathway** of width `w`, attached to a room side that faces it.

Floors have depth, all tops are at z = 0, and there are no walls or ceilings yet.

The RSP proof covers zero-width edge drawings only. Real floors need margins the plain layout does not have:

- pathways can leave a node only 5° apart;
- a doorway can sit 4 units from its node while nodes are 60 apart;
- doorways can sit right at cell corners.

Skinning therefore fits the footprints into the partition with explicit margins, so they are overlap-free **by construction**. It returns an error when no fit exists.

## Pipeline

1. **`center_rooms`** moves each node point to the centre of the largest circle inscribed in its cell.
   - That point is strictly inside the cell, so every partition invariant still holds.
   - This gives each room the most space its cell allows. It cuts the scale needed at depth 3 from a median of about 4.4 to about 1.
2. **`fit_floor_plan`** finds a uniform scale `s ≥ 1` and builds the footprints at that scale (see below).
3. **`assign_skinners`** picks a skinner per floor piece. This is the delegation seam for region- or biome-specific meshing. Today every piece gets `RspPieceSkinner::Slab`.
4. **`skin_floor_plan`** emits triangles.

`skin_rsp_layer` runs all four. `validate_floor_plan` independently re-checks the result with separating-axis tests.

## Construction at a fixed scale

Inputs: convex counter-clockwise cells `C_v` with disjoint interiors, node points `x_v` strictly inside them, and one doorway per constraint on the wall its two cells share. All lengths below are at scale `s`.

1. **Shared walls.**
   - Two cells with disjoint interiors meet in a convex set, so their shared wall `S_ab` is one segment.
   - It is found from the existing doorway: take the edge line both cells have there, clip it to each cell, and intersect the two intervals.
   - If the cells only touch at a point → `NoSharedWall`.
2. **Door slots.**
   - The `k` constraints between one cell pair split `S_ab` into `k` equal slots, ordered by their old doorways.
   - Each constraint's **door** `D_e` is the middle `w` of its slot.
   - Requires: slot length ≥ `w + 2·gap` (`SlotTooShort`).
   - The door centre becomes the new doorway and is inserted as a collinear vertex of both cells. The RSP invariants still hold, so `validate_rsp_embedding` passes on the fitted layer.
3. **Sectors.**
   - Seen from `x_v`, each door covers an angular interval around its centre direction `θ_e`. Those directions are distinct, because a ray from an interior point leaves a convex cell exactly once.
   - The circle is cut at the middle of each gap between consecutive doors. `angular_gap / 2` is kept free on each side of a cut.
   - Requires: every gap ≥ `angular_gap` (`DoorsTooClose`).
   - A node with one exit gets the whole circle.
4. **Room radius.**
   - A side of width `w` on a circle of radius `R` spans `2α`, with `α = asin(w / 2R)`.
   - `R_req` is the smallest radius ≥ `room_min_radius` whose side fits every sector.
   - `R = max(R_req, min(room_preferred_radius, d_v − gap))`, where `d_v` is the distance from `x_v` to the cell boundary.
   - Requires: `R + gap ≤ d_v` (`RoomTooLarge`).
5. **Faces.**
   - Each exit gets the chord `φ_e ± α` of the room circle, where `φ_e` is `θ_e` clamped so the chord stays inside its sector.
   - That chord is exactly `w` wide and perpendicular to `φ_e`.
   - Requires: both door ends lie at least `gap` beyond the chord's line (`DoorBehindFace`).
   - Requires: the angular hull of chord and door is narrower than π (`WedgeTooWide`).
6. **Room polygon.** Face chords, with arc points filling each gap at least every `2π / target_sides`. There is no cap on the vertex count. All vertices lie on one circle, so the polygon is convex. A node with no exits is a regular `target_sides`-gon.
7. **Half-pathways.** `Q = conv(face ∪ door)`: the quad from the room side to the door. The two halves of a pathway meet at its door.

## Why footprints cannot overlap

All pieces are convex.

- **Containment.**
  - The room lies inside the interior of its cell, because `R + gap ≤ d_v`.
  - The face lies in the room and the door lies on the cell wall. The cell is convex, so the quad lies in the cell.
  - A convex combination that puts weight on an interior point is interior. So the quad meets the cell boundary only in its door.
- **Different cells.**
  - Cell interiors are disjoint, so pieces of different cells can meet only on cell boundaries. By containment, those meeting points are door segments.
  - Slots are disjoint, so the only cross-cell contact is the two halves of one pathway, along their door.
- **Same cell.**
  - Each quad lies inside its wedge: the convex cone from `x_v` through its four points, narrower than π.
  - Wedges sit in disjoint sectors at least `angular_gap` apart, so quads of one room are disjoint.
  - The room lies on the inner side of each face line, and the door lies strictly beyond it. So a quad meets its room only along that face.

Pieces therefore have disjoint interiors and touch only along declared joins: room side ↔ half-pathway, and half-pathway ↔ half-pathway at the door. Join edges share bit-identical endpoints. Slabs emit side walls only on non-join edges, so the whole mesh is a closed, consistently oriented surface (tested).

## Choosing the scale

`fit_floor_plan` tries `s = 1, 2, 4, …`, then bisects 8 steps between the last failure and the first success. Only scales whose construction succeeded are returned.

The search always terminates. As `s` grows:

- each door's angular size shrinks toward 0, while the directions between doors do not depend on scale;
- so the sectors approach fixed positive widths, and `R_req` approaches a finite limit;
- meanwhile `d_v`, slot lengths and door distances grow without bound;
- and the face clamp moves a face at most `α < π/2` from its door.

Every requirement therefore holds at some finite scale. Past `MAX_SKIN_SCALE` (2²⁰) the result is `ScaleLimit { blocker }`, carrying the last requirement that failed.

Scale needed with defaults (60 seeds, loop weight 2/3):

| depth | median | p90 | max |
|---|---|---|---|
| 2 | 1.00 | 1.00 | 9.3 |
| 3 | 1.05 | 1.86 | 5.9 |
| 4 | 1.78 | 3.11 | 3.8 |
| 5 | 2.27 | 4.83 | 7.4 |

## Defaults (layout units; `preferred_edge_length` = 120)

| Field | Default |
|---|---|
| `pathway_width` | 24 |
| `gap` | 2.4 |
| `room_min_radius` | 18 |
| `room_preferred_radius` | 30 |
| `floor_thickness` | 6 |
| `target_sides` | 6 |
| `angular_gap` | 2° |

`sanitize_config` replaces values that are not finite and positive. It also keeps `room_min_radius` above `w/2` and `room_preferred_radius ≥ room_min_radius`.

## C ABI

- `margen_generate_rsp_skin(seed, subdivisions, loop_weight, config, out_handle)` takes the same graph and layout as `margen_generate_rsp_skeletal`, then skins it.
  - `config` may be NULL.
  - Failure → `MARGEN_ERR_GENERATION_FAILED` plus `margen_last_error`.
- `margen_rsp_skin_mesh` returns positions and normals in margen XYZ (Z up, flat shaded), CCW-outward triangle indices, and `MargenRspSkinPiece` triangle ranges.
  - Rooms: `id` = node.
  - Pathways: `id` = constraint and `node` = the room each half leaves.
- `margen_rsp_skin_rooms` returns room node ids, centres and radii (for spawn points and annotations).
- `margen_rsp_skin_scale` returns the scale that was applied.
- Free the handle with `margen_rsp_skin_free`.
- Debug builds assert `validate_floor_plan` and `validate_rsp_embedding` before export.

## Not yet

- Walls, ceilings and height variation.
- Per-region skinner assignment.
- Exposing the fitted partition (cells, moved doorways) over the ABI.
