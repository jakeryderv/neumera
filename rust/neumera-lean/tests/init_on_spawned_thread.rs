//! The first Lean call in the process happens on a Rust-spawned thread.
//!
//! Process initialization is once per process (`OnceLock`), so this scenario
//! cannot share an executable with tests that initialize on the main thread:
//! whichever ran first would decide the outcome. Cargo builds every file in
//! `tests/` into its own executable and runs it in a fresh process, so this
//! file holds exactly one test and nothing here calls into Lean before it.
//!
//! The thread that initializes the runtime is attached by the runtime itself.
//! It must not be attached a second time, and it must not be detached when it
//! exits (process teardown is the runtime's job). The test thread then uses
//! Lean after that thread has exited, and so does a further spawned thread,
//! which fails if the initializing thread's exit tore down anything the
//! process still needs.

use std::thread;

use neumera_lean::IntVec;

#[test]
fn spawned_thread_initializes_then_main_thread_uses_lean() {
    let out = thread::spawn(|| {
        let a = IntVec::from_slice(&[3, 4, 3 << 40]).unwrap();
        let b = IntVec::from_slice(&[-3, i64::MAX - 3]).unwrap();
        a.concat(&b).to_vec().unwrap()
    })
    .join()
    .unwrap();
    assert_eq!(out, vec![3, 4, 3 << 40, -3, i64::MAX - 3]);

    // The initializing thread has exited. The process must still be usable
    // from this thread and from further spawned threads.
    let a = IntVec::from_slice(&[1]).unwrap();
    assert_eq!(a.concat(&a).to_vec().unwrap(), vec![1, 1]);
    let out = thread::spawn(|| IntVec::from_slice(&[9, 8]).unwrap().to_vec().unwrap())
        .join()
        .unwrap();
    assert_eq!(out, vec![9, 8]);
}
