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

 14. (opt-in) Switch the Lean pin to a      -> build fails naming stale
     second installed toolchain without       lean-rs-sys bindings; passes
     cleaning the target directory.           after `cargo clean -p lean-rs-sys`

Case 14 runs only when NEUMERA_ALT_TOOLCHAIN names an installed toolchain that
lean-rs-sys supports (for example `leanprover/lean4:v4.34.0`).

Usage: python scripts/contract_negative_tests.py [--keep]
Standard library only. Needs cargo, lake, and lean on PATH.
"""

from __future__ import annotations

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
    copy_tree(REPO / ".cargo", scratch / ".cargo")
    (scratch / "rust").mkdir()
    for f in ("Cargo.toml", "Cargo.lock"):
        shutil.copy2(REPO / "rust" / f, scratch / "rust" / f)
    copy_tree(
        REPO / "rust" / "neumera-lean",
        scratch / "rust" / "neumera-lean",
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
    """Run cargo against the scratch workspace, from inside the scratch copy so
    its own `.cargo/config.toml` (and so its ELAN_TOOLCHAIN) applies."""
    # Never inherit a manifest override or toolchain override from the caller;
    # cases set their own.
    env = {
        k: v
        for k, v in os.environ.items()
        if k not in ("NEUMERA_LEAN_MANIFEST", "ELAN_TOOLCHAIN")
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
    """Point both copies of the pin at `toolchain`."""
    (root / "lean-toolchain").write_text(toolchain + "\n")
    config = root / ".cargo" / "config.toml"
    lines = config.read_text().splitlines(keepends=True)
    out = [
        f'ELAN_TOOLCHAIN = "{toolchain}"\n' if ln.startswith("ELAN_TOOLCHAIN") else ln
        for ln in lines
    ]
    config.write_text("".join(out))


def case_toolchain_switch(root: Path, alt: str) -> tuple[bool, str, float]:
    """Case 14: switching the pin without cleaning must be caught, because the
    lean-rs-sys build script does not rerun on a pin change and would leave
    bindings for the previous toolchain in the target directory."""
    original = (root / "lean-toolchain").read_text().strip()
    total = 0.0
    try:
        set_pin(root, alt)
        code, out, secs = build(root, {})
        total += secs
        expect = "lean-rs-sys was compiled against a different Lean version"
        if code == 0 or expect not in out:
            return False, "stale bindings were not detected:\n" + out, total
        code, out, secs = cargo(root, ["clean", "-p", "lean-rs-sys"], {})
        total += secs
        if code != 0:
            return False, "cargo clean -p lean-rs-sys failed:\n" + out, total
        code, out, secs = build(root, {})
        total += secs
        if code != 0:
            return (
                False,
                f"build under {alt} failed after cleaning the bindings:\n" + out,
                total,
            )
        return True, out, total
    finally:
        set_pin(root, original)
        cargo(root, ["clean", "-p", "lean-rs-sys"], {})


def main(argv: list[str]) -> int:
    keep = "--keep" in argv
    scratch = make_scratch()
    print(f"scratch copy: {scratch}")
    pristine = {p: (scratch / p).read_text() for p in (BASIC, CONTRACTS)}
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
                14,
                f"switch pin to {alt} without cleaning",
                lambda root: case_toolchain_switch(root, alt),
            )
        )
    else:
        print(
            "case 14   : skipped (set NEUMERA_ALT_TOOLCHAIN to a second installed toolchain)"
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
