//! Public-surface layout guarantees. The crate-private repr structs are
//! exercised by `#[cfg(test)] mod tests` blocks inside `src/repr.rs`.

// Tests assert structural invariants; panicking on failure is the desired
// behaviour.
#![allow(clippy::panic, clippy::expect_used)]

#[test]
fn opaque_lean_object_is_zero_sized() {
    assert_eq!(core::mem::size_of::<neumera_lean_sys::lean_object>(), 0);
}

#[test]
fn version_and_digest_constants_resolve() {
    assert!(!neumera_lean_sys::LEAN_VERSION.is_empty());
    assert!(!neumera_lean_sys::LEAN_HEADER_PATH.is_empty());
    assert_eq!(neumera_lean_sys::LEAN_HEADER_DIGEST.len(), 64);
}

#[test]
fn digest_constant_matches_the_recorded_pin() {
    let recorded = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/lean.h.sha256"))
        .expect("lean.h.sha256 is readable");
    assert_eq!(recorded.trim(), neumera_lean_sys::LEAN_HEADER_DIGEST);
}
