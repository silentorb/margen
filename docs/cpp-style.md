# C++ style guide (archived)

The living native style for **margen algorithms** is now [rust-style.md](rust-style.md).

The historical C++ port (CMake, Catch2, `mythic::` / `margen::` headers) is preserved on git branch **`archive/cpp-port`**.

C++ code that only **consumes** margen via the C ABI (e.g. future margen-godot GDExtension sources) should follow that host’s conventions; it does not need to mimic the old data-oriented C++ generation style.
