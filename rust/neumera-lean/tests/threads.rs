//! Lean objects on Rust-spawned threads. Each thread must be attached to the
//! runtime before it allocates and detached when it exits; the crate does both
//! implicitly, so these tests check that construction, concatenation, reading,
//! and dropping all work on threads Lean did not create, serially and
//! concurrently. Every test initializes the runtime on its own thread before
//! spawning, so the spawned threads must attach and detach themselves. The
//! case where a spawned thread initializes is `init_on_spawned_thread.rs`,
//! which is its own executable because initialization is once per process.

use std::sync::{Arc, Barrier};
use std::thread;

use neumera_lean::{IntVec, runtime};

fn exercise(seed: i64) -> Vec<i64> {
    let a = IntVec::from_slice(&[seed, seed + 1, seed << 40]).unwrap();
    let b = IntVec::from_slice(&[-seed, i64::MAX - seed]).unwrap();
    let c = a.concat(&b);
    assert_eq!(c.len(), 5);
    let out = c.to_vec().unwrap();
    drop(a);
    drop(b);
    drop(c);
    out
}

#[test]
fn spawned_thread_can_use_lean() {
    runtime::init().unwrap();
    let out = thread::spawn(|| exercise(7)).join().unwrap();
    assert_eq!(out, vec![7, 8, 7 << 40, -7, i64::MAX - 7]);
}

#[test]
fn many_threads_concurrently() {
    runtime::init().unwrap();
    const THREADS: i64 = 16;
    let barrier = Arc::new(Barrier::new(THREADS as usize));
    let handles: Vec<_> = (1..=THREADS)
        .map(|seed| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let mut last = Vec::new();
                for round in 0..50 {
                    last = exercise(seed * 1000 + round);
                }
                last
            })
        })
        .collect();
    for (i, h) in handles.into_iter().enumerate() {
        let seed = i as i64 + 1;
        let expected_seed = seed * 1000 + 49;
        assert_eq!(h.join().unwrap()[0], expected_seed);
    }
}

#[test]
fn thread_that_exits_and_is_replaced() {
    runtime::init().unwrap();
    // Threads attach on first use and detach on exit; a fresh thread after
    // several have exited must still work.
    for seed in 0..8 {
        let out = thread::spawn(move || exercise(seed)).join().unwrap();
        assert_eq!(out[0], seed);
    }
}
