# Static-linking spike

Date: 2026-10-28. Platform: Linux x86_64, Lean 4.35.0, Rust 1.99.

**Status:** adopted for executables. The `static` feature of `neumera-lean-sys` (forwarded by `neumera-lean`) links exactly this set; `just test-static` and CI's `static-link` job exercise it. **It does not link into a shared object** with the toolchain's archives, so it is not yet a path to the Python extension; see *Follow-up measurements* and *Open*.

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

- **The runtime is `leanrt` plus `Init`.** `libleancpp.a`, `libLean.a` (298 MB), and `libLake.a` are not needed to call compiled Lean code; they are the compiler and build tool. The `static` feature of upstream `lean-rs-sys` linked all of them, which was the wrong shape for this project; the vendored `neumera-lean-sys` now links the runtime only. `digama0/lean-sys` links `Lean` and `leancpp` too, but its link-group ordering was the useful reference.
- **`Init` and `leanrt` reference each other**, so they go inside `--start-group`/`--end-group`. Without the group the link fails depending on archive order.
- **Much of `lean.h` is `static inline` and has no symbol.** The spike needed to mirror `lean_alloc_array`, `lean_array_size`, and `lean_dec_ref` against the object header layout (`i32 m_rc`, `u16 m_cs_sz`, `u8 m_other`, `u8 m_tag`, then for arrays `size_t m_size`, `size_t m_capacity`). `lean_mk_empty_array` and `lean_alloc_small` are also inline. Exported and usable directly: `lean_alloc_object`, `lean_array_push`, `lean_dec_ref_cold`, `lean_initialize_runtime_module`, `lean_io_mark_end_initialization`, and the module initializer `initialize_neumera_Neumera(uint8_t builtin)`.
- **Linking is independent of which sys crate declares the functions.** The spike used its own `extern "C"` block and build script. The owned sys crate can emit exactly these directives.

## Consequences for the owned sys crate

1. Static linking of `leanrt` + `Init` + the toolchain's `gmp`, `uv`, `c++`, `c++abi` is the `static` feature. `dynamic` (`libleanshared` with an rpath) stays the default for development builds until the Python extension has been linked statically; dropping it then is the simplification to make.
2. The header-digest gate stays: the mirrored inline helpers depend on the header layout, and the gate is what makes a Lean bump safe.
3. A PyO3 extension module is a shared object, not an executable. The archives are position-independent, but `libleanrt.a` uses the local-exec thread-local-storage model, which a shared object cannot contain; see below. The extension needs a different runtime build or a different packaging.

## Follow-up measurements (same day, while adopting the result)

- **No linker group is needed when the archives are linked from a crate.** rustc bundles each `static=` archive named by a build script into the crate's rlib, so the final link sees one archive and the `Init`/`leanrt` cycle resolves on its own. Link arguments from a dependency's build script do not reach dependents' binaries, so the group could not have been emitted from the sys crate anyway.
- **Every archive is position-independent, but `libleanrt.a` cannot go into a shared object.** `readelf -r` finds zero `R_X86_64_32`/`R_X86_64_32S` relocations in `libInit.a` (about 209,700 relocations), `libleanrt.a`, `libgmp.a`, `libuv.a`, `libc++.a`, `libc++abi.a`, and the Lake-built `libneumera_Neumera.a`. However `libleanrt.a` contains 307 `R_X86_64_TPOFF32` relocations, the local-exec TLS model, in eight objects (`mimalloc.cpp.o` 202, `interrupt.cpp.o` 38, `io.cpp.o` 21, `thread.cpp.o` 19, `stackinfo.cpp.o` 13, `object.cpp.o` 7, `alloc.cpp.o` 4, `memory.cpp.o` 3; variables such as `__mi_theap_default`, `lean_g_tls`, `g_heartbeat`). Both rust-lld and GNU ld refuse them with `-shared` ("relocation R_X86_64_TPOFF32 against lean_g_tls cannot be used with -shared"). A `cdylib` depending on `neumera-lean` in static mode therefore fails to link, found by the adversarial review of the adopting PR. `libleanshared.so` itself was built with the initial-exec model (`FLAGS STATIC_TLS`, 26 `R_X86_64_TPOFF64`), so Lean compiles the runtime differently for the shared build. No other archive has TLS relocations except two harmless `TLSLD`/`DTPOFF32` in `libc++abi.a`.
- **Lean's mimalloc does not override `malloc`.** `libleanrt.a` defines `mi_malloc`, `mi_free`, and 186 other `mi_*` symbols, and no `malloc`, `free`, `calloc`, `realloc`, or `operator new` (the only libc-named definition is a weak `free_sized`, the C23 name, which forwards to `free`; `libc++abi.a` defines weak `operator new`/`delete`, as every libc++abi does); `lean.h` calls `mi_malloc` by name under `LEAN_MIMALLOC`. A Python process keeps its own allocator and Lean objects live in mimalloc's heap; the two do not interpose on each other. A static test binary exports no allocator symbol.
- **Static test binaries** depend only on `libc`, `libm`, `libgcc_s`, and the loader, carry no `RUNPATH`, and pass under `env -i PATH=/usr/bin:/bin` and with `~/.elan` hidden by a mount namespace. The result holds with GNU ld as well as rust-lld.
- **`lean_initialize` is not in the runtime.** It initializes the compiler too and lives in `libleancpp`; the sys crate declares it under `dynamic` only. Compiled Lean code needs `lean_initialize_runtime_module`, which the runtime provides.
- **Not exercised:** C++ exceptions thrown through the runtime (`leanrt` references `__cxa_throw`; static mode unwinds through the system `libgcc_s`, where `libleanshared` carries its own unwinder).

## Open

- macOS: archive names and the `c++` linkage differ (`lean-sys` links `c++` and `c++abi` dynamically there). Not tested.
- **The Python extension cannot use this static mode as shipped.** Because of the local-exec TLS in `libleanrt.a`, a `cdylib` does not link. Options, none chosen yet: (a) rebuild `leanrt` from the Lean sources at the pinned tag with a shared-object-safe TLS model (`-ftls-model=initial-exec`, as `libleanshared` evidently is), 35 C++ files, and link that archive instead; (b) bundle `libleanshared.so` (231 MB, includes the compiler) in the wheel; (c) run Lean code in a statically linked sidecar executable and talk to it from Python. The static mode here stays correct for executables and tests.
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
