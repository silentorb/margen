# Mythic

**Mythic** is a general-purpose utility layer—not world generation. The name continues a long habit of calling engine utility layers “Mythic” across homemade engines and Unreal plugins.

In this repository Mythic is a **separate CMake target and namespace** (`mythic` / `mythic::`) living beside margen so it can later move to its own repo without rewriting margen internals.

## Unreal sources

Workspace folder **`unreal-marloth-plugins`** → host `/mnt/e/dev/games/marloth/Plugins`:

| Plugin | Modules |
|--------|---------|
| Mythic | MythicSimulation, MythicClient, Resourceful, ResourcefulClient, ResourcefulEditor |
| MythicTesting | MythicTesting |

Generation depended primarily on **MythicSimulation** (`Dice`, math, distribution helpers).

## Ported (session 0)

- `mythic::Dice` — `mythic/Mythic/Dice.h` (+ `.cpp`)
- `mythic::Vector3i` — `mythic/Math/Vector3.h`
- `mythic::InlineVector` / `FixedArray` — `mythic/Mythic/Containers.h`

## Not ported (yet / ever in mythic core)

- MythicClient, Resourceful*
- Actor bounds, raycasting, static meshes, editor gizmos
- DotEnv / persistence / UE UObject wrappers

Those stay Unreal-specific or belong in engine integrators.
