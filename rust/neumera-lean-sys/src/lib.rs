//! Raw FFI bindings for the Lean 4 C ABI.
//!
//! Vendored from `lean-rs-sys` (see `VENDORED.md`) and pinned to the single
//! Lean toolchain named in the repository's `lean-toolchain` file. Calling
//! any function in this crate is `unsafe`; `neumera-lean` is the safe layer.
//!
//! Raw functions inherit Lean's C ABI ownership rules:
//! - `lean_obj_arg` is **owned** (caller transfers a refcount).
//! - `b_lean_obj_arg` is **borrowed** (caller retains the refcount).
//! - `lean_obj_res` returns an **owned** reference.
//!
//! [`lean_object`] is intentionally opaque: it is zero-sized and `!Send +
//! !Sync + !Unpin`. Downstream code reaches refcount, tag, and payload state
//! only through this crate's `pub unsafe fn` helpers, never by reading
//! fields. The layout assumptions are pinned at build time: `build.rs`
//! computes the SHA-256 of the pinned toolchain's `include/lean/lean.h` and
//! requires it to equal the digest recorded in `lean.h.sha256` next to this
//! crate. A Lean bump therefore changes two files, `lean-toolchain` and
//! `lean.h.sha256`, and the build fails until the digest is deliberately
//! updated; `VENDORED.md` says what to re-check before doing so.
//!
//! Layering:
//! - Inline mirrors of `lean.h`'s `static inline` helpers live alongside the
//!   `extern "C"` declarations for the matching category (e.g. refcount,
//!   string, array). Each `pub unsafe fn` carries a `# Safety` section, each
//!   `unsafe { ... }` block carries a `// SAFETY:` comment.
//! - A crate-private `repr` module defines the Lean object layout
//!   (`LeanObjectRepr` and friends). These types are intentionally not
//!   re-exported.

#![allow(unsafe_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod array;
pub mod closure;
pub mod consts;
pub mod ctor;
pub mod external;
pub mod init;
pub mod io;
pub mod memory;
pub mod nat_int;
pub mod object;
pub mod refcount;
pub mod scalar;
pub mod string;
pub mod symbols;
pub mod types;

pub(crate) mod repr;

pub use consts::{LEAN_HEADER_DIGEST, LEAN_HEADER_PATH, LEAN_VERSION};
pub use symbols::{COMPILER_SYMBOLS, REQUIRED_SYMBOLS};
pub use types::{
    b_lean_obj_arg, b_lean_obj_res, lean_obj_arg, lean_obj_res, lean_object, u_lean_obj_arg,
};
