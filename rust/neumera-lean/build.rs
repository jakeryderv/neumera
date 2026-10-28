//! Build script: builds the Lean library with Lake, runs the contract manifest
//! emitter in the same build, validates the manifest's identity against the
//! artifacts about to be linked, generates one Rust constant per contract, and
//! emits the link directives for the static library.
//!
//! The manifest checks that every guarantee-bearing Rust rule references a
//! Lean contract checked in this build, with the expected statement. It does
//! not verify that the Rust rule applies the contract correctly; see
//! `docs/design-example-sorted-vectors.md`, section 4.
//!
//! `NEUMERA_LEAN_MANIFEST` supplies a manifest to validate *instead of* the
//! one emitted by this build. It exists so the negative tests can prove that
//! the identity checks reject drift. The emitter still runs, and a supplied
//! manifest must match this build's emission in everything except the
//! recorded identity (toolchain, version, hashes): the contract metadata that
//! becomes Rust code is never taken from a supplied file.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;

/// Axioms a contract may depend on. Must agree with `axiomAllowlist` in
/// `lean/Manifest.lean`; the build fails if the two lists differ.
const AXIOM_ALLOWLIST: &[&str] = &["propext", "Classical.choice", "Quot.sound"];

/// The Lean namespace whose theorems are contracts.
const CONTRACTS_NAMESPACE: &str = "Neumera.Contracts";

/// Path of the Lake target that produces the static library.
const STATIC_LIB_TARGET: &str = "Neumera:static";

/// Upper bound for the manifest emitter, which normally takes about a second.
const EMITTER_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Deserialize)]
struct Manifest {
    schema: u32,
    hash_algorithm: String,
    lean_toolchain: String,
    lean_version: String,
    executable: Artifact,
    checked_artifacts: BTreeMap<String, Artifact>,
    dependencies: Artifact,
    axiom_allowlist: Vec<String>,
    contracts: Vec<Contract>,
}

#[derive(Deserialize)]
struct Artifact {
    path: String,
    hash: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
struct Contract {
    name: String,
    statement: String,
    export: String,
    axioms: Vec<String>,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("crate lives at <repo>/rust/neumera-lean");
    let lean_dir = repo.join("lean");
    let toolchain_file = repo.join("lean-toolchain");
    let lake_manifest = lean_dir.join("lake-manifest.json");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for p in [
        lean_dir.join("lakefile.toml"),
        lean_dir.join("Neumera.lean"),
        lean_dir.join("Neumera"),
        lean_dir.join("Manifest.lean"),
        toolchain_file.clone(),
        lake_manifest.clone(),
    ] {
        println!("cargo:rerun-if-changed={}", p.display());
    }
    for var in ["NEUMERA_LEAN_MANIFEST", "ELAN_TOOLCHAIN", "LEAN_SYSROOT"] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    // 1. Toolchain pin, and one toolchain for both Lake and the FFI bindings.
    //
    // `lean-rs-sys` discovers Lean through `LEAN_SYSROOT` or a plain
    // `lean --print-prefix` run from its own package directory, where this
    // repository's `lean-toolchain` file is not visible to elan. The root
    // `.cargo/config.toml` therefore sets `ELAN_TOOLCHAIN` for every process
    // Cargo runs, and this script checks that it equals the pin, so Lake and
    // the bindings cannot resolve different toolchains.
    let pinned_line = fs::read_to_string(&toolchain_file)
        .unwrap_or_else(|e| fail(format!("cannot read {}: {e}", toolchain_file.display())))
        .trim()
        .to_string();
    let pinned_version = pinned_line
        .rsplit(':')
        .next()
        .map(|v| v.trim_start_matches('v').to_string())
        .unwrap_or_default();
    match env::var("ELAN_TOOLCHAIN") {
        Ok(v) if v.trim() == pinned_line => {}
        Ok(v) => fail(format!(
            "ELAN_TOOLCHAIN is {v:?} but {} pins {pinned_line:?}; the two must agree (see .cargo/config.toml)",
            toolchain_file.display()
        )),
        Err(_) => fail(format!(
            "ELAN_TOOLCHAIN is not set. Run cargo from inside the repository so .cargo/config.toml applies, \
             or set ELAN_TOOLCHAIN={pinned_line}; it pins the toolchain for the `lean-rs-sys` build script too"
        )),
    }
    let lean_version_line = run(&lean_dir, "lake", &["env", "lean", "--version"]);
    let running_version = parse_lean_version(&lean_version_line);
    if running_version != pinned_version {
        fail(format!(
            "Lean toolchain mismatch: {} pins {pinned_version} but `lake env lean --version` reports {running_version}",
            toolchain_file.display()
        ));
    }
    let prefix_dir = canonical(&PathBuf::from(
        run(&lean_dir, "lake", &["env", "lean", "--print-prefix"]).trim(),
    ));
    if let Some(sysroot) = env::var_os("LEAN_SYSROOT") {
        let sysroot = canonical(&PathBuf::from(sysroot));
        if sysroot != prefix_dir {
            fail(format!(
                "LEAN_SYSROOT={} would link `lean-rs-sys` against a different Lean than Lake's {}",
                sysroot.display(),
                prefix_dir.display()
            ));
        }
    }

    // 2. Build the Lean library (incremental; Lake decides what is stale).
    run(&lean_dir, "lake", &["build"]);
    let static_lib = canonical(&PathBuf::from(
        run(&lean_dir, "lake", &["query", STATIC_LIB_TARGET]).trim(),
    ));
    if !static_lib.is_file() {
        fail(format!(
            "Lake reported static library {} but it does not exist",
            static_lib.display()
        ));
    }
    let lib_dir = static_lib.parent().unwrap().to_path_buf();
    let olean_dir = lib_dir.join("lean");

    // 3. A supplied manifest (negative tests) is identity-checked first, before
    //    the emitter imports anything: a tampered artifact must be rejected on
    //    its hash, not handed to Lean's module loader.
    let supplied = env::var_os("NEUMERA_LEAN_MANIFEST").map(|p| {
        let p = PathBuf::from(p);
        // Re-validate when the supplied file changes, not only its path.
        println!("cargo:rerun-if-changed={}", p.display());
        let m = read_manifest(&p);
        validate_identity(
            &m,
            &pinned_line,
            &running_version,
            &static_lib,
            &olean_dir,
            &lake_manifest,
            &lean_dir,
        );
        (p, m)
    });

    // 4. Emit this build's manifest. Always: the contract metadata and the
    //    module set come from here, whatever manifest was supplied.
    let fresh_path = lean_dir
        .join(".lake")
        .join("build")
        .join("contracts-manifest.json");
    run_with_timeout(
        &lean_dir,
        "lake",
        &[
            "env",
            "lean",
            "--run",
            "Manifest.lean",
            &fresh_path.to_string_lossy(),
            &static_lib.to_string_lossy(),
            &lake_manifest.to_string_lossy(),
            &toolchain_file.to_string_lossy(),
        ],
        EMITTER_TIMEOUT,
        &out_dir,
    );
    let fresh = read_manifest(&fresh_path);
    validate_identity(
        &fresh,
        &pinned_line,
        &running_version,
        &static_lib,
        &olean_dir,
        &lake_manifest,
        &lean_dir,
    );

    // 5. A supplied manifest must match this build's emission in everything
    //    but identity; it is then the one whose identity was validated above.
    let manifest = match supplied {
        None => fresh,
        Some((p, supplied)) => {
            if supplied.contracts != fresh.contracts {
                fail(format!(
                    "supplied manifest {} records contracts that differ from this build's; \
                     contract metadata is not taken from a supplied manifest",
                    p.display()
                ));
            }
            if supplied.axiom_allowlist != fresh.axiom_allowlist {
                fail(format!(
                    "supplied manifest {} records an axiom allowlist that differs from this build's",
                    p.display()
                ));
            }
            let supplied_modules: Vec<_> = supplied.checked_artifacts.keys().collect();
            let fresh_modules: Vec<_> = fresh.checked_artifacts.keys().collect();
            if supplied_modules != fresh_modules {
                fail(format!(
                    "manifest identity mismatch: supplied manifest {} records checked artifacts {supplied_modules:?} \
                     but this build checked {fresh_modules:?}",
                    p.display()
                ));
            }
            supplied
        }
    };
    if manifest.axiom_allowlist != AXIOM_ALLOWLIST {
        fail(format!(
            "axiom allowlist disagreement: Lean emitter has {:?}, build script has {AXIOM_ALLOWLIST:?}",
            manifest.axiom_allowlist
        ));
    }

    // 6. Contracts: re-audit axioms (cheap) and validate shape.
    if manifest.contracts.is_empty() {
        fail("manifest contains no contracts".to_string());
    }
    let prefix = format!("{CONTRACTS_NAMESPACE}.");
    for c in &manifest.contracts {
        let Some(short) = c.name.strip_prefix(&prefix) else {
            fail(format!(
                "contract {} is outside namespace {CONTRACTS_NAMESPACE}",
                c.name
            ))
        };
        if short.is_empty()
            || short.contains('.')
            || !short.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
        {
            fail(format!(
                "contract {} has an unsupported short name {short:?}",
                c.name
            ));
        }
        if c.export.is_empty()
            || !c
                .export
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            fail(format!(
                "contract {} has an invalid export symbol {:?}",
                c.name, c.export
            ));
        }
        for ax in &c.axioms {
            if ax == "sorryAx" {
                fail(format!(
                    "axiom audit failed: contract {} depends on sorryAx",
                    c.name
                ));
            }
            if !AXIOM_ALLOWLIST.contains(&ax.as_str()) {
                fail(format!(
                    "axiom audit failed: contract {} depends on {ax}, not in allowlist",
                    c.name
                ));
            }
        }
    }

    // 7. Generate contract constants.
    let generated = render_contracts(&manifest, &prefix);
    fs::write(out_dir.join("contracts.rs"), generated).unwrap();

    // 8. Link the static library. `lean-rs-sys` links the Lean runtime
    //    (`libleanshared`) and sets the rpath for its own targets; this crate's
    //    tests and examples need the rpath too.
    let lib_name = static_lib
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("lib"))
        .unwrap_or_else(|| {
            fail(format!(
                "unexpected static library name {}",
                static_lib.display()
            ))
        });
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static={lib_name}");
    let runtime_lib_dir = prefix_dir.join("lib").join("lean");
    // Applies to every binary this package links (unit and integration tests,
    // examples). Downstream crates that produce binaries must emit their own.
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        runtime_lib_dir.display()
    );
}

/// Identity: schema, toolchain, version, and every recorded artifact bound to
/// this build's output path with its bytes rehashed and compared.
fn validate_identity(
    manifest: &Manifest,
    pinned_line: &str,
    running_version: &str,
    static_lib: &Path,
    olean_dir: &Path,
    lake_manifest: &Path,
    lean_dir: &Path,
) {
    if manifest.schema != 1 {
        fail(format!(
            "manifest schema {} is not supported (expected 1)",
            manifest.schema
        ));
    }
    if manifest.hash_algorithm != "fnv1a64" {
        fail(format!(
            "manifest hash algorithm {:?} is not fnv1a64",
            manifest.hash_algorithm
        ));
    }
    if manifest.lean_toolchain != pinned_line {
        fail(format!(
            "manifest identity mismatch: manifest was produced with toolchain {:?}, pin is {pinned_line:?}",
            manifest.lean_toolchain
        ));
    }
    if manifest.lean_version != running_version {
        fail(format!(
            "manifest identity mismatch: manifest Lean version {} differs from running Lean {running_version}",
            manifest.lean_version
        ));
    }
    check_artifact("executable", &manifest.executable, static_lib, lean_dir);
    if manifest.checked_artifacts.is_empty() {
        fail("manifest identity mismatch: no checked artifacts recorded".to_string());
    }
    for (module, artifact) in &manifest.checked_artifacts {
        // `Neumera.Basic` → `<lib>/lean/Neumera/Basic.olean`. The path the
        // manifest records must be this one; the bytes are hashed from here.
        let mut expected = olean_dir.to_path_buf();
        for component in module.split('.') {
            expected.push(component);
        }
        expected.set_extension("olean");
        check_artifact(
            &format!("checked artifact {module}"),
            artifact,
            &expected,
            lean_dir,
        );
    }
    check_artifact(
        "dependencies (lake-manifest.json)",
        &manifest.dependencies,
        lake_manifest,
        lean_dir,
    );
}

/// The validated artifact must be the file this build produced at `expected`,
/// and its bytes must hash to what the manifest records. Also registers the
/// file as a build input, so changing it after validation reruns this script.
fn check_artifact(what: &str, artifact: &Artifact, expected: &Path, base: &Path) {
    println!("cargo:rerun-if-changed={}", expected.display());
    let recorded = canonical(&resolve(base, &artifact.path));
    let expected = canonical(expected);
    if recorded != expected {
        fail(format!(
            "manifest identity mismatch: {what} is recorded at {} but this build's output is {}",
            recorded.display(),
            expected.display()
        ));
    }
    let bytes = fs::read(&expected).unwrap_or_else(|e| {
        fail(format!(
            "manifest identity mismatch: cannot read {what} at {}: {e}",
            expected.display()
        ))
    });
    let actual = format!("{:016x}", fnv1a64(&bytes));
    if actual != artifact.hash {
        fail(format!(
            "manifest identity mismatch: {what} at {} hashes to {actual} but the manifest records {}. \
             The manifest does not describe the artifacts being linked; rebuild without NEUMERA_LEAN_MANIFEST.",
            expected.display(),
            artifact.hash
        ));
    }
}

fn read_manifest(path: &Path) -> Manifest {
    serde_json::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| fail(format!("cannot read manifest {}: {e}", path.display()))),
    )
    .unwrap_or_else(|e| fail(format!("manifest {} is not valid: {e}", path.display())))
}

fn render_contracts(manifest: &Manifest, prefix: &str) -> String {
    let mut out = String::new();
    out.push_str(
        "// Generated by build.rs from the contract manifest. Do not edit.\n\
         //\n\
         // Each constant is a contract that Lean checked in the build that produced\n\
         // the linked library. A guarantee-bearing rule references its constant; if\n\
         // the contract disappears from the manifest, the reference fails to compile.\n\n\
         /// A Lean contract checked in this build.\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct Contract {\n\
         \x20   /// Fully qualified Lean name of the contract theorem.\n\
         \x20   pub name: &'static str,\n\
         \x20   /// The contract statement, as Lean pretty-printed it.\n\
         \x20   pub statement: &'static str,\n\
         \x20   /// The executable export this contract describes.\n\
         \x20   pub export: &'static str,\n\
         \x20   /// Axioms the proof transitively depends on (all allowlisted).\n\
         \x20   pub axioms: &'static [&'static str],\n\
         }\n\n",
    );
    out.push_str(&format!(
        "/// Lean version that checked these contracts.\npub const LEAN_VERSION: &str = {:?};\n",
        manifest.lean_version
    ));
    out.push_str(&format!(
        "/// FNV-1a 64 hash of the linked static library, as recorded by the manifest.\npub const EXECUTABLE_HASH: &str = {:?};\n\n",
        manifest.executable.hash
    ));
    let mut all = Vec::new();
    for c in &manifest.contracts {
        let short = c.name.strip_prefix(prefix).unwrap();
        let ident = short.to_uppercase();
        all.push(ident.clone());
        let axioms = c
            .axioms
            .iter()
            .map(|a| format!("{a:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        // The statement may span several lines, so it goes through a string
        // literal (`{:?}` escapes it) rather than a `///` line comment.
        let doc = format!("`{}`\n\n```text\n{}\n```", c.name, c.statement);
        out.push_str(&format!(
            "#[doc = {doc:?}]\npub const {ident}: Contract = Contract {{\n    name: {:?},\n    statement: {:?},\n    export: {:?},\n    axioms: &[{axioms}],\n}};\n\n",
            c.name, c.statement, c.export
        ));
    }
    out.push_str("/// Every contract in the manifest.\npub const ALL: &[&Contract] = &[");
    out.push_str(
        &all.iter()
            .map(|i| format!("&{i}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("];\n");
    out
}

/// FNV-1a, 64-bit. Must match `fnv1a64` in `lean/Manifest.lean`.
fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

fn resolve(base: &Path, p: &str) -> PathBuf {
    let p = PathBuf::from(p);
    if p.is_absolute() { p } else { base.join(p) }
}

/// Canonical form for path comparison; a path that does not exist is returned
/// as given (the caller then reports it as unreadable).
fn canonical(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// `Lean (version 4.35.0, x86_64-unknown-linux-gnu, commit ..., Release)` → `4.35.0`
fn parse_lean_version(line: &str) -> String {
    line.split("version ")
        .nth(1)
        .and_then(|rest| rest.split([',', ')']).next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| fail(format!("cannot parse Lean version from {line:?}")))
}

/// Like [`run`], but kills the process and fails if it has not finished in
/// time. Lean's module loader has been observed to hang on a corrupted
/// `.olean`; a hang must become a build failure, not a stuck CI job.
///
/// Output goes to files, not pipes: a child that fills a pipe blocks on write
/// and would look exactly like a hang until the timeout.
fn run_with_timeout(dir: &Path, program: &str, args: &[&str], timeout: Duration, log_dir: &Path) {
    let stdout_path = log_dir.join("emitter.stdout");
    let stderr_path = log_dir.join("emitter.stderr");
    let open = |p: &Path| {
        fs::File::create(p).unwrap_or_else(|e| fail(format!("cannot create {}: {e}", p.display())))
    };
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::from(open(&stdout_path)))
        .stderr(Stdio::from(open(&stderr_path)))
        .spawn()
        .unwrap_or_else(|e| {
            fail(format!(
                "cannot run `{program} {}` in {}: {e}",
                args.join(" "),
                dir.display()
            ))
        });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                fail(format!(
                    "`{program} {}` in {} did not finish within {}s and was killed. \
                     If the Lean build directory may be corrupt, run `lake clean` in {} and rebuild. \
                     Partial output: {} and {}",
                    args.join(" "),
                    dir.display(),
                    timeout.as_secs(),
                    dir.display(),
                    stdout_path.display(),
                    stderr_path.display()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => fail(format!("waiting for `{program}`: {e}")),
        }
    };
    if !status.success() {
        fail(format!(
            "`{program} {}` in {} failed ({status}):\n--- stdout ---\n{}\n--- stderr ---\n{}",
            args.join(" "),
            dir.display(),
            fs::read_to_string(&stdout_path).unwrap_or_default(),
            fs::read_to_string(&stderr_path).unwrap_or_default()
        ));
    }
}

fn run(dir: &Path, program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| {
            fail(format!(
                "cannot run `{program} {}` in {}: {e}. Is elan installed and `{program}` on PATH?",
                args.join(" "),
                dir.display()
            ))
        });
    if !output.status.success() {
        fail(format!(
            "`{program} {}` in {} failed ({}):\n--- stdout ---\n{}\n--- stderr ---\n{}",
            args.join(" "),
            dir.display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn fail(msg: String) -> ! {
    // Cargo shows `cargo:warning=` lines without `-vv`; the panic carries the
    // same text so the failure is visible either way.
    for line in msg.lines() {
        println!("cargo:warning=neumera-lean: {line}");
    }
    panic!("neumera-lean build failed: {msg}");
}
