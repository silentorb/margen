# Mythic

**Mythic** is a general-purpose utility layer—not world generation. The name continues a long habit of calling engine utility layers “Mythic” across homemade engines and Unreal plugins.

In this repository Mythic is a **separate Cargo crate** (`mythic`) living beside `margen` so it can later move to its own repo without rewriting margen internals.

## Unreal sources

Workspace folder **`unreal-marloth-plugins`** → host `/mnt/e/dev/games/marloth/Plugins`:

| Plugin | Modules |
|--------|---------|
| Mythic | MythicSimulation, MythicClient, Resourceful, ResourcefulClient, ResourcefulEditor |
| MythicTesting | MythicTesting |

Generation depended primarily on **MythicSimulation** (`Dice`, math, distribution helpers).

## Ported (Rust)

- `mythic::Dice` — `crates/mythic/src/dice.rs` (seeded `StdRng`; not bit-identical to C++ `std::mt19937`)
- `mythic::Vector3i` — `crates/mythic/src/vector3.rs`
- `mythic::Vector2i` — `crates/mythic/src/vector2.rs`
- `mythic::psuedorandom` — `crates/mythic/src/random.rs`

Inline/fixed SmallVector containers from the C++ port are **not** reimplemented; callers use `Vec` / arrays unless size semantics prove load-bearing.

## Not ported (yet / ever in mythic core)

- MythicClient, Resourceful*
- Actor bounds, raycasting, static meshes, editor gizmos
- DotEnv / persistence / UE UObject wrappers

Those stay Unreal-specific or belong in engine integrators.
