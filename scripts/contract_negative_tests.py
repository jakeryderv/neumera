#!/usr/bin/env python3
"""Negative tests for the contract manifest.

Each case mutates a scratch copy of the Lean and Rust sources in a way that
must make `cargo build -p neumera-lean` fail, and checks that the failure
names the expected cause. A baseline build of the unmodified scratch copy must
succeed first, so that a failure is attributable to the mutation.

Cases (see docs/design-example-sorted-vectors.md, section 4):

  1. Delete the underlying theorem.        -> contracts module fails to elaborate
  2. Weaken the underlying theorem.        -> contracts module fails to elaborate
  3. Rename the contract in the contracts  -> constant not generated; Rust rule
     module (Lean still builds).              fails to compile
  4. Pair a manifest with a wrong          -> identity check fails
     executable hash with a fresh build.
  5. Replace the proof with `sorry`.       -> axiom audit fails on sorryAx
  6. Wrong checked-artifact hash           -> identity check fails
     (executable hash correct).
  7. Wrong dependency hash                 -> identity check fails
     (executable hash correct).
  8. Checked artifact path points         -> identity check fails (path is
     somewhere else.                          not this build's output)
  9. Checked artifact omitted from the     -> identity check fails
     manifest.
 10. A valid manifest supplied through     -> build succeeds, then fails after
     NEUMERA_LEAN_MANIFEST is edited in       the file is edited at the same
     place.                                   path (rerun-if-changed works).
 11. Checked artifact path points at the  -> identity check fails even though
     static archive, with the archive's      the hash matches the file at the
     correct hash.                            recorded path
 12. Supplied manifest forges contract     -> rejected: contract metadata must
     metadata (statement, empty axioms)      equal this build's emission
     with every hash correct.
 13. After a valid supplied-manifest       -> build reruns validation and fails
     build, an .olean is tampered (the       (artifacts are build inputs)
     manifest is untouched).

 14. Corrupt the recorded lean.h digest     -> neumera-lean-sys fails naming the
     (rust/neumera-lean-sys/lean.h.sha256).    digest mismatch

 15. (opt-in) Switch the Lean pin to a      -> build fails on the digest gate;
     second installed toolchain whose          after recording that toolchain's
     lean.h differs, without cleaning the      lean.h digest the bindings rebuild
     target directory.                         and the build passes

Case 15 runs only when NEUMERA_ALT_TOOLCHAIN names a second installed toolchain
(for example `leanprover/lean4:v4.34.0`). Its lean.h must differ from the
pinned one; a release and its last candidates share a header, and the case
reports that it was skipped for such a pair.

Usage: python scripts/contract_negative_tests.py [--keep]
Standard library only. Needs cargo, lake, and lean on PATH.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BASIC = Path("lean/Neumera/Basic.lean")
CONTRACTS = Path("lean/Neumera/Contracts.lean")
MANIFEST = Path("lean/.lake/build/contracts-manifest.json")

THEOREM = """/-- The length of a concatenation is the sum of the input lengths. -/
theorem concat_length (a b : Array Int) : (concat a b).size = a.size + b.size := by
  simp [concat]"""


@dataclass
class Case:
    number: int
    title: str
    mutate: Callable[[Path, dict[str, str]], None]
    expect: str  # substring that must appear in the build output
    # Manifest cases tamper with the emitted manifest, so it must first be
    # regenerated against the current (restored) sources by a passing build.
    fresh_manifest: bool = False


def replace_in(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected exactly one occurrence of:\n{old}")
    path.write_text(text.replace(old, new))


def case_delete_theorem(root: Path, env: dict[str, str]) -> None:
    replace_in(root / BASIC, THEOREM, "-- theorem deleted by negative test")


def case_weaken_theorem(root: Path, env: dict[str, str]) -> None:
    replace_in(
        root / BASIC,
        THEOREM,
        "theorem concat_length (a b : Array Int) : (concat a b).size ≥ a.size := by\n  simp [concat]",
    )


def case_rename_contract(root: Path, env: dict[str, str]) -> None:
    replace_in(
        root / CONTRACTS, "theorem concat_length :", "theorem concat_length_renamed :"
    )


def tampered_manifest(
    root: Path, env: dict[str, str], tamper: Callable[[dict], None]
) -> Path:
    """Write a copy of the emitted manifest with `tamper` applied and point the
    build at it. The executable hash is left correct unless `tamper` changes it."""
    manifest = json.loads((root / MANIFEST).read_text())
    tamper(manifest)
    path = root / "tampered-manifest.json"
    path.write_text(json.dumps(manifest))
    env["NEUMERA_LEAN_MANIFEST"] = str(path)
    return path


def case_stale_manifest(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        m["executable"]["hash"] = "0000000000000000"

    tampered_manifest(root, env, tamper)


def case_wrong_artifact_hash(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        m["checked_artifacts"]["Neumera.Basic"]["hash"] = "0000000000000000"

    tampered_manifest(root, env, tamper)


def case_wrong_dependency_hash(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        m["dependencies"]["hash"] = "0000000000000000"

    tampered_manifest(root, env, tamper)


def case_missing_artifact_file(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        entry = m["checked_artifacts"]["Neumera.Basic"]
        entry["path"] = entry["path"].replace("Basic.olean", "Missing.olean")

    tampered_manifest(root, env, tamper)


def case_artifact_path_to_archive(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        entry = m["checked_artifacts"]["Neumera.Basic"]
        entry["path"] = m["executable"]["path"]
        entry["hash"] = m["executable"]["hash"]

    tampered_manifest(root, env, tamper)


def case_forged_contract_metadata(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        c = m["contracts"][0]
        c["statement"] = "False"
        c["axioms"] = []

    tampered_manifest(root, env, tamper)


def case_omitted_artifact(root: Path, env: dict[str, str]) -> None:
    def tamper(m: dict) -> None:
        del m["checked_artifacts"]["Neumera.Basic"]

    tampered_manifest(root, env, tamper)


def case_sorry(root: Path, env: dict[str, str]) -> None:
    replace_in(root / BASIC, "  simp [concat]", "  sorry")


DIGEST_FILE = Path("rust") / "neumera-lean-sys" / "lean.h.sha256"


def case_corrupt_digest(root: Path, _env: dict[str, str]) -> None:
    (root / DIGEST_FILE).write_text("0" * 64 + "\n")


CASES = [
    Case(
        1, "delete the underlying theorem", case_delete_theorem, "Neumera.concat_length"
    ),
    Case(
        2,
        "weaken the underlying theorem",
        case_weaken_theorem,
        "Neumera/Contracts.lean",
    ),
    Case(
        3,
        "rename the contract (Lean still builds)",
        case_rename_contract,
        "CONCAT_LENGTH",
    ),
    Case(
        4,
        "wrong executable hash in supplied manifest",
        case_stale_manifest,
        "manifest identity mismatch: executable",
        fresh_manifest=True,
    ),
    Case(5, "replace the proof with sorry", case_sorry, "sorryAx"),
    Case(
        6,
        "wrong checked-artifact hash",
        case_wrong_artifact_hash,
        "manifest identity mismatch: checked artifact Neumera.Basic",
        fresh_manifest=True,
    ),
    Case(
        7,
        "wrong dependency hash",
        case_wrong_dependency_hash,
        "manifest identity mismatch: dependencies",
        fresh_manifest=True,
    ),
    Case(
        8,
        "checked artifact file missing",
        case_missing_artifact_file,
        "manifest identity mismatch: checked artifact Neumera.Basic is recorded at",
        fresh_manifest=True,
    ),
    Case(
        9,
        "checked artifact omitted",
        case_omitted_artifact,
        "manifest identity mismatch: supplied manifest",
        fresh_manifest=True,
    ),
    Case(
        11,
        "checked artifact path points at the archive",
        case_artifact_path_to_archive,
        "manifest identity mismatch: checked artifact Neumera.Basic is recorded at",
        fresh_manifest=True,
    ),
    Case(
        12,
        "supplied manifest forges contract metadata",
        case_forged_contract_metadata,
        "records contracts that differ from this build's",
        fresh_manifest=True,
    ),
    Case(
        14,
        "corrupt the recorded lean.h digest",
        case_corrupt_digest,
        "has SHA-256",
    ),
]


def case_artifact_tampered_after_build(root: Path) -> tuple[bool, str, float]:
    """Case 13: a valid supplied manifest builds; then an .olean is altered
    without touching the manifest. Cargo must rerun validation, which fails."""
    code, out, secs0 = build(root, {})
    if code != 0:
        return False, "re-baseline before case 13 failed:\n" + out, secs0
    env: dict[str, str] = {}
    tampered_manifest(root, env, lambda m: None)
    code, out, secs = build(root, env)
    secs += secs0
    if code != 0:
        return False, "valid supplied manifest did not build:\n" + out, secs
    olean = root / "lean/.lake/build/lib/lean/Neumera/Contracts.olean"
    original = olean.read_bytes()
    try:
        olean.write_bytes(original + b"\n")
        code, out, secs2 = build(root, env)
    finally:
        olean.write_bytes(original)
    expect = "manifest identity mismatch: checked artifact Neumera.Contracts"
    return code != 0 and expect in out, out, secs + secs2


def case_override_rerun(root: Path) -> tuple[bool, str, float]:
    """Case 10: a valid supplied manifest builds; editing it in place at the
    same path must make the very next build re-validate and fail."""
    code, out, secs0 = build(root, {})
    if code != 0:
        return False, "re-baseline before case 10 failed:\n" + out, secs0
    env: dict[str, str] = {}
    path = tampered_manifest(root, env, lambda m: None)
    code, out, secs = build(root, env)
    secs += secs0
    if code != 0:
        return False, "valid supplied manifest did not build:\n" + out, secs
    manifest = json.loads(path.read_text())
    manifest["checked_artifacts"]["Neumera.Contracts"]["hash"] = "0000000000000000"
    path.write_text(json.dumps(manifest))
    code, out, secs2 = build(root, env)
    expect = "manifest identity mismatch: checked artifact Neumera.Contracts"
    return code != 0 and expect in out, out, secs + secs2


def copy_tree(src: Path, dst: Path, ignore: tuple[str, ...] = ()) -> None:
    shutil.copytree(src, dst, symlinks=True, ignore=shutil.ignore_patterns(*ignore))


def make_scratch() -> Path:
    scratch = Path(tempfile.mkdtemp(prefix="neumera-negative-"))
    copy_tree(REPO / "lean", scratch / "lean")
    shutil.copy2(REPO / "lean-toolchain", scratch / "lean-toolchain")
    (scratch / "rust").mkdir()
    for f in ("Cargo.toml", "Cargo.lock"):
        shutil.copy2(REPO / "rust" / f, scratch / "rust" / f)
    for crate in ("neumera-lean-sys", "neumera-lean"):
        copy_tree(
            REPO / "rust" / crate,
            scratch / "rust" / crate,
            ignore=("target",),
        )
    copy_tree(
        REPO / "rust" / "neumera-py",
        scratch / "rust" / "neumera-py",
        ignore=("target",),
    )
    return scratch


def cargo(
    root: Path, args: list[str], extra_env: dict[str, str]
) -> tuple[int, str, float]:
    """Run cargo against the scratch workspace."""
    # Never inherit a manifest override or toolchain override from the caller;
    # the scratch copy's own pin must decide. Case 15 reads cargo's "Compiling"
    # lines, which `CARGO_TERM_QUIET` would suppress.
    env = {
        k: v
        for k, v in os.environ.items()
        if k not in ("NEUMERA_LEAN_MANIFEST", "ELAN_TOOLCHAIN", "CARGO_TERM_QUIET")
    }
    env.update({"CARGO_TARGET_DIR": str(root / "target"), **extra_env})
    start = time.monotonic()
    proc = subprocess.run(
        ["cargo", *args, "--manifest-path", str(root / "rust" / "Cargo.toml")],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode, proc.stdout + proc.stderr, time.monotonic() - start


def build(root: Path, extra_env: dict[str, str]) -> tuple[int, str, float]:
    return cargo(root, ["build", "-p", "neumera-lean"], extra_env)


def set_pin(root: Path, toolchain: str) -> None:
    (root / "lean-toolchain").write_text(toolchain + "\n")


def case_toolchain_switch(root: Path, alt: str) -> tuple[bool, str, float]:
    """Case 15: the digest gate and the pin, end to end.

    (a) Changing the pin alone must fail: the alternate toolchain's lean.h has
        a different digest from the recorded one, and the bindings mirror the
        header, so the build must stop until the digest is deliberately updated.
    (b) Recording the alternate toolchain's digest must make the build pass,
        with `neumera-lean-sys` rebuilt and no manual `cargo clean`."""
    original_pin = (root / "lean-toolchain").read_text()
    original_digest = (root / DIGEST_FILE).read_text()
    probe = subprocess.run(
        ["lean", "--print-prefix"],
        env={**os.environ, "ELAN_TOOLCHAIN": alt},
        capture_output=True,
        text=True,
        check=False,
    )
    if probe.returncode != 0:
        return False, f"cannot resolve {alt}:\n" + probe.stderr, 0.0
    header = Path(probe.stdout.strip()) / "include" / "lean" / "lean.h"
    alt_digest = hashlib.sha256(header.read_bytes()).hexdigest()
    if alt_digest == original_digest.strip():
        return (
            True,
            f"skipped: {alt} shares lean.h with the pinned toolchain, so the digest gate has nothing to reject",
            0.0,
        )
    total = 0.0
    try:
        set_pin(root, alt)
        code, out, secs = build(root, {})
        total += secs
        expect = "has SHA-256"
        if code == 0 or expect not in out:
            return (
                False,
                "pin change was not stopped by the digest gate:\n" + out,
                total,
            )
        (root / DIGEST_FILE).write_text(alt_digest + "\n")
        code, out, secs = build(root, {})
        total += secs
        if code != 0:
            return (
                False,
                f"build under {alt} failed with its digest recorded:\n" + out,
                total,
            )
        if "Compiling neumera-lean-sys" not in out:
            return (
                False,
                "neumera-lean-sys was not rebuilt after the pin change:\n" + out,
                total,
            )
        return True, out, total
    finally:
        (root / "lean-toolchain").write_text(original_pin)
        (root / DIGEST_FILE).write_text(original_digest)


def main(argv: list[str]) -> int:
    keep = "--keep" in argv
    scratch = make_scratch()
    print(f"scratch copy: {scratch}")
    pristine = {p: (scratch / p).read_text() for p in (BASIC, CONTRACTS, DIGEST_FILE)}
    failures = 0

    code, out, secs = build(scratch, {})
    print(f"baseline  : {'ok' if code == 0 else 'FAILED'} ({secs:.1f}s)")
    if code != 0:
        print(out[-4000:])
        print("baseline build must succeed before negative tests mean anything")
        return 1

    for case in CASES:
        secs = 0.0
        if case.fresh_manifest:
            code, out, secs = build(scratch, {})
            if code != 0:
                print(
                    f"case {case.number:<2}   : FAILED re-baseline before {case.title}"
                )
                print(out[-4000:])
                failures += 1
                continue
        extra_env: dict[str, str] = {}
        case.mutate(scratch, extra_env)
        code, out, secs_case = build(scratch, extra_env)
        secs += secs_case
        failed = code != 0
        named = case.expect in out
        ok = failed and named
        verdict = "ok" if ok else "FAILED"
        detail = (
            ""
            if ok
            else (
                " (build succeeded)"
                if not failed
                else f" (output lacks {case.expect!r})"
            )
        )
        print(f"case {case.number:<2}   : {verdict} {case.title}{detail} ({secs:.1f}s)")
        if not ok:
            failures += 1
            print(out[-4000:])
        for p, text in pristine.items():
            (scratch / p).write_text(text)

    multi_step: list[tuple[int, str, Callable[[Path], tuple[bool, str, float]]]] = [
        (10, "edit supplied manifest in place", case_override_rerun),
        (
            13,
            "tamper an .olean after a valid build",
            case_artifact_tampered_after_build,
        ),
    ]
    alt = os.environ.get("NEUMERA_ALT_TOOLCHAIN")
    if alt:
        multi_step.append(
            (
                15,
                f"switch pin to {alt} without cleaning",
                lambda root: case_toolchain_switch(root, alt),
            )
        )
    else:
        print(
            "case 15   : skipped (set NEUMERA_ALT_TOOLCHAIN to a second installed toolchain)"
        )
    for number, title, fn in multi_step:
        ok, out, secs = fn(scratch)
        verdict = "ok" if ok else "FAILED"
        print(f"case {number:<2}   : {verdict} {title} ({secs:.1f}s)")
        if not ok:
            failures += 1
            print(out[-4000:])

    if keep:
        print(f"kept {scratch}")
    else:
        shutil.rmtree(scratch, ignore_errors=True)
    if failures:
        print(f"{failures} negative test(s) did not fail the build as expected")
        return 1
    print("all negative tests failed the build as expected")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
