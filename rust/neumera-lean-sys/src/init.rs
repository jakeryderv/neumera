//! Runtime initialization and thread attachment.
//!
//! These symbols are defined by the Lean runtime but **not declared in
//! `lean.h`** (they are part of Lean's runtime entry-point surface, not the
//! object-ABI header). The `LEAN_HEADER_DIGEST` build check therefore does
//! not gate them; instead they are protected by `REQUIRED_SYMBOLS` plus
//! `tests/linkage.rs`, which fails to link if any are missing.
//!
//! [`lean_initialize`] is the exception: it initializes the compiler as well
//! as the runtime and lives in `libleancpp`, which only `libleanshared` links.
//! It exists under the `dynamic` feature only; the `static` feature links the
//! runtime alone, and programs that only call compiled Lean code use
//! [`lean_initialize_runtime_module`].

use core::ffi::c_char;

unsafe extern "C" {
    /// Initialize the Lean runtime **and the compiler**. Must be called at most
    /// once per process. Defined in `libleancpp`, hence `dynamic` only; see the
    /// module docs.
    #[cfg(feature = "dynamic")]
    pub fn lean_initialize();

    /// Initialize the runtime module (compact regions, persistent objects).
    /// Must be called once per process before invoking any compiled Lean
    /// module's `init` entry point; the full `lean_initialize` is not needed
    /// to run compiled code.
    pub fn lean_initialize_runtime_module();

    /// Attach the current OS thread to the Lean runtime so it may invoke
    /// compiled Lean code. Each thread that calls into Lean must pair this
    /// with [`lean_finalize_thread`] before exiting.
    pub fn lean_initialize_thread();

    /// Detach the current OS thread from the Lean runtime.
    pub fn lean_finalize_thread();

    /// Install the process command-line arguments for `System.Args`. `argv`
    /// must outlive any Lean code that reads the arguments.
    pub fn lean_setup_args(argc: i32, argv: *mut *mut c_char);

    /// Start the Lean task manager with a runtime-chosen number of worker
    /// threads (`lean.h:1272`).
    pub fn lean_init_task_manager();

    /// Start the Lean task manager with the requested number of worker
    /// threads (`lean.h:1273`).
    pub fn lean_init_task_manager_using(num_workers: u32);

    /// Stop the Lean task manager and join all workers (`lean.h:1274`).
    pub fn lean_finalize_task_manager();
}
