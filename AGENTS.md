# Agent notes — margen

## Project

Engine-agnostic **Rust** world generation library with a **C ABI** for hosts. No Godot or Unreal APIs in this tree.

| Layer | Crate | Role |
|-------|-------|------|
| **mythic** | `crates/mythic` | General utilities (RNG, math). Ported from Unreal `Plugins/Mythic` (`MythicSimulation`). Not world-gen. |
| **margen** | `crates/margen` | World generation. Depends on mythic. |
| **margen-ffi** | `crates/margen-ffi` | `cdylib` / `staticlib` C ABI; header [`include/margen.h`](include/margen.h). |

Godot integration: sibling repo **margen-godot** (GDExtension calling the C ABI). Game-specific generation glue for Marloth belongs in the **marloth** Godot repo (likely C#) when needed—not here early.

## Layout

| Path | Purpose |
|------|---------|
| [`crates/mythic/`](crates/mythic/) | Mythic utilities |
| [`crates/margen/`](crates/margen/) | World generation |
| [`crates/margen-ffi/`](crates/margen-ffi/) | C ABI |
| [`include/margen.h`](include/margen.h) | Hand-written C header |
| [`docs/`](docs/) | Design / port docs |

## Conventions

- **Rust style:** Follow [docs/rust-style.md](docs/rust-style.md) — plain data, free functions, flat pipeline steps.
- **Line endings:** Unix (LF). See [`.gitattributes`](.gitattributes) and [`.editorconfig`](.editorconfig).
- Prefer a **late engine boundary**: generation outputs datasets; engines convert later via the C ABI.
- Do **not** put Dice under the `margen` crate.
- **Friction:** If Unreal constructs force awkward Rust, stop and report (see rust-style).

## Logic improvement classes

Default port = Unreal behavior. **New classes** of intentional logic change must be proposed and approved once; then document the class here so later matching fixes do not need re-approval.

| Class | Meaning |
|-------|---------|
| `partial-branch-bug` | One control-flow branch implements the intended effect; a sibling branch silently skips it. Fix the incomplete branch. |
| `empty-input-sentinel` | Do not plant fake sentinel elements for empty inputs; keep consumers empty-safe (no-op / early return / explicit min-size contract) so the pipeline still runs. |
| `incorrect-traversal-endpoint` | Traversal advances the wrong node/edge endpoint. Fix the walk; add a test when practical. |

## Reference (Unreal port source)

In the marloth workspace:

- `unreal-marloth` → Unreal `Source/` (Generation*, Modeling, …)
- `unreal-marloth-plugins` → Unreal `Plugins/` (Mythic, MythicTesting, …)

Historical C++ port: git branch **`archive/cpp-port`**.

## Build

```bash
cargo test
cargo build -p margen-ffi --release
```

## Docs

- [docs/rust-style.md](docs/rust-style.md) — living Rust style
- [docs/overview.md](docs/overview.md) — margen goals, port roadmap
- [docs/mythic.md](docs/mythic.md) — Mythic layer and UE plugin scope
- [docs/cpp-style.md](docs/cpp-style.md) — archived C++ pointer
