# Agent notes — margen

## Project

Engine-agnostic C++ **world generation** library. No Godot or Unreal APIs in this tree.

Two layers in one repo (keep them separate):

| Layer | Namespace / target | Role |
|-------|--------------------|------|
| **mythic** | `mythic::` / `mythic::mythic` | General utilities (RNG, math, inline containers). Ported from Unreal `Plugins/Mythic` (`MythicSimulation`). Not world-gen. |
| **margen** | `margen::` / `margen::margen` | World generation. PUBLIC-links mythic. |

Godot integration: sibling repo **margen-godot** (GDExtension). Game-specific generation glue for Marloth belongs in the **marloth** Godot repo when needed—not here early.

## Layout

| Path | Purpose |
|------|---------|
| [`mythic/`](mythic/) | Mythic headers **and** sources together (e.g. `Mythic/Dice.h` + `Dice.cpp`) |
| [`margen/`](margen/) | Margen headers **and** sources together (e.g. `Story/GenerateStory.h` + `.cpp`) |
| [`tests/`](tests/) | Catch2 unit tests |
| [`docs/`](docs/) | Design / port docs |
| [`cmake/`](cmake/) | Package export helpers |

There is **no** separate top-level `include/` vs `src/` split.

## Conventions

- **C++ style:** Follow [docs/cpp-style.md](docs/cpp-style.md) (shared with margen-godot and other marloth-workspace C++). Data-oriented / “C++ without classes,” public data, free functions, liberal `const`, no `friend`.
- **Line endings:** Unix (LF). See [`.gitattributes`](.gitattributes) and [`.editorconfig`](.editorconfig).
- **Headers:** use `.h` (not `.hpp`) for C++ headers.
- **Indent:** tabs for C++ (match Unreal sources).
- **Colocated headers and sources:** Keep `.h` and `.cpp` in the same directories. Do not introduce an `include/` tree. C-style header separation is a legacy design that may have made sense historically but has not propagated to modern languages and drives many of C++’s worst ergonomics; prefer mitigating the header/source divide and treating C++ more like contemporary languages where a module’s files live together.
- **Near-copy ports:** Prefer a near-literal copy of unreal-marloth organization and style. Mirror PascalCase paths/filenames, lowerCamelCase identifiers, comments, and header/`.cpp` splits. Change only engine/STL substitutions (see [docs/overview.md](docs/overview.md)).
- **C++20**, modern CMake (target-centric; no directory-global includes). Public includes use the repo root (`#include <mythic/Mythic/Dice.h>`, `#include <margen/Story/GenerateStory.h>`).
- Prefer **STL** for large/unknown containers; preserve Unreal **inline/fixed** semantics with `mythic::InlineVector` / `mythic::FixedArray` when porting `TInlineAllocator` / `TFixedAllocator`.
- Do **not** put Dice/containers under `margen::`.
- Prefer a **late engine boundary**: generation outputs datasets; engines convert later.

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

## Build

```bash
cmake -S . -B build -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Debug
cmake --build build
ctest --test-dir build --output-on-failure
```

## Docs

- [docs/cpp-style.md](docs/cpp-style.md) — shared C++ style (data-oriented / FP-inspired)
- [docs/overview.md](docs/overview.md) — margen goals, port roadmap, substitutions
- [docs/mythic.md](docs/mythic.md) — Mythic layer and UE plugin scope
