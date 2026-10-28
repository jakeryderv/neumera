# Project instructions for coding agents

Read `CONTRIBUTING.md` first; it is the source of truth for layout, commands, and the PR workflow. These notes are what an agent most often gets wrong.

## Environment

- `lean`, `lake`, and `elan` live in `~/.elan/bin`, which non-interactive shells may not have on `PATH`. Prepend it before running Lake or building `rust/neumera-lean`.
- Run cargo from inside the repository so `.cargo/config.toml` applies; it pins `ELAN_TOOLCHAIN` for dependency build scripts. The build fails with an explanation otherwise.

## Workflow

- Work on a branch and open a draft PR as soon as it is pushed. CI runs on pull requests, not on branch pushes.
- Merge with `gh pr merge --squash --delete-branch`. Never add `Co-Authored-By` or tool attribution to commits or PRs.
- Changes to `rust/neumera-lean/build.rs`, any `unsafe` code, `lean/Manifest.lean`, or the manifest format get an independent adversarial review before merge (a second agent or a reviewer who has not seen the author's reasoning). Record its findings in the PR. Both such reviews during the first slice found real defects the author's own checks missed.

## Scope discipline

- The Python adapter (`rust/neumera-py`) does not depend on `rust/neumera-lean` yet. Do not add that dependency, bundle the Lean runtime, or touch the wheel jobs unless the task is explicitly about Python integration or distribution.
