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
| **mythic** | General utilities (`Dice`, `Vector3i`, …). See [mythic.md](mythic.md). |
| **margen** | World generation (graphing → structure → analysis → population → surfacing → integration). |
| **margen-ffi** | C ABI (`include/margen.h`) for engine hosts (margen-godot, future C# P/Invoke). |

## Stack

- **Implementation language:** Rust (Cargo workspace under `crates/`).
- **Engine boundary:** Generation outputs datasets; integrators convert later.
- **Hosts:** margen-godot (C++ GDExtension) and Marloth game glue (C#) call the **C ABI**, not Rust internals.

## Port policy

Ports from Unreal should preserve **behavior and pipeline shape**, expressed as idiomatic Rust with **flat FP steps**. See [rust-style.md](rust-style.md).

**Logic:** default = Unreal behavior. Intentional fixes use documented [improvement classes](../AGENTS.md#logic-improvement-classes). New classes need user approval once, then get listed in agent docs.

Historical C++ near-copy policy and sources: branch **`archive/cpp-port`**.

## Port roadmap (from Unreal)

| Stage | Unreal modules | Status |
|-------|----------------|--------|
| 0 | MythicSimulation subset + GenerationGraphing (DAG/story, `sectors_to_dag`) | **Done** (Rust) |
| 1 | Generation core types + story → `CellGrid` | **Done** (Rust: `story_to_clusters`, `generate_story_grid` + C ABI) |
| 1–2 | GenerationStructure (rasterization, optional cluster connect, Block/congruence, Biome type) | **Done** (Rust: `rasterize_cluster_grid`, optional `connect_cluster_cells`; spatial biome distribution still later) |
| 2 | GenerationGraphingAnalysis + Population (math) | Later |
| 3+ | Surfacing face IR, Integration facade | Later |
| — | Modeling / Editing / SimulationGeneration game glue | Rewrite, defer, or marloth-side |

Story graph pipeline detail: [story-graph.md](story-graph.md).

## Engine-agnostic types

Unreal engine types (`UBiome`, `UDataAsset`, GameplayTags, material assets) are **not** referenced from margen. Generation-relevant fields live as plain Rust structs (`margen::Biome`, placeable ids, tag strings). Hosts translate adapter resources at the boundary.

**Shared biome ownership:** many cells share one biome via `Arc<Biome>` on `WorldCell`. Numeric `Biome.id` maps to the C ABI `MargenCell.biome_id`.

## Build

Requires a Rust toolchain (1.88+ recommended).

```bash
cargo test
cargo build -p margen-ffi --release
```

## Engine boundary

Margen stays engine-agnostic. Integrators (e.g. margen-godot) convert final datasets into engine meshes, materials, and actors via the C ABI.
