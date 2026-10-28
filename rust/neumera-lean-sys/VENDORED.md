# Vendored from lean-rs

This crate is a vendored copy of two crates from `jcreinhold/lean-rs`, taken through the project's fork `jakeryderv/lean-rs`:

| Upstream crate | Upstream commit | Fork tag |
|---|---|---|
| `lean-rs-sys` 0.9.4 | `d2dc03b4c53d6c811d5bef1fe6d481a110c374d1` (2026-10-13) plus fork commit `8ec5664865f256b5cd71aa36d88c6bec199a3a7e` | `v0.9.4+neumera.1` |
| `lean-rs-abi` 0.9.4 | same | same |

Upstream licence: MIT OR Apache-2.0, copyright Jacob Reinhold. The licence texts are `LICENSE-MIT` and `LICENSE-APACHE` in this directory and apply to this crate. The rest of the repository is Apache-2.0.

Vendored 2026-10-28. The fork repository is kept as a reference and is no longer a dependency.

## What was kept

`src/`: every FFI module (`array`, `closure`, `ctor`, `external`, `init`, `io`, `memory`, `nat_int`, `object`, `refcount`, `repr`, `scalar`, `string`, `types`) is token-identical to upstream after the crate-name rename and this repository's rustfmt settings (different line width, so the diff is whitespace only). `src/symbols.rs` is `lean-rs-abi`'s `REQUIRED_SYMBOLS` table, byte-identical. Tests `linkage.rs` and `safety_grep.rs` (the latter now lists every file in `src/`; upstream's list skipped `ctor.rs` and `memory.rs`).

## What was changed

- **One pinned Lean version instead of a supported window.** Upstream matched the installed `lean.h` against a table of digests covering several releases. Here `build.rs` resolves the toolchain named in the repository's `lean-toolchain`, requires its `LEAN_VERSION_STRING` to equal the pin, and requires the SHA-256 of its `lean.h` to equal `lean.h.sha256` next to this crate. The `SupportedToolchain` table, `lean-rs-abi`'s query functions, `LEAN_RESOLVED_VERSION`, and the `lean_v_*` / `lean_at_least_*` cfg flags are gone.
- **Discovery is pin-driven.** `build.rs` runs `lean --print-prefix` with `ELAN_TOOLCHAIN` set to the pin, so it does not depend on the current directory, elan directory overrides, `LEAN_SYSROOT`, or a fixture package (elan still finds its toolchains through `ELAN_HOME`, which is a rerun input). The pin file is `../../lean-toolchain` relative to the crate, not the nearest ancestor. `neumera-lean` runs Lake with the same `ELAN_TOOLCHAIN` setting. This is what removed the duplicated pin in `.cargo/config.toml`.
- **`links = "lean"`**, publishing `cargo:prefix`, `cargo:version`, `cargo:header_digest` (`DEP_LEAN_PREFIX`, `DEP_LEAN_VERSION`, `DEP_LEAN_HEADER_DIGEST` in dependents). Upstream used `links = "leanshared"` and the fork added the metadata.
- **Features**: `dynamic` (default) and `static`, mutually exclusive and checked. `static` no longer links upstream's list (`Lean`, `leancpp`, `Lake`, system `stdc++`); it links the runtime only, `leanrt` and `Init` plus the toolchain's static `gmp`, `uv`, `c++`, `c++abi`, bundled into the rlib (Linux only so far). The crate also publishes `cargo:link_mode`. The no-op `mimalloc` and `metadata-only` features and the docs.rs branch are gone.
- **Rewritten**: `src/lib.rs` (crate docs, module list without `supported`, export list), `src/consts.rs` (`lean-rs-abi`'s constants inlined as literals plus the build-time toolchain identity), `tests/layout.rs` (the two supported-window tests replaced by a check that `LEAN_HEADER_DIGEST` equals `lean.h.sha256`).
- **Dropped**: `trybuild` compile-fail snapshots (platform-sensitive, upstream gated them behind an env var), the `digests/manifest.json` mirror, upstream README.

## Bumping Lean

Only release pins (`leanprover/lean4:vX.Y.Z`, release candidates included) are accepted; channel pins such as `stable` fail with an explanation, since the digest file records one release's header.

1. Change `lean-toolchain` at the repository root. If that toolchain is not installed, the first `cargo build` makes elan download it inside the build script with no visible progress; run `elan toolchain install <pin>` first to see it happen.
2. Build. If the new toolchain's `lean.h` differs, `neumera-lean-sys` fails and prints its digest. If it is byte-identical (Lean groups a release with its last candidates this way), the digest gate passes and only the version gate applies; nothing in `src/` can have changed meaning, so steps 3 and 4 are moot.
3. Diff the new `lean.h` against the previous one, paying attention to the `static inline` functions and the structs that `src/repr.rs` mirrors (object header, array, scalar array, string, closure, external, task, thunk, ref, ctor). Upstream's `supported.rs` history records what changed at each release and is a useful cross-check; so is the installed header of the previous toolchain.
4. Update the mirrors if needed, then write the new digest to `lean.h.sha256`.
5. Run `just check` and `just negative-tests`.

## Comparing with upstream

`~/dev/projects/lean-rs` (the fork clone) can fetch upstream read-only. Diff `crates/lean-rs-sys/src` against `src/` here, ignoring the crate name, to see what upstream changed since the vendored commit.
