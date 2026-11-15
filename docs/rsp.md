# Relative Space Partitioning (RSP)

Experimental parallel path beside the hex-grid generation pipeline. Intent comes from the design journal (`docs/journal/2026-11-14.md`): define node graphs by **relative direction constraints**, defer absolute distance/scale until late realization.

## Crates

| Crate | Role |
|-------|------|
| `margen_rsp` | Core model: nodes, direction vectors, constraints, graph |
| `margen_rsp_generation` | Generators (currently skeletal only) |

Neither crate depends on `margen_generation*`. Hosts consume results through `margen_ffi` / `include/margen.h`.

## Direction model

`RspDirection` is a RH Z-up `mythic_math::Vector3i`, not an enum. Logic compares against constants:

| Constant | Vector |
|----------|--------|
| `NORTH` | `(0, 1, 0)` |
| `SOUTH` | `(0, -1, 0)` |
| `EAST` | `(1, 0, 0)` |
| `WEST` | `(-1, 0, 0)` |
| `UP` | `(0, 0, 1)` (reserved; generators must not emit yet) |
| `DOWN` | `(0, 0, -1)` (reserved; generators must not emit yet) |

**2D-first generation:** current generators only emit planar constraints (`z == 0`). The direction type is already 3D so Up/Down and non-cardinal vectors can arrive without reshaping `RspConstraint`.

An `RspConstraint` means **subject lies in `direction` of reference** (e.g. A is North of B when `direction == NORTH`).

## Skeletal generator

`generate_skeletal_rsp()` returns two nodes (`0`, `1`) and one constraint `{ subject: 0, direction: NORTH, reference: 1 }`. Absolute placement is not part of the graph; the C ABI also exports a trivial **debug layout** (`MargenVec2` per node) for editor drawing only.

## C ABI

- `margen_generate_rsp_skeletal` → opaque `MargenRspGraph`
- Bulk node ids, constraints (`MargenIVec3` direction), and debug layout positions
- Free with `margen_rsp_graph_free`

## Out of scope (for now)

Absolute metric realization, loops, multi-scale subdivision, emitting Up/Down, and replacing the hex/`MargenWorldMesh` pipeline.
