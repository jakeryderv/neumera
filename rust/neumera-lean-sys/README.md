# neumera-lean-sys

Raw FFI bindings for the Lean 4 C ABI, pinned to the Lean toolchain in the repository's `lean-toolchain`. Every function is `unsafe`; `neumera-lean` is the safe layer on top.

The build script resolves the pinned toolchain, checks its `lean.h` against `lean.h.sha256`, and links the Lean runtime one of two ways: feature `dynamic` (default) links `libleanshared` with an rpath into the toolchain; feature `static` links the runtime archives (`leanrt`, `Init`, and the toolchain's `gmp`, `uv`, `c++`, `c++abi`) into the rlib, so binaries need no toolchain at run time (Linux only so far; see `docs/static-linking-spike.md`). It publishes the resolved prefix, version, and link mode to dependents as `DEP_LEAN_PREFIX`, `DEP_LEAN_VERSION`, and `DEP_LEAN_LINK_MODE`.

Vendored from `lean-rs-sys` and `lean-rs-abi`; provenance, licence, changes, and the Lean bump procedure are in `VENDORED.md`.
