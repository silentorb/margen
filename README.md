# margen

Engine-agnostic **Rust** library for generating congruent, maze-like game worlds, exposed to engines through a **C ABI**.

This repository contains:

- **mythic** — general-purpose utilities (`crates/mythic`)
- **margen** — world generation (`crates/margen`), depends on mythic
- **margen-ffi** — C ABI (`crates/margen-ffi`, header [`include/margen.h`](include/margen.h))

Godot integration lives in the sibling **margen-godot** repository (GDExtension linking this ABI).

## Build

Requires a Rust toolchain.

```bash
cargo test
cargo build -p margen-ffi --release
```

## Layout

| Path | Purpose |
|------|---------|
| `crates/mythic/` | Mythic utilities |
| `crates/margen/` | World generation |
| `crates/margen-ffi/` | C ABI |
| `include/margen.h` | C header for hosts |
| `docs/` | Design and port notes |

See [AGENTS.md](AGENTS.md) for agent conventions and [docs/overview.md](docs/overview.md) for world-gen goals.

Historical C++ sources: branch `archive/cpp-port`.
