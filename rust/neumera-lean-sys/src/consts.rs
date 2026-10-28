//! ABI constants.
//!
//! The static `lean.h` layout constants (object tags, allocator ceilings,
//! constructor-shape limits) are plain literals: they are facts of the Lean C
//! ABI that have not changed across the releases this crate has tracked. The
//! toolchain identity below is baked by `build.rs` from the `lean.h` this
//! crate was built against.

// Tag constants—`lean.h:83–95`.
pub const LEAN_MAX_CTOR_TAG: u8 = 243;
pub const LEAN_PROMISE: u8 = 244;
pub const LEAN_CLOSURE: u8 = 245;
pub const LEAN_ARRAY: u8 = 246;
pub const LEAN_STRUCT_ARRAY: u8 = 247;
pub const LEAN_SCALAR_ARRAY: u8 = 248;
pub const LEAN_STRING: u8 = 249;
pub const LEAN_MPZ: u8 = 250;
pub const LEAN_THUNK: u8 = 251;
pub const LEAN_TASK: u8 = 252;
pub const LEAN_REF: u8 = 253;
pub const LEAN_EXTERNAL: u8 = 254;
pub const LEAN_RESERVED: u8 = 255;

// Object-allocator constants—`lean.h:30–32`.
pub const LEAN_CLOSURE_MAX_ARGS: usize = 16;
pub const LEAN_OBJECT_SIZE_DELTA: usize = 8;
pub const LEAN_MAX_SMALL_OBJECT_SIZE: usize = 4096;

// Constructor-shape ceilings—`lean.h:97–98`.
pub const LEAN_MAX_CTOR_FIELDS: usize = 256;
pub const LEAN_MAX_CTOR_SCALARS_SIZE: usize = 1024;

/// `LEAN_VERSION_STRING` from the `lean.h` this crate was built against;
/// equal to the version named in the repository's `lean-toolchain`.
pub const LEAN_VERSION: &str = env!("LEAN_VERSION");

/// Filesystem path to the `lean.h` this crate was built against.
pub const LEAN_HEADER_PATH: &str = env!("LEAN_HEADER_PATH");

/// SHA-256 of that `lean.h`, lowercase hex; equal to `lean.h.sha256`.
pub const LEAN_HEADER_DIGEST: &str = env!("LEAN_HEADER_DIGEST");
