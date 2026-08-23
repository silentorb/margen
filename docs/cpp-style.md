# C++ style guide

Shared style for **margen**, **margen-godot**, and any native C++ added in the marloth workspace. Expand this document as more Unreal patterns are codified.

Sibling repos should link here rather than duplicating rules.

## Orientation

Prefer **“C++ without classes”** over “C with classes.”

Functional programming inspires this: **immutability** and **separating data from behavior** improve structure even when advanced FP techniques (e.g. currying) are impractical in C++.

## Data vs functions

- Prefer **plain data** (`struct` with public fields) plus **free utility functions** that take and return those types.
- Avoid member functions except the most common / intrinsic cases (e.g. a few operations on core atomic types, or truly inseparable helpers).
- Do **not** grow types by adding methods in every dependent module—dependents add their own free functions instead (open for extension without editing the type).
- Limit **`class`** mostly to **containers** or **very core atomic types** (e.g. `Dice`). Domain / generation types should stay plain data + free functions.

## Access control and const

- Prefer **public data**. Heavy use of `private` / `protected` to “protect” callers from state is a smell and often signals larger architectural problems. Push state management as far down the stack as practical instead.
- Prefer **`const`** and immutable values for real guarantees—far more useful than hiding fields. Good architecture makes liberal `const` natural.
- **Do not use `friend`.** Prefer free functions in the same namespace (or public data + free operators).

Low-level container storage (`InlineVector` / `FixedArray` buffers) may remain implementation-private for now; that is not a model for domain types.

## Collections

- Prefer **homogeneous** collections and **normalized** data (SQL-like): break work into steps that iterate batched, same-shaped records.
- **Heterogeneous** collections or interfaces are allowed when they are clearly cleaner, but must **not** be the default for new problems.

## Project layout and naming

- Colocate `.h` and `.cpp` in the same directories (no separate `include/` tree). See [AGENTS.md](../AGENTS.md).
- Use `.h` for headers (not `.hpp`).
- Tabs for C++ indentation; Unix (LF) line endings.
- Near-copy Unreal ports: mirror folder/file names and identifier spelling; change only engine/STL substitutions. See [overview.md](overview.md) and [AGENTS.md](../AGENTS.md).
- Logic fixes while porting: use documented [improvement classes](../AGENTS.md#logic-improvement-classes); propose new classes before first use.

## Scope of retrofit

Apply these rules to **new code** immediately. Existing ports that still use member functions or access specifiers may be cleaned up when those files are next touched, or in a dedicated pass—do not block unrelated work on a mass rewrite.
