//! Rust calls Lean: concatenation through the FFI, and the contract that
//! the build tied to it.

use neumera_lean::{Error, IntVec, contracts, runtime};

#[test]
fn runtime_initializes() {
    assert_eq!(runtime::init(), Ok(()));
    assert_eq!(runtime::init(), Ok(()), "init is idempotent");
}

#[test]
fn concat_through_lean() {
    let a = IntVec::from_slice(&[1, 2, 3]).unwrap();
    let b = IntVec::from_slice(&[4, 5]).unwrap();
    let c = a.concat(&b);
    assert_eq!(c.to_vec().unwrap(), vec![1, 2, 3, 4, 5]);
    // The contract `(concat a b).size = a.size + b.size`, observed rather than
    // assumed, because this is a test and not the rule.
    assert_eq!(c.len(), a.len() + b.len());
    // Inputs are borrowed, not consumed.
    assert_eq!(a.to_vec().unwrap(), vec![1, 2, 3]);
    assert_eq!(b.to_vec().unwrap(), vec![4, 5]);
}

#[test]
fn concat_with_empty() {
    let e = IntVec::from_slice(&[]).unwrap();
    let a = IntVec::from_slice(&[7]).unwrap();
    assert!(e.is_empty());
    assert_eq!(e.concat(&e).len(), 0);
    assert_eq!(e.concat(&a).to_vec().unwrap(), vec![7]);
    assert_eq!(a.concat(&e).to_vec().unwrap(), vec![7]);
}

#[test]
fn integers_beyond_scalar_range_round_trip() {
    // Lean boxes small ints inline and heap-allocates the rest; both paths
    // must round-trip, including the i64 extremes.
    let xs = [
        0,
        1,
        -1,
        i32::MAX as i64,
        i32::MIN as i64,
        1 << 40,
        -(1 << 40),
        i64::MAX,
        i64::MIN,
    ];
    let v = IntVec::from_slice(&xs).unwrap();
    assert_eq!(v.to_vec().unwrap(), xs.to_vec());
    let w = v.concat(&v);
    let mut expected = xs.to_vec();
    expected.extend_from_slice(&xs);
    assert_eq!(w.to_vec().unwrap(), expected);
}

#[test]
fn clone_shares_the_lean_object() {
    let a = IntVec::from_slice(&[1, 2]).unwrap();
    let b = a.clone();
    drop(a);
    assert_eq!(b.to_vec().unwrap(), vec![1, 2]);
}

#[test]
fn contract_metadata_is_what_the_rule_relies_on() {
    let c = contracts::CONCAT_LENGTH;
    assert_eq!(c.name, "Neumera.Contracts.concat_length");
    assert_eq!(c.export, "neumera_concat");
    assert!(c.statement.contains("size"), "statement: {}", c.statement);
    for ax in c.axioms {
        assert_ne!(*ax, "sorryAx");
        assert!(
            ["propext", "Classical.choice", "Quot.sound"].contains(ax),
            "unexpected axiom {ax}"
        );
    }
    assert!(contracts::ALL.contains(&&contracts::CONCAT_LENGTH));
    assert_eq!(contracts::LEAN_VERSION, "4.35.0");
    assert_eq!(contracts::EXECUTABLE_HASH.len(), 16);
}

#[test]
fn error_display() {
    assert_eq!(
        Error::Overflow { index: 3 }.to_string(),
        "element 3 does not fit in i64"
    );
}
