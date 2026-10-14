# Coordinate system

Margen Rust and the C ABI use a single **right-handed Z-up** world space:

| Axis | Meaning |
|------|---------|
| **+X** | Right / east |
| **+Y** | Forward / north |
| **+Z** | Up |

Hex grids lie in the **XY** plane; cell height and vertical neighbors use **Z**. Compass direction indices (`NORTH`, `SOUTH`, …) map to this basis (`NORTH = (0, 1, 0)`, `UP = (0, 0, 1)`).

`MargenVec3` and all face/point exports are in this space. Hosts convert at the engine boundary (late conversion). For Godot (right-handed Y-up), the position map is:

\[
(x,\ y,\ z) \mapsto (x,\ z,\ -y)
\]

Extents such as `cell_size` remap component-wise as Godot `(sx, sy_up, sz) →` margen `(sx, sz, sy_up)` before ABI calls.

## Face winding

Exported face polygons are ordered so a **right-handed** edge cross product points **outward** (counter-clockwise when viewed from outside). That is the engine-agnostic IR contract; see [surfacing.md](surfacing.md).

**Godot** treats **clockwise** winding as the front face (default back-face cull). The orientation-preserving position map above does not flip winding, so **margen-godot** reverses triangle fan order (and derived normals) when building `ArrayMesh` surfaces. Do not reverse the Rust IR to match Godot—keep late conversion at the host.

## Versus Unreal reference

Unreal Generation source is **left-handed Z-up** with south-positive **+Y** in the hex tables (`NORTH = (0, -1, 0)`). Margen intentionally diverges (`rh-z-up-coordinates`): compass vectors use **+Y north** on the same hex lattice (cube distance 1), and prism corner rings sit with north at **+Y**. Do not “fix” generation back to Unreal axis tables when porting further modules—adapt Unreal numbers into this basis instead.

### SharedCellCorner remaps

`SharedCellCorner` collapses local corners onto canonical indices **7** and **8** by shifting the cell key. Unreal’s remap deltas are compass moves (north / southwest / northwest in UE vectors). Margen must use the **same compass neighbors expressed in RH grid vectors**, not component-wise Y-flips of the Unreal literals—flat diagonals differ (`UE NE=(1,-1)` vs Rust `NE=(1,0)`), so a Y-flip maps to the wrong cells and breaks slope corner columns.

| Case | Compass | RH delta |
|------|---------|----------|
| 6 | North | `(0, 1, 0)` |
| 9 | Southwest | `(-1, 0, 0)` |
| 10 | Northwest | `(-1, 1, 0)` |
| 11 | Northwest | `(-1, 1, 0)` |
