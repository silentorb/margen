# margen

Engine-agnostic **Rust** library for generating congruent, maze-like game worlds, exposed to engines through a **C ABI**.

This repository is a Cargo workspace of **focused crates** (Unreal modules → crates; no umbrella packages):

- **mythic_*** — utilities (`mythic_math`, `mythic_dice`, `mythic_distribution`)
- **margen_generation*** — world generation pipeline (`margen_generation`, `margen_generation_graphing`, `margen_generation_structure`, `margen_generation_graphing_analysis`)
- **margen_ffi** — C ABI (`crates/margen_ffi`, header [`include/margen.h`](include/margen.h))

Godot integration lives in the sibling **margen-godot** repository (GDExtension linking this ABI).

## Build

Requires a Rust toolchain.

```bash
cargo test
cargo build -p margen_ffi --release
```

## Layout

| Path | Purpose |
|------|---------|
| `crates/mythic_*/` | Mythic utility crates |
| `crates/margen_generation*/` | Generation pipeline crates |
| `crates/margen_ffi/` | C ABI |
| `include/margen.h` | C header for hosts |
| `docs/` | Design and port notes |

See [AGENTS.md](AGENTS.md) for agent conventions and [docs/overview.md](docs/overview.md) for world-gen goals.

Historical C++ sources: branch `archive/cpp-port`.
