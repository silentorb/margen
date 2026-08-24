# Margen overview

Margen generates congruent, maze-like game worlds from atomic parts with organic geometry.

## Goals

- Congruent, maze-like maps
- Worlds built from atomic parts (macro prefabs optional)
- Organic / fluid geometry (not rigid cubes and fixed corners)
- Morphing levels
- Non-linear gating (keys, doors, switches)
- Limited room-over-room (some features remain 2D-oriented)
- Robust even distribution algorithms
- Spatially aware placement so actors fit their spaces

## Layers

| Layer | Purpose |
|-------|---------|
| **mythic** | General utilities (`Dice`, `Vector3i`, `InlineVector`, …). See [mythic.md](mythic.md). |
| **margen** | World generation (graphing → structure → analysis → population → surfacing → integration). |

## Near-copy port policy

Ports from Unreal should stay as close to a copy as practical:

- Same relative folders and PascalCase filenames under repo-root `margen/` and `mythic/` (headers colocated with `.cpp`; **no** separate `include/` tree)
- Same identifier spelling (lowerCamelCase APIs)
- Same comments and which APIs live in headers vs translation units
- Wrap symbols once in `margen::` or `mythic::` for packaging (no extra nested namespaces)

**Layout note:** Colocate `.h` and `.cpp` to mitigate C++’s legacy header/source split rather than accentuating it (see [AGENTS.md](../AGENTS.md)).

**Allowed substitutions:** `TArray`/`TMap`/`TSet`/`TPair` → `InlineVector`/`FixedArray`/`std::unordered_map`/`std::unordered_set`/`std::pair`; strip UHT/`*_API`; `FDice`/`FRandomStream` → `mythic::Dice`/`std::mt19937`; UE integer typedefs → `std::int*_t`.

**Logic:** default = Unreal behavior. Intentional fixes use documented [improvement classes](../AGENTS.md#logic-improvement-classes). New classes need user approval once, then get listed in agent docs.

## Containers

| Unreal | Port |
|--------|------|
| `TArray<T, TInlineAllocator<N>>` | `mythic::InlineVector<T, N>` |
| `TArray<T, TFixedAllocator<N>>` | `mythic::FixedArray<T, N>` |
| `TSet` with inline set allocator | Inline/fixed array + uniqueness helpers |
| plain `TArray` / `TMap` / `TSet` | `std::vector` / `std::unordered_map` / `std::unordered_set` |

## Port roadmap (from Unreal)

| Stage | Unreal modules | Status |
|-------|----------------|--------|
| 0 | MythicSimulation subset + GenerationGraphing (DAG/story, `SectorsToDag`) | **Done** |
| 1 | Generation core types + story → `CellGrid` | **In progress** (core types, `storyToClusters`, `generateStoryGrid` ported) |
| 1–2 | GenerationStructure (rasterization, winding path, …) | Later |
| 2 | GenerationGraphingAnalysis + Population (math) | Later |
| 3+ | Surfacing face IR, Integration facade | Later |
| — | Modeling / Editing / SimulationGeneration game glue | Rewrite, defer, or marloth-side |

Story graph pipeline detail: [story-graph.md](story-graph.md).

## Build

Requires CMake ≥ 3.21, Ninja, and a C++20 compiler.

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build
ctest --test-dir build --output-on-failure
```

## Engine boundary

Margen stays engine-agnostic. Integrators (e.g. margen-godot) convert final datasets into engine meshes, materials, and actors.
