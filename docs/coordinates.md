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

## Versus Unreal reference

Unreal Generation source is **left-handed Z-up** with south-positive **+Y** in the hex tables (`NORTH = (0, -1, 0)`). Margen intentionally diverges (`rh-z-up-coordinates`): compass vectors use **+Y north** on the same hex lattice (cube distance 1), and prism corner rings sit with north at **+Y**. Do not “fix” generation back to Unreal axis tables when porting further modules—adapt Unreal numbers into this basis instead.
