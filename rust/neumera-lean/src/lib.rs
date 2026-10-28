//! Lean integration layer for neumera.
//!
//! This crate will own the Lean runtime connection, the wrapper types that
//! carry established properties on concrete data, and the contract-manifest
//! consumer that ties each guarantee-bearing rule to a Lean contract checked
//! in the same build. It has no Python dependency; the PyO3 adapter lives in
//! the `neumera-py` crate.
//!
//! Populated in the first implementation slice. See
//! `docs/design-example-sorted-vectors.md` in the repository.
