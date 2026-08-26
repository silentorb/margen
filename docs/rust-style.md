# Rust style guide

Living style for margen and mythic Rust crates. C++ hosts that only call the C ABI (e.g. margen-godot) follow their own host conventions; see the archived [cpp-style.md](cpp-style.md) pointer.

## Orientation

Prefer **lowest-common-denominator FP**: plain data, free functions, and many **simple flat steps** instead of few deeply nested mutating steps. That keeps ownership and lifetimes obvious.

## Data vs functions

- Prefer plain `struct`s with public fields plus **free functions**.
- Keep inherent methods minimal (constructors, trivial accessors).
- Do not grow domain types with methods in every dependent crate—dependents add free functions.

## Ownership and mutation

- Prefer **immutable inputs / owned outputs**.
- Mutate only in tight local steps; avoid long-lived shared `&mut` bags across nested helpers.
- When the borrow checker fights simultaneous `&mut Dice` + `&mut CellGrid` + nested map walks, **stage flat passes** (collect, then apply)—do not reach for interior mutability by default.

## Collections

- Prefer **homogeneous** collections and normalized records (SQL-like).
- Default to `Vec`, `HashMap` / `BTreeMap`, `HashSet` / `BTreeSet`.
- Use `BTreeMap` / `BTreeSet` when iteration order must be deterministic across platforms.
- Do **not** reimplement Unreal `TInlineAllocator` SmallVectors unless size semantics are load-bearing; prefer `Vec` / fixed arrays / `arrayvec` only when needed.

## Errors

- Prefer `Result` / explicit status for **expected** failure.
- Panic only for programmer invariants or documented fail-fast aborts.
- At the C ABI boundary, use status codes + `margen_last_error` (see `margen_ffi`).

## Porting from Unreal

- Preserve **algorithm and pipeline behavior**.
- Express code in **idiomatic Rust** (snake_case APIs, modules within a crate).
- **Unreal module ≡ Cargo crate** (minimum). Internal UE folders become Rust modules inside that crate. If a UE module was already a catch-all (MythicSimulation), **split into focused crates** instead of one `*_simulation` bag.
- **Forbidden:** collapsing multiple Unreal modules into one crate; inventing umbrella/facade crates (`mythic`, `margen`). Extra focused crates are welcome.
- Intentional logic fixes use documented [improvement classes](../AGENTS.md#logic-improvement-classes). Propose new classes before first use.
- **Friction rule:** If an Unreal construct forces awkward Rust (UObject graphs, float-keyed maps, polymorphic editor subclasses, shared mutable bags that will not stage cleanly), **stop and report** instead of inventing a forced workaround.

## Naming and layout

- Crates live under `crates/` with **snake_case** package names (`mythic_dice`, `margen_generation_graphing`, `margen_ffi`, …).
- Public C header: [`include/margen.h`](../include/margen.h).
- Unix (LF) line endings; follow `.editorconfig`.
