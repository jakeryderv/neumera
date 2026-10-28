<!-- Title: conventional commit format (feat:, fix:, build:, ci:, docs:, ...).
     It becomes the squash commit subject. -->

## What

<!-- Scope implemented. Say what is deliberately out of scope. -->

## Validation

<!-- Local checks run, CI link, anything verified only locally or opt-in. -->

## Limitations

<!-- What this leaves open and where that is tracked. -->

## Checklist

- [ ] `just check` passes locally
- [ ] Touches `build.rs`, `unsafe`, `lean/Manifest.lean`, or the contract manifest format: `just negative-tests` passes, and an independent adversarial review of those files has been done and its findings addressed or recorded here
- [ ] Docs updated where behavior changed (`CONTRIBUTING.md`, `docs/`)
