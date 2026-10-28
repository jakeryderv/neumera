//! Static link mode: the Lean runtime lives inside the test binary, so no
//! toolchain library may be mapped into the process while Lean code runs.
//! Compiled only with `--no-default-features --features static`.

#![cfg(feature = "static")]

use neumera_lean::{IntVec, runtime};

#[test]
fn lean_runs_without_any_toolchain_library() {
    assert_eq!(runtime::init(), Ok(()));
    let a = IntVec::from_slice(&[1, 2]).unwrap();
    let b = IntVec::from_slice(&[3]).unwrap();
    assert_eq!(a.concat(&b).to_vec().unwrap(), vec![1, 2, 3]);

    let maps = std::fs::read_to_string("/proc/self/maps").expect("Linux only");
    let toolchain: Vec<&str> = maps
        .lines()
        .filter(|l| l.contains("libleanshared") || l.contains("/.elan/"))
        .collect();
    assert!(
        toolchain.is_empty(),
        "toolchain libraries are mapped into the process: {toolchain:#?}"
    );
}
