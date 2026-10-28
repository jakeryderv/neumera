//! Static link mode: the Lean runtime lives inside the test binary, so no
//! library from the toolchain may be mapped into the process while Lean code
//! runs. Compiled only with `--no-default-features --features static`.
//!
//! The check reads `/proc/self/maps` after calling into Lean and looks for
//! the resolved toolchain prefix (baked in by the build script) and for
//! `libleanshared` under any path. It inspects what is mapped at that moment,
//! which is what matters for a process that has already run Lean code; it does
//! not parse the ELF, so CI's `static-link` job is the place for an `ldd`
//! level check if one is ever wanted.

#![cfg(feature = "static")]

use neumera_lean::{IntVec, runtime};

const TOOLCHAIN_PREFIX: &str = env!("NEUMERA_LEAN_TOOLCHAIN_PREFIX");

#[test]
fn lean_runs_without_any_toolchain_library() {
    assert_eq!(runtime::init(), Ok(()));
    let a = IntVec::from_slice(&[1, 2]).unwrap();
    let b = IntVec::from_slice(&[3]).unwrap();
    assert_eq!(a.concat(&b).to_vec().unwrap(), vec![1, 2, 3]);

    let maps = std::fs::read_to_string("/proc/self/maps").expect("Linux only");
    let toolchain: Vec<&str> = maps
        .lines()
        .filter(|l| l.contains(TOOLCHAIN_PREFIX) || l.contains("libleanshared"))
        .collect();
    assert!(
        toolchain.is_empty(),
        "toolchain libraries are mapped into the process (prefix {TOOLCHAIN_PREFIX}): {toolchain:#?}"
    );
}
