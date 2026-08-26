# Mythic

**Mythic** is a general-purpose utility layer—not world generation. The name continues a long habit of calling engine utility layers “Mythic” across homemade engines and Unreal plugins.

In Unreal, the **Mythic** plugin contained multiple modules (`MythicSimulation`, `Resourceful`, `MythicClient`, …). In this repository those become **separate Cargo crates** when ported—never one catch-all `mythic` package. **MythicSimulation** itself was already too broad; its ported surface is split by concern.

## Unreal sources

Workspace folder **`unreal-marloth-plugins`** → host `/mnt/e/dev/games/marloth/Plugins`:

| Plugin | Modules |
|--------|---------|
| Mythic | MythicSimulation, MythicClient, Resourceful, ResourcefulClient, ResourcefulEditor |
| MythicTesting | MythicTesting |

Generation depended primarily on **MythicSimulation** (`Dice`, math, distribution helpers).

## Ported crates (Rust)

| Crate | Contents |
|-------|----------|
| `mythic_math` | `Vector2i`, `Vector3i` |
| `mythic_dice` | `Dice`, `get_static_dice`, `psuedorandom` (`StdRng`; not bit-identical to C++ `std::mt19937`) |
| `mythic_distribution` | `shuffle_vec`, `select_many_ordered` (depends on `mythic_dice`) |

Inline/fixed SmallVector containers from the C++ port are **not** reimplemented; callers use `Vec` / arrays unless size semantics prove load-bearing.

## Not ported yet (future crates when needed)

| Future crate | UE source | Notes |
|--------------|-----------|-------|
| `mythic_resourceful` | Resourceful | Strongest non-sim candidate if attributes go engine-agnostic |
| `mythic_client` | MythicClient | UI-bound → often stays in engine hosts |
| `mythic_dotenv` | DotEnv | Engine-agnostic candidate |
| — | ActorBounds, raycasting, meshes, gizmos, editor | Stay in engine integrators |

Do **not** create a `mythic` or `mythic_simulation` facade crate.
