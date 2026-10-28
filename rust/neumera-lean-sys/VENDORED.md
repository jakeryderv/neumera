# Vendored from lean-rs

This crate is a vendored copy of two crates from `jcreinhold/lean-rs`, taken through the project's fork `jakeryderv/lean-rs`:

| Upstream crate | Upstream commit | Fork tag |
|---|---|---|
| `lean-rs-sys` 0.9.4 | `d2dc03b4c53d6c811d5bef1fe6d481a110c374d1` (2026-10-13) plus fork commit `8ec5664865f256b5cd71aa36d88c6bec199a3a7e` | `v0.9.4+neumera.1` |
| `lean-rs-abi` 0.9.4 | same | same |

Upstream licence: MIT OR Apache-2.0, copyright Jacob Reinhold. The licence texts are `LICENSE-MIT` and `LICENSE-APACHE` in this directory and apply to this crate. The rest of the repository is Apache-2.0.

Vendored 2026-10-28. The fork repository is kept as a reference and is no longer a dependency.

## What was kept

`src/`: every FFI module (`array`, `closure`, `ctor`, `external`, `init`, `io`, `memory`, `nat_int`, `object`, `refcount`, `repr`, `scalar`, `string`, `types`) unchanged apart from the crate name in doc comments. `src/symbols.rs` is `lean-rs-abi`'s `REQUIRED_SYMBOLS` table. `src/consts.rs` holds `lean-rs-abi`'s layout constants as literals plus the build-time toolchain identity. Tests `layout.rs`, `linkage.rs`, `safety_grep.rs`.

## What was changed

- **One pinned Lean version instead of a supported window.** Upstream matched the installed `lean.h` against a table of digests covering several releases. Here `build.rs` resolves the toolchain named in the repository's `lean-toolchain`, requires its `LEAN_VERSION_STRING` to equal the pin, and requires the SHA-256 of its `lean.h` to equal `lean.h.sha256` next to this crate. The `SupportedToolchain` table, `lean-rs-abi`'s query functions, `LEAN_RESOLVED_VERSION`, and the `lean_v_*` / `lean_at_least_*` cfg flags are gone.
- **Discovery is pin-driven.** `build.rs` runs `lean --print-prefix` with `ELAN_TOOLCHAIN` set to the pin, so it does not depend on the current directory, `LEAN_SYSROOT`, `ELAN_HOME` lookups, or a fixture package. This is what removed the duplicated pin in `.cargo/config.toml`.
- **`links = "lean"`**, publishing `cargo:prefix`, `cargo:version`, `cargo:header_digest` (`DEP_LEAN_PREFIX`, `DEP_LEAN_VERSION`, `DEP_LEAN_HEADER_DIGEST` in dependents). Upstream used `links = "leanshared"` and the fork added the metadata.
- **Features**: `dynamic` (default) and `static`, mutually exclusive and checked. The no-op `mimalloc` and `metadata-only` features and the docs.rs branch are gone.
- **Dropped**: `trybuild` compile-fail snapshots (platform-sensitive, upstream gated them behind an env var), the `digests/manifest.json` mirror, upstream README.

## Bumping Lean

1. Change `lean-toolchain` at the repository root.
2. Build. `neumera-lean-sys` fails with the new header's digest.
3. Diff the new `lean.h` against the previous one, paying attention to the `static inline` functions and the structs that `src/repr.rs` mirrors (object header, array, scalar array, string, closure, external, task, thunk, ref, ctor). Upstream's `supported.rs` history records what changed at each release and is a useful cross-check; so is the installed header of the previous toolchain.
4. Update the mirrors if needed, then write the new digest to `lean.h.sha256`.
5. Run `just check` and `just negative-tests`.

## Comparing with upstream

`~/dev/projects/lean-rs` (the fork clone) can fetch upstream read-only. Diff `crates/lean-rs-sys/src` against `src/` here, ignoring the crate name, to see what upstream changed since the vendored commit.
