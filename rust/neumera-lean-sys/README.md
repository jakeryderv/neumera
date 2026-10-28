# neumera-lean-sys

Raw FFI bindings for the Lean 4 C ABI, pinned to the Lean toolchain in the repository's `lean-toolchain`. Every function is `unsafe`; `neumera-lean` is the safe layer on top.

The build script resolves the pinned toolchain, checks its `lean.h` against `lean.h.sha256`, and links `libleanshared` (feature `dynamic`, default). The `static` feature is inherited from upstream and does not yet produce a linkable binary on Linux; the runtime-only static mode is the next step (see `docs/static-linking-spike.md`). It publishes the resolved prefix and version to dependents as `DEP_LEAN_PREFIX` and `DEP_LEAN_VERSION`.

Vendored from `lean-rs-sys` and `lean-rs-abi`; provenance, licence, changes, and the Lean bump procedure are in `VENDORED.md`.
