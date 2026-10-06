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

## Layers (crates)

| Layer | Crates | Purpose |
|-------|--------|---------|
| **Mythic** | `mythic_math`, `mythic_dice`, `mythic_distribution` | General utilities. See [mythic.md](mythic.md). |
| **Generation** | `margen_generation` → `margen_generation_graphing` → `margen_generation_structure` → `margen_generation_graphing_analysis` → `margen_generation_surfacing` | World generation pipeline (UE Generation* modules as separate crates). |
| **C ABI** | `margen_ffi` | C ABI (`include/margen.h`) for engine hosts (margen-godot / Marloth). Prefer bulk exports (`margen_world_faces_bulk`) over chatty getters. |

There is **no** catch-all `mythic` or `margen` crate. Callers depend on the leaf crates they need.

## Stack

- **Implementation language:** Rust (Cargo workspace under `crates/`).
- **Engine boundary:** Generation outputs datasets; integrators convert later.
- **Hosts:** margen-godot (C++ GDExtension) and Marloth native glue call the **C ABI**, not Rust internals.

## Port policy

Ports from Unreal should preserve **behavior and pipeline shape**, expressed as idiomatic Rust with **flat FP steps**. See [rust-style.md](rust-style.md).

**Crate mapping:** each Unreal **module** becomes at least one Cargo **crate**. Folders inside a module become Rust modules within that crate. Catch-all modules (MythicSimulation) are split into focused crates. Extra focused crates are allowed; umbrella/facade crates are not.

**Logic:** default = Unreal behavior. Intentional fixes use documented [improvement classes](../AGENTS.md#logic-improvement-classes). New classes need user approval once, then get listed in agent docs.

Historical C++ near-copy policy and sources: branch **`archive/cpp-port`**.

## Port roadmap (from Unreal)

| Stage | Unreal modules | Status |
|-------|----------------|--------|
| 0 | MythicSimulation subset + GenerationGraphing (DAG/story, `sectors_to_dag`) | **Done** (`mythic_*`, `margen_generation_graphing`) |
| 1 | Generation core types + story → `CellGrid` | **Done** (`margen_generation`, `margen_generation_structure` + C ABI) |
| 1–2 | GenerationStructure (rasterization, optional cluster connect, Block/congruence, Biome type, spatial `BiomeDistribution`, prefab-seed `windingPath` / `generate_world_grid`) | **Done** (nested sub-biomes deferred; refinement later) |
| 2 | GenerationGraphingAnalysis + Population (math) | **Done** (GraphingAnalysis crate); Population **Later** |
| 3 | GenerationSurfacing face IR through `RenderFace` | **Done** (`margen_generation_surfacing`; empty corner maps = prism faces) |
| 3+ | Integration facade, face/world-grid C ABI, `GridRefinement` | **Done** (world-faces C ABI); Integration facade + `GridRefinement` later |
| — | Modeling / Editing / SimulationGeneration game glue | Rewrite, defer, or marloth-side |

Story graph pipeline detail: [story-graph.md](story-graph.md). Sector analysis detail: [sector-analysis.md](sector-analysis.md). Production winding path: [winding-path.md](winding-path.md). Face IR: [surfacing.md](surfacing.md).

## Engine-agnostic types

Unreal engine types (`UBiome`, `UDataAsset`, GameplayTags, material assets) are **not** referenced from margen. Generation-relevant fields live as plain Rust structs (`margen_generation::Biome`, placeable ids, tag strings). Hosts translate adapter resources at the boundary.

**Shared biome ownership:** many cells share one biome via `Arc<Biome>` on `WorldCell`. Numeric `Biome.id` maps to the C ABI `MargenCell.biome_id`.

## Build

Requires a Rust toolchain (1.88+ recommended).

```bash
cargo test
cargo build -p margen_ffi --release
```

## Engine boundary

Margen stays engine-agnostic. Integrators (e.g. margen-godot) convert final datasets into engine meshes, materials, and actors via the C ABI.
