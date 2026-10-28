//! Handles stored in thread-local storage.
//!
//! A thread detaches from the Lean runtime when its last handle and its
//! attachment slot are both gone, so a handle parked in another thread-local
//! stays valid through teardown: its destructor can still read it, build new
//! vectors, concatenate, and drop, whatever order the destructors run in.
//!
//! On glibc, thread-local destructors run in reverse order of registration,
//! and Rust registers a destructor on a key's first access. The tests below
//! initialize the user key before and after the crate's own thread-local is
//! first touched, so both orders are exercised there; elsewhere the order is
//! unspecified and the tests still have to pass.
//!
//! Every test initializes the runtime on its own thread before spawning, so
//! the spawned thread is never the initializing thread and therefore must
//! detach (`lean_finalize_thread`) on exit. Otherwise the thread that happens
//! to touch Lean first would decide whether the detach path is exercised.

use std::cell::RefCell;
use std::sync::mpsc::{Sender, channel};
use std::thread;

use neumera_lean::{IntVec, runtime};

/// A thread-local that owns a handle and, when the thread tears it down,
/// exercises every handle operation and reports the results.
struct Parked {
    vec: IntVec,
    report: Sender<Vec<i64>>,
}

impl Drop for Parked {
    fn drop(&mut self) {
        // Read the parked handle.
        let parked = self.vec.to_vec().unwrap();
        // Allocate new objects on a thread that may already have dropped its
        // attachment slot, concatenate through Lean, read the result, and drop
        // everything. Each step touches the Lean heap.
        let fresh = IntVec::from_slice(&[100, 1 << 40]).unwrap();
        let joined = self.vec.concat(&fresh);
        let cloned = joined.clone();
        drop(joined);
        let mut out = parked;
        out.extend(cloned.to_vec().unwrap());
        self.report.send(out).unwrap();
    }
}

thread_local! {
    static PARKED_FIRST: RefCell<Option<Parked>> = const { RefCell::new(None) };
    static PARKED_SECOND: RefCell<Option<Parked>> = const { RefCell::new(None) };
}

fn expected(seed: i64) -> Vec<i64> {
    vec![seed, seed + 1, seed, seed + 1, 100, 1 << 40]
}

#[test]
fn handle_parked_before_crate_thread_local_is_touched() {
    runtime::init().unwrap();
    // Register the user key's destructor first (it then runs *after* the
    // crate's slot destructor on glibc): the hard case.
    let (tx, rx) = channel();
    thread::spawn(move || {
        PARKED_FIRST.with(|p| assert!(p.borrow().is_none()));
        let vec = IntVec::from_slice(&[1, 2]).unwrap();
        PARKED_FIRST.with(|p| *p.borrow_mut() = Some(Parked { vec, report: tx }));
    })
    .join()
    .unwrap();
    assert_eq!(rx.recv().unwrap(), expected(1));
}

#[test]
fn handle_parked_after_crate_thread_local_is_touched() {
    runtime::init().unwrap();
    // The crate's slot is registered first, so on glibc the user key's
    // destructor runs before it: the easy case.
    let (tx, rx) = channel();
    thread::spawn(move || {
        let vec = IntVec::from_slice(&[5, 6]).unwrap();
        PARKED_SECOND.with(|p| *p.borrow_mut() = Some(Parked { vec, report: tx }));
    })
    .join()
    .unwrap();
    assert_eq!(rx.recv().unwrap(), expected(5));
}

#[test]
fn two_parked_handles_in_different_keys() {
    runtime::init().unwrap();
    let (tx, rx) = channel();
    thread::spawn(move || {
        PARKED_FIRST.with(|p| assert!(p.borrow().is_none()));
        let a = IntVec::from_slice(&[10, 11]).unwrap();
        let b = IntVec::from_slice(&[20, 21]).unwrap();
        PARKED_FIRST.with(|p| {
            *p.borrow_mut() = Some(Parked {
                vec: a,
                report: tx.clone(),
            })
        });
        PARKED_SECOND.with(|p| *p.borrow_mut() = Some(Parked { vec: b, report: tx }));
    })
    .join()
    .unwrap();
    let mut reports = vec![rx.recv().unwrap(), rx.recv().unwrap()];
    reports.sort();
    assert_eq!(reports, vec![expected(10), expected(20)]);
}

#[test]
fn many_threads_park_handles() {
    runtime::init().unwrap();
    let (tx, rx) = channel();
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let tx = tx.clone();
            thread::spawn(move || {
                let seed = 1000 * (i + 1);
                let vec = IntVec::from_slice(&[seed, seed + 1]).unwrap();
                PARKED_FIRST.with(|p| *p.borrow_mut() = Some(Parked { vec, report: tx }));
            })
        })
        .collect();
    drop(tx);
    for h in handles {
        h.join().unwrap();
    }
    let mut reports: Vec<_> = rx.iter().collect();
    reports.sort();
    let mut want: Vec<_> = (0..8).map(|i| expected(1000 * (i + 1))).collect();
    want.sort();
    assert_eq!(reports, want);
}
