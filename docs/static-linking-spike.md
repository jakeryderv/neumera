# Static-linking spike

Date: 2026-10-28. Platform: Linux x86_64, Lean 4.35.0, Rust 1.99.

**Status:** adopted. The `static` feature of `neumera-lean-sys` (forwarded by `neumera-lean`) links exactly this set; `just test-static` and CI's `static-link` job exercise it. Measurements made while adopting it are in *Follow-up measurements* below.

## Question

Can a binary call compiled Lean code without the toolchain present at run time? Today `neumera-lean` links `libleanshared.so` with an rpath into `~/.elan`, which cannot ship in a wheel.

## Result

Yes. A throwaway Rust program linked the Lean runtime and the Neumera static library into a single executable, called `neumera_concat`, and ran in a bare environment (`env -i PATH=/usr/bin:/bin`) with no Lean toolchain on any library path.

| | |
|---|---|
| Binary size | 4.6 MB (release, opt-level 1) |
| Run-time dependencies (`ldd`) | `libc`, `libm`, `libgcc_s`, the dynamic loader |
| Archives linked | `libneumera_Neumera.a`; `libInit.a` and `libleanrt.a` in one link group; `libgmp.a`, `libuv.a`, `libc++.a`, `libc++abi.a` |

All archives come from the toolchain: `lib/lean/` holds `Init` and `leanrt`; `lib/` holds `gmp`, `uv`, `c++`, and `c++abi` as static archives. Nothing is taken from the system except libc.

## Findings

- **The runtime is `leanrt` plus `Init`.** `libleancpp.a`, `libLean.a` (298 MB), and `libLake.a` are not needed to call compiled Lean code; they are the compiler and build tool. The `static` feature of `lean-rs-sys` (inherited by the vendored `neumera-lean-sys`) links all of them, which is the wrong shape for this project. `digama0/lean-sys` links `Lean` and `leancpp` too, but its link-group ordering was the useful reference.
- **`Init` and `leanrt` reference each other**, so they go inside `--start-group`/`--end-group`. Without the group the link fails depending on archive order.
- **Much of `lean.h` is `static inline` and has no symbol.** The spike needed to mirror `lean_alloc_array`, `lean_array_size`, and `lean_dec_ref` against the object header layout (`i32 m_rc`, `u16 m_cs_sz`, `u8 m_other`, `u8 m_tag`, then for arrays `size_t m_size`, `size_t m_capacity`). `lean_mk_empty_array` and `lean_alloc_small` are also inline. Exported and usable directly: `lean_alloc_object`, `lean_array_push`, `lean_dec_ref_cold`, `lean_initialize_runtime_module`, `lean_io_mark_end_initialization`, and the module initializer `initialize_neumera_Neumera(uint8_t builtin)`.
- **Linking is independent of which sys crate declares the functions.** The spike used its own `extern "C"` block and build script. The owned sys crate can emit exactly these directives.

## Consequences for the owned sys crate

1. Static linking of `leanrt` + `Init` + the toolchain's `gmp`, `uv`, `c++`, `c++abi` is the `static` feature. `dynamic` (`libleanshared` with an rpath) stays the default for development builds until the Python extension has been linked statically; dropping it then is the simplification to make.
2. The header-digest gate stays: the mirrored inline helpers depend on the header layout, and the gate is what makes a Lean bump safe.
3. A PyO3 extension module is a shared object, not an executable; the same archives link into it. Position independence is confirmed below.

## Follow-up measurements (same day, while adopting the result)

- **No linker group is needed when the archives are linked from a crate.** rustc bundles each `static=` archive named by a build script into the crate's rlib, so the final link sees one archive and the `Init`/`leanrt` cycle resolves on its own. Link arguments from a dependency's build script do not reach dependents' binaries, so the group could not have been emitted from the sys crate anyway.
- **Every archive is position-independent.** `readelf -r` finds zero `R_X86_64_32`/`R_X86_64_32S` relocations in `libInit.a` (209,722 relocations), `libleanrt.a`, `libgmp.a`, `libuv.a`, `libc++.a`, `libc++abi.a`, and the Lake-built `libneumera_Neumera.a`. The same archives can link into a shared object.
- **Lean's mimalloc does not override `malloc`.** `libleanrt.a` defines `mi_malloc`, `mi_free`, and 186 other `mi_*` symbols, and no `malloc`, `free`, `calloc`, `realloc`, or `operator new`; `lean.h` calls `mi_malloc` by name under `LEAN_MIMALLOC`. A Python process keeps its own allocator and Lean objects live in mimalloc's heap; the two do not interpose on each other. A static test binary exports no allocator symbol.
- **Static test binaries** depend only on `libc`, `libm`, `libgcc_s`, and the loader, carry no `RUNPATH`, and pass under `env -i PATH=/usr/bin:/bin`.

## Open

- macOS: archive names and the `c++` linkage differ (`lean-sys` links `c++` and `c++abi` dynamically there). Not tested.
- Initialization in a Python extension: `lean_initialize_runtime_module` must run once per process and `lean_io_mark_end_initialization` after module init; the existing `runtime::init` design covers this but has only been exercised in executables.

## Spike source

Build script link directives (the only part that matters):

```rust
println!("cargo:rustc-link-search=native={neumera_lib}");
println!("cargo:rustc-link-search=native={prefix}/lib/lean");
println!("cargo:rustc-link-search=native={prefix}/lib");
println!("cargo:rustc-link-lib=static=neumera_Neumera");
println!("cargo:rustc-link-arg=-Wl,--start-group");
println!("cargo:rustc-link-lib=static=Init");
println!("cargo:rustc-link-lib=static=leanrt");
println!("cargo:rustc-link-arg=-Wl,--end-group");
for l in ["gmp", "uv", "c++", "c++abi"] { println!("cargo:rustc-link-lib=static={l}"); }
for l in ["pthread", "dl", "m", "rt"] { println!("cargo:rustc-link-lib=dylib={l}"); }
```

The program initialized the runtime and the module, built two arrays through `lean_alloc_object` and `lean_array_push`, called `neumera_concat`, and asserted the result length. It was not kept; this document is the record.
