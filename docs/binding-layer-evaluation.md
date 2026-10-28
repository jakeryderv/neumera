# Binding layer evaluation

Decision record for how the Rust integration crate reaches the Lean runtime. Written during the first implementation slice (2026-10-28) against Lean `v4.35.0`.

## Decision

Use **`lean-rs-sys`** (version 0.9.4), the raw bindings crate, with its default dynamic link against `libleanshared`. Do not use its safe companion `lean-rs`. Write the ownership layer, wrapper types, and build orchestration in this repository.

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
- **Build integration.** Discovers Lean via `lean --print-prefix` (or `LEAN_SYSROOT`) and emits link directives: by default `-lleanshared` plus an rpath for its own targets. A `static` feature exists but links `libLean` and `libLake` as well, which this project does not need. The elan proxy resolves the toolchain from the working directory, so the pin lives at the repository root as `lean-toolchain`, with `lean/lean-toolchain` a symlink to it, and every `lean` or `lake` invocation inside the tree resolves the same version. The `lean-rs-sys` build script runs outside the tree (in Cargo's git checkout), which is why `.cargo/config.toml` also sets `ELAN_TOOLCHAIN`; see Consequences.
- **Cost.** One dependency plus `lean-rs-abi` (metadata) and `sha2` (build-time). A few seconds of compile time on a cold target directory, nothing warm.

### 2. `lean-rs` 0.9.4 (not chosen)

The safe front door over `lean-rs-sys`, with a companion `lean-toolchain` build-script helper.

- **What it provides.** `LeanRuntime::init()`, RAII `Obj<'lean>` handles whose lifetime is tied to a runtime borrow, `IntoLean`/`TryFromLean` conversions (including `Vec<T>` and `i64`, which refuses heap big integers), a `LeanCapability` that loads a Lake-built **shared** library at run time through `libloading`, looks up `@[export]` symbols by name, and dispatches typed calls checked against signature metadata recorded by `build.rs`. Structured errors with diagnostic codes. Module initializer naming for both the pre-4.27 and current schemes.
- **Why not.** Its model is dynamic loading of a shared library discovered through its own capability manifest, with `Obj<'lean>` lifetimes that would appear on every public type in this crate. This project's design is static linking of the Lake-built library with exports called directly, plain owned wrapper types, and a build script that already has to run Lake, emit and validate the contract manifest, and generate code. Adopting `lean-rs` would mean either two build orchestrations or bending the contract manifest around `lean-toolchain`'s capability manifest. The `i64` decoder also rejects heap integers outright, where this crate wants the full `i64` range.
- **What it got right that this crate copied.** Initialization is idempotent on the Lean side, so no Rust-side flag is needed beyond `OnceLock`. Decoding refuses rather than truncates. The initializer symbol follows `initialize_<pkg>_<Module>` with underscores doubled in component names; for package `neumera` and module `Neumera` it is `initialize_neumera_Neumera`, confirmed with `nm` on the built library.

### 3. Hand-written FFI (not chosen)

About the same amount of code as what this crate writes on top of `lean-rs-sys`, plus reimplementations of the inline helpers (reference counting fast paths, boxing, array header layout, scalar range dispatch) and a header checksum guard of our own. The reimplementations are exactly the version-coupled part, and `lean-rs-sys` maintains them across a tested window with a checksum gate. With 4.35.0 supported there was no reason to carry that ourselves. Had the pin been unsupported, the comparison would have been between pinning a supported version (no code cost, one version behind) and writing roughly 150 lines of layout-dependent `unsafe` code; the former would still have been preferable unless a specific 4.35.0 feature were needed.

## Source of the bindings

The two crates are consumed from the project's fork, `jakeryderv/lean-rs`, through `[patch.crates-io]` in `rust/Cargo.toml`, pinned to a commit. The version requirement stays as upstream numbers it (`0.9.4`); the patch only redirects where that version's source comes from, so removing the fork later is deleting the patch section. Fixes this project needs are carried in the fork rather than submitted upstream, tagged `v<upstream version>+neumera.<n>` so they never collide with upstream's numbering.

| Pinned commit | Fork tag | Patches relative to upstream |
|---|---|---|
| `8ec5664865f256b5cd71aa36d88c6bec199a3a7e` | `v0.9.4+neumera.1` | `lean-rs-sys` build script: `ELAN_TOOLCHAIN` is a rerun input; resolved prefix, Lean version, and `lean.h` digest emitted as `cargo:` metadata for dependents |

## Consequences

See `static-linking-spike.md` for the measurement that settles the distribution question: the runtime (`leanrt` + `Init` + the toolchain's static `gmp`, `uv`, `c++`) links statically into a binary with no Lean dependency at run time.

- The Lean runtime is linked dynamically from the toolchain's `lib/lean/libleanshared.so`. This crate's tests and examples carry an rpath to it. Any downstream crate that produces a binary or `cdylib` must emit its own rpath or bundle the library; the Python wheel will have to solve this, and that is out of scope for this slice.
- `lean-rs-sys` must support each Lean version this project pins. Bumping the pin means checking its supported window first.
- Two toolchains pinned at the repository root: `rust-toolchain.toml` and `lean-toolchain`. The Lean pin is repeated as `ELAN_TOOLCHAIN` in `.cargo/config.toml`, because `lean-rs-sys` runs `lean --print-prefix` from its own package directory where the pin file is not visible; the `neumera-lean` build script fails if the two values differ. The fork's `lean-rs-sys` reruns its build script when `ELAN_TOOLCHAIN` changes and publishes the prefix and version it resolved as `links` metadata (`DEP_LEANSHARED_PREFIX`, `DEP_LEANSHARED_LEAN_VERSION`); the `neumera-lean` build script compares them with Lake's and fails if the bindings and the Lean library would come from different toolchains.
