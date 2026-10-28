//! Build script for `neumera-lean-sys`.
//!
//! 1. Read the repository's `lean-toolchain` (two directories up from this
//!    crate), and resolve that toolchain's prefix by running
//!    `lean --print-prefix` with `ELAN_TOOLCHAIN` set to the pin. elan's proxy
//!    honours the variable ahead of any `lean-toolchain` file, so the result
//!    does not depend on the current directory.
//! 2. Require `LEAN_VERSION_STRING` in `<prefix>/include/lean/version.h` to
//!    equal the pinned version, and the SHA-256 of `<prefix>/include/lean/lean.h`
//!    to equal `lean.h.sha256` next to this crate. The mirrored inline helpers
//!    in `src/` are written against that header; a different header must be
//!    re-checked before the digest is updated.
//! 3. Emit `cargo:rustc-env=…` for `src/consts.rs`, and `cargo:prefix`,
//!    `cargo:version`, `cargo:header_digest`, `cargo:link_mode` metadata, which
//!    Cargo exposes to dependents' build scripts as `DEP_LEAN_PREFIX`,
//!    `DEP_LEAN_VERSION`, `DEP_LEAN_HEADER_DIGEST`, `DEP_LEAN_LINK_MODE` (this
//!    crate declares `links = "lean"`).
//! 4. Emit link directives for the selected feature: `dynamic` links
//!    `libleanshared` with an rpath into the toolchain; `static` links the
//!    runtime archives into the rlib so binaries need no toolchain at run time.
//! 5. Emit `cargo:rerun-if-*` for every input: the pin file, the digest file,
//!    `lean.h`, `version.h`, `PATH`, and `ELAN_HOME`.

// Build scripts panic to abort the build with a diagnostic; that is the
// correct failure mode here.
#![allow(clippy::panic, clippy::manual_assert)]

use sha2::{Digest, Sha256};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let dynamic = env::var_os("CARGO_FEATURE_DYNAMIC").is_some();
    let static_link = env::var_os("CARGO_FEATURE_STATIC").is_some();
    if dynamic == static_link {
        panic!(
            "neumera-lean-sys: exactly one of the features `dynamic` and `static` must be enabled"
        );
    }

    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // The repository root pin, two levels up (`<repo>/rust/neumera-lean-sys`).
    // Resolved explicitly rather than by searching ancestors, so a stray
    // `lean-toolchain` nearer this crate cannot change the result.
    let pin_file = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("crate lives at <repo>/rust/neumera-lean-sys")
        .join("lean-toolchain");
    if !pin_file.is_file() {
        panic!(
            "neumera-lean-sys: no lean-toolchain at {}",
            pin_file.display()
        );
    }
    let pin_line = fs::read_to_string(&pin_file)
        .unwrap_or_else(|err| {
            panic!(
                "neumera-lean-sys: cannot read {}: {err}",
                pin_file.display()
            )
        })
        .trim()
        .to_string();
    let pinned_version = pin_line
        .rsplit(':')
        .next()
        .map(|v| v.trim_start_matches('v'))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            panic!(
                "neumera-lean-sys: cannot parse a version from {pin_line:?} in {}",
                pin_file.display()
            )
        })
        .to_string();
    if !pinned_version.starts_with(|c: char| c.is_ascii_digit()) {
        panic!(
            "neumera-lean-sys: {} pins {pin_line:?}; only release pins of the form leanprover/lean4:vX.Y.Z are \
             supported, because lean.h.sha256 records the header of one release",
            pin_file.display()
        );
    }

    let prefix = resolve_prefix(&pin_line);
    let header_path = prefix.join("include").join("lean").join("lean.h");
    if !header_path.is_file() {
        panic!(
            "neumera-lean-sys: {} resolved to {} but it has no include/lean/lean.h",
            pin_line,
            prefix.display()
        );
    }

    let installed_version = read_lean_version(&prefix);
    if installed_version != pinned_version {
        panic!(
            "neumera-lean-sys: {} pins Lean {pinned_version} but the toolchain at {} reports {installed_version}",
            pin_file.display(),
            prefix.display()
        );
    }

    let digest_file = manifest_dir.join("lean.h.sha256");
    let expected_digest = fs::read_to_string(&digest_file)
        .unwrap_or_else(|err| {
            panic!(
                "neumera-lean-sys: cannot read {}: {err}",
                digest_file.display()
            )
        })
        .trim()
        .to_string();
    let actual_digest = sha256_file(&header_path);
    if actual_digest != expected_digest {
        panic!(
            "neumera-lean-sys: lean.h at {} has SHA-256 {actual_digest} but {} records {expected_digest}. \
             The inline helpers in src/ mirror lean.h; after a Lean bump, diff the header against the previous \
             one, re-check the mirrored helpers (see VENDORED.md), then record the new digest.",
            header_path.display(),
            digest_file.display()
        );
    }

    println!("cargo:rustc-env=LEAN_VERSION={installed_version}");
    println!("cargo:rustc-env=LEAN_HEADER_PATH={}", header_path.display());
    println!("cargo:rustc-env=LEAN_HEADER_DIGEST={actual_digest}");

    // Metadata for dependents (`DEP_LEAN_*` in their build scripts).
    println!("cargo:prefix={}", prefix.display());
    println!("cargo:version={installed_version}");
    println!("cargo:header_digest={actual_digest}");
    println!(
        "cargo:link_mode={}",
        if static_link { "static" } else { "dynamic" }
    );

    emit_link_directives(&prefix, static_link);

    println!("cargo:rerun-if-changed={}", pin_file.display());
    println!("cargo:rerun-if-changed={}", digest_file.display());
    println!("cargo:rerun-if-changed={}", header_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        prefix
            .join("include")
            .join("lean")
            .join("version.h")
            .display()
    );
    println!("cargo:rerun-if-changed=build.rs");
    // Any PATH difference (IDE vs shell, `just` vs direct cargo) reruns this
    // script and recompiles the two Lean crates. Accepted: PATH decides which
    // `lean` answers, and the version and digest gates must see the result.
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=ELAN_HOME");
}

fn resolve_prefix(pin_line: &str) -> PathBuf {
    let out = Command::new("lean")
        .arg("--print-prefix")
        .env("ELAN_TOOLCHAIN", pin_line)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "neumera-lean-sys: cannot run `lean --print-prefix`: {err}. Is ~/.elan/bin on PATH?"
            )
        });
    if !out.status.success() {
        panic!(
            "neumera-lean-sys: `lean --print-prefix` with ELAN_TOOLCHAIN={pin_line} exited {} (stderr: {})",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let prefix = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if prefix.is_empty() {
        panic!("neumera-lean-sys: `lean --print-prefix` printed nothing");
    }
    PathBuf::from(prefix)
}

fn read_lean_version(prefix: &Path) -> String {
    let version_h = prefix.join("include").join("lean").join("version.h");
    let text = fs::read_to_string(&version_h).unwrap_or_else(|err| {
        panic!(
            "neumera-lean-sys: cannot read {}: {err}",
            version_h.display()
        )
    });
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("#define LEAN_VERSION_STRING") {
            let trimmed = rest.trim().trim_matches('"');
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    panic!(
        "neumera-lean-sys: no LEAN_VERSION_STRING in {}",
        version_h.display()
    );
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path)
        .unwrap_or_else(|err| panic!("neumera-lean-sys: cannot read {}: {err}", path.display()));
    let digest = Sha256::digest(&bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn emit_link_directives(prefix: &Path, static_link: bool) {
    let lib_lean = prefix.join("lib").join("lean");
    let lib = prefix.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_lean.display());
    println!("cargo:rustc-link-search=native={}", lib.display());

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if static_link {
        // Runtime only: `leanrt` (allocator, objects, IO) and `Init` (the Lean
        // prelude compiled Lean code links against), plus the C libraries they
        // need, all taken from the toolchain's own static archives so nothing
        // but libc comes from the system. `Lean`, `leancpp`, and `Lake` are the
        // compiler and build tool and are not linked. See
        // docs/static-linking-spike.md for the measurements.
        //
        // rustc bundles every `static=` archive named here into this crate's
        // rlib, so the final link sees one archive and the mutual references
        // between `Init` and `leanrt` resolve without a linker group. Link
        // arguments would not propagate to dependents' binaries anyway.
        if target_os != "linux" {
            panic!(
                "neumera-lean-sys: the `static` feature is implemented for Linux only (target_os={target_os}); \
                 use the `dynamic` feature elsewhere"
            );
        }
        for lib in ["Init", "leanrt", "gmp", "uv", "c++", "c++abi"] {
            println!("cargo:rustc-link-lib=static={lib}");
        }
        for lib in ["pthread", "dl", "m", "rt"] {
            println!("cargo:rustc-link-lib=dylib={lib}");
        }
    } else {
        println!("cargo:rustc-link-lib=dylib=leanshared");
        // rpath for this crate's own binaries (tests). `cargo:rustc-link-arg`
        // applies only to the emitting package; dependents emit their own.
        if matches!(target_os.as_str(), "macos" | "linux") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_lean.display());
        }
    }
}
