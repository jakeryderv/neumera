# Binding layer evaluation

Decision record for how the Rust integration crate reaches the Lean runtime. Written during the first implementation slice (2026-10-28) against Lean `v4.35.0`.

## Decision

Use **`lean-rs-sys`** (version 0.9.4), the raw bindings crate, with its default dynamic link against `libleanshared`. Do not use its safe companion `lean-rs`. Write the ownership layer, wrapper types, and build orchestration in this repository.

**Update 2026-10-28:** the crate is now vendored as `rust/neumera-lean-sys` (see *Source of the bindings*). The evaluation below is kept as the record of why this binding was chosen over the alternatives.

## What was evaluated

Three options, assessed on runtime initialization, ownership, conversions, error handling, build integration, and version coupling.

### 1. `lean-rs-sys` 0.9.4 (chosen)

Raw FFI bindings for the Lean 4 C ABI, by Jacob Reinhold. Published 2026-10-13, the day after Lean 4.35.0.

- **Version support.** Declares a supported window of 4.30.0 through 4.35.0. The build script computes a SHA-256 of the installed `lean.h` and fails with a bounded diagnostic if it is outside the window. The installed 4.35.0 header matched, so the pinned toolchain needed no workaround.
- **What it provides.** `extern` declarations for the runtime (`lean_initialize_runtime_module`, `lean_io_mark_end_initialization`, `lean_dec_ref_cold`, and so on) plus Rust reimplementations of the `static inline` helpers from `lean.h` that cannot be linked: boxing, reference counting, array layout access, scalar and big-integer dispatch, IO result inspection. Every function is `unsafe` and documents its invariant. `lean_object` is opaque; layout is reached only through the helpers.
- **Runtime initialization.** Declarations only. The sequence (runtime module, then the Lean-generated `initialize_<pkg>_<Module>`, then end-of-initialization) is the caller's responsibility. This crate does it once per process behind a `OnceLock` and converts an IO error result into `Error::Init` after printing Lean's own description.
- **Ownership.** Helpers only (`lean_inc`, `lean_dec`). The owned-argument convention for `@[export]` functions is this crate's policy: retain before each call, release on drop, confirmed against the 4.35.0 FFI reference, which states that `@[export]` parameters are always owned.
- **Conversions.** `lean_int64_to_int` dispatches between inline scalars and heap big integers; `lean_scalar_to_int64` reads scalars. It does not declare `lean_int64_of_big_int`, so this crate declares that one function itself and range-checks with `lean_int_big_le` before decoding. Array construction uses `lean_alloc_array` and `lean_array_set_core`.
- **Error handling.** None beyond IO result predicates; appropriate for a sys crate.
- **Build integration.** Discovers Lean via `lean --print-prefix` (or `LEAN_SYSROOT`) and emits link directives: by default `-lleanshared` plus an rpath for its own targets. A `static` feature exists but links `libLean` and `libLake` as well, which this project does not need. The elan proxy resolves the toolchain from the working directory, so the pin lives at the repository root as `lean-toolchain`, with `lean/lean-toolchain` a symlink to it, and every `lean` or `lake` invocation inside the tree resolves the same version. As a registry or git dependency the `lean-rs-sys` build script ran outside the tree, which required the pin to be repeated in `.cargo/config.toml`; vendoring removed that (see *Source of the bindings*).
- **Cost.** One dependency plus `lean-rs-abi` (metadata) and `sha2` (build-time). A few seconds of compile time on a cold target directory, nothing warm.

### 2. `lean-rs` 0.9.4 (not chosen)

The safe front door over `lean-rs-sys`, with a companion `lean-toolchain` build-script helper.

- **What it provides.** `LeanRuntime::init()`, RAII `Obj<'lean>` handles whose lifetime is tied to a runtime borrow, `IntoLean`/`TryFromLean` conversions (including `Vec<T>` and `i64`, which refuses heap big integers), a `LeanCapability` that loads a Lake-built **shared** library at run time through `libloading`, looks up `@[export]` symbols by name, and dispatches typed calls checked against signature metadata recorded by `build.rs`. Structured errors with diagnostic codes. Module initializer naming for both the pre-4.27 and current schemes.
- **Why not.** Its model is dynamic loading of a shared library discovered through its own capability manifest, with `Obj<'lean>` lifetimes that would appear on every public type in this crate. This project's design is static linking of the Lake-built library with exports called directly, plain owned wrapper types, and a build script that already has to run Lake, emit and validate the contract manifest, and generate code. Adopting `lean-rs` would mean either two build orchestrations or bending the contract manifest around `lean-toolchain`'s capability manifest. The `i64` decoder also rejects heap integers outright, where this crate wants the full `i64` range.
- **What it got right that this crate copied.** Initialization is idempotent on the Lean side, so no Rust-side flag is needed beyond `OnceLock`. Decoding refuses rather than truncates. The initializer symbol follows `initialize_<pkg>_<Module>` with underscores doubled in component names; for package `neumera` and module `Neumera` it is `initialize_neumera_Neumera`, confirmed with `nm` on the built library.

### 3. Hand-written FFI (not chosen)

About the same amount of code as what this crate writes on top of `lean-rs-sys`, plus reimplementations of the inline helpers (reference counting fast paths, boxing, array header layout, scalar range dispatch) and a header checksum guard of our own. The reimplementations are exactly the version-coupled part, and `lean-rs-sys` maintains them across a tested window with a checksum gate. With 4.35.0 supported there was no reason to carry that ourselves. Had the pin been unsupported, the comparison would have been between pinning a supported version (no code cost, one version behind) and writing roughly 150 lines of layout-dependent `unsafe` code; the former would still have been preferable unless a specific 4.35.0 feature were needed.

## Source of the bindings

The bindings are vendored as the workspace crate `rust/neumera-lean-sys`: `lean-rs-sys` 0.9.4 and its `lean-rs-abi` metadata crate, taken at commit `8ec5664` of the project's fork `jakeryderv/lean-rs` (upstream `d2dc03b` plus one fork patch) on 2026-10-28, under their MIT or Apache-2.0 licence. The crate's `VENDORED.md` records provenance, what was changed, and the Lean bump procedure.

Why vendored rather than depended on: the project intends to shape this layer (a runtime-only static link mode next, see `static-linking-spike.md`), the sys crate can read the repository's pin directly only when it lives in the tree, and the upstream project is a single-author monorepo whose other six crates this project does not use. The fork repository is kept as a reference for comparing with upstream and is no longer a dependency.

Changes from upstream in the vendored copy: a single pinned Lean version with a recorded `lean.h` digest replaces the supported-version window; toolchain discovery runs `lean --print-prefix` with `ELAN_TOOLCHAIN` set to the pin, so the pin lives in one file; `links = "lean"` publishes the resolved prefix and version to `neumera-lean`'s build script as `DEP_LEAN_PREFIX` and `DEP_LEAN_VERSION`.

## Consequences

See `static-linking-spike.md` for the measurement that settles the distribution question: the runtime (`leanrt` + `Init` + the toolchain's static `gmp`, `uv`, `c++`) links statically into a binary with no Lean dependency at run time.

- The Lean runtime is linked dynamically from the toolchain's `lib/lean/libleanshared.so`. This crate's tests and examples carry an rpath to it. Any downstream crate that produces a binary or `cdylib` must emit its own rpath or bundle the library; the Python wheel will have to solve this, and that is out of scope for this slice.
- A Lean bump is a reviewed change to the vendored bindings: the mirrored inline helpers must be re-checked against the new `lean.h` before its digest is recorded in `rust/neumera-lean-sys/lean.h.sha256`.
- Two toolchains pinned at the repository root: `rust-toolchain.toml` and `lean-toolchain`. `neumera-lean-sys` resolves the Lean pin itself and publishes the prefix and version it used as `links` metadata; the `neumera-lean` build script compares them with Lake's and with the pin, and fails if the bindings and the Lean library would come from different toolchains.
