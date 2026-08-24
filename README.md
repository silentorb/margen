# margen

Engine-agnostic C++ library for generating congruent, maze-like game worlds.

This repository contains two layers:

- **mythic** — general-purpose utilities (`mythic::`), not world generation
- **margen** — world generation (`margen::`), depends on mythic

Godot integration lives in the sibling **margen-godot** repository (GDExtension).

## Build

Requires CMake ≥ 3.21 and a C++20 compiler.

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build
ctest --test-dir build --output-on-failure
```

## Layout

| Path | Purpose |
|------|---------|
| `mythic/` | Mythic headers and sources together |
| `margen/` | Margen headers and sources together |
| `tests/` | Catch2 unit tests |
| `docs/` | Design and port notes |

See [AGENTS.md](AGENTS.md) for agent conventions and [docs/overview.md](docs/overview.md) for world-gen goals.
