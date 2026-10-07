# Agent notes — margen

## Project

Engine-agnostic **Rust** world generation library with a **C ABI** for hosts. No Godot or Unreal APIs in this tree.

The workspace is **many small crates**. Unreal **modules** map to Cargo **crates** (not Rust modules). Catch-all UE modules (e.g. MythicSimulation) are split further into focused crates. There is **no** umbrella `mythic` or `margen` crate.

| Area | Crates | Role |
|------|--------|------|
| **Mythic utilities** | `mythic_math`, `mythic_dice`, `mythic_distribution` | Vectors, RNG, shuffle/select. From Unreal Mythic / MythicSimulation feature areas—not world-gen. |
| **Generation** | `margen_generation`, `margen_generation_graphing`, `margen_generation_structure`, `margen_generation_graphing_analysis`, `margen_generation_surfacing` | UE Generation* modules, one crate each (DAG enforced by Cargo). |
| **C ABI** | `margen_ffi` | `cdylib` / `staticlib`; header [`include/margen.h`](include/margen.h). |

Godot integration: sibling repo **margen-godot** (GDExtension calling the C ABI). Game-specific generation glue for Marloth belongs in the **marloth** Godot repo (likely C#) when needed—not here early.

## Layout

| Path | Purpose |
|------|---------|
| [`crates/mythic_*/`](crates/) | Focused Mythic utility crates |
| [`crates/margen_generation*/`](crates/) | Generation pipeline crates |
| [`crates/margen_ffi/`](crates/margen_ffi/) | C ABI |
| [`include/margen.h`](include/margen.h) | Hand-written C header |
| [`docs/`](docs/) | Design / port docs |

## Conventions

- **Rust style:** Follow [docs/rust-style.md](docs/rust-style.md) — plain data, free functions, flat pipeline steps.
- **Crate policy:** Unreal module ≡ at least one Cargo crate. Prefer more focused crates over catch-alls. **Forbidden:** collapsing multiple UE modules into one crate, or inventing umbrella/facade crates (`mythic`, `margen`).
- **Line endings:** Unix (LF). See [`.gitattributes`](.gitattributes) and [`.editorconfig`](.editorconfig).
- Prefer a **late engine boundary**: generation outputs datasets; engines convert later via the C ABI.
- **Coordinates:** Rust and the C ABI use **right-handed Z-up** (+X right, +Y north/forward, +Z up). See [docs/coordinates.md](docs/coordinates.md). Hosts convert at the boundary.
- Dice lives in **`mythic_dice`**; generation crates must not own RNG utilities.
- **Friction:** If an Unreal construct would force awkward Rust, redesign with existing margen types—or stop and report (see rust-style). Do not copy UE patterns just to stay close to the reference source.

## Unreal reference policy

Unreal `Source/` / `Plugins/` are a **behavioral / feature reference**, not a template to mirror line-by-line.

- Preserve **player-visible and pipeline outcomes** for the feature being ported.
- **Implementation may differ**: types, control flow, intermediate maps, and folder layout inside a crate need not match Unreal.
- Prefer **existing margen types and APIs**; design clean flat steps for this stack. Do not reintroduce UE-only patterns (UObject graphs, `TFixedAllocator` bags, compressed `FIntVector` keys, dead branches) unless they earn their keep.
- **Crate mapping:** Unreal module → at least one Cargo crate. Internal modules need **not** mirror UE folders 1:1. No umbrella/facade crates.
- Coordinates always adapt into RH Z-up (`rh-z-up-coordinates`).

## Logic improvement classes

**Improvement classes** gate intentional **outcome** divergences from Unreal reference behavior (bug fixes / deliberate product changes)—not stylistic redesigns. New classes must be proposed and approved once; then document them here so later matching fixes do not need re-approval.

| Class | Meaning |
|-------|---------|
| `partial-branch-bug` | One control-flow branch implements the intended effect; a sibling branch silently skips it. Fix the incomplete branch. |
| `empty-input-sentinel` | Do not plant fake sentinel elements for empty inputs; keep consumers empty-safe (no-op / early return / explicit min-size contract) so the pipeline still runs. |
| `incorrect-traversal-endpoint` | Traversal advances the wrong node/edge endpoint. Fix the walk; add a test when practical. |
| `rh-z-up-coordinates` | Canonical space is right-handed Z-up (+Y north), not Unreal left-handed / south-positive hex tables. Adapt ports into this basis; see [docs/coordinates.md](docs/coordinates.md). |

## Reference (Unreal port source)

In the marloth workspace:

- `unreal-marloth` → Unreal `Source/` (Generation*, Modeling, …)
- `unreal-marloth-plugins` → Unreal `Plugins/` (Mythic, MythicTesting, …)

Historical C++ near-copy: git branch **`archive/cpp-port`** (superseded policy; do not use as the style target).

## Build

```bash
cargo test
cargo build -p margen_ffi --release
```

## Docs

- [docs/rust-style.md](docs/rust-style.md) — living Rust style
- [docs/overview.md](docs/overview.md) — margen goals, port roadmap
- [docs/coordinates.md](docs/coordinates.md) — right-handed Z-up contract and host maps
- [docs/surfacing.md](docs/surfacing.md) — face IR through `RenderFace`
- [docs/mythic.md](docs/mythic.md) — Mythic crates and UE plugin scope
- [docs/cpp-style.md](docs/cpp-style.md) — archived C++ pointer
