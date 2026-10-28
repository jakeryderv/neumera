# Project direction

A short record of where this repository started, why its direction changed, and what the change does and does not affect. Kept so the lineage stays visible after the README is rewritten.

## Original scope (October 2026)

The repository was created as a Python library with a Rust extension, described as:

> Neumera is a Python library for exploring machine learning, numerical computing, and optimization through foundational implementations and practical utilities. It combines Python and Rust to support experimentation, learning, and high-performance computation.

The pyproject description read: "Foundational implementations and utilities for machine learning, numerical computing, and optimization."

Nothing was implemented under that scope. The repository held tooling only: a PyO3 extension stub, maturin packaging, uv, just, ruff, ty, pytest, CI, release workflow, and commit conventions.

Exploratory notes written under the original scope live in `docs/notes/`. They work toward representing mathematical meaning explicitly in Python (domains, properties, relations, operations) while delegating computation to existing libraries. They are the direct ancestors of the current direction and remain useful reference.

## Current direction (from 2026-10-28)

The repository now hosts the idea described in [Bringing mathematical meaning into everyday programming](bringing-mathematical-meaning-into-everyday-programming.md), worked through concretely in [Design example: sorted vectors](design-example-sorted-vectors.md): Lean-checked definitions and theorems, a Rust integration layer that preserves established properties on concrete data, and a Python adapter over the Rust layer.

The change was made because the Python-plus-Rust build setup was already in place and nothing implemented depended on the original scope. The original scope is not abandoned; machine learning, numerical computing, and optimization are candidate downstream applications of the toolkit, to be revisited once the toolkit exists.

## Naming

- **`neumera`** stays as the Python package and PyPI distribution name for now. It is already published, the badges and release workflow refer to it, and renaming a Python package is cheap to do later and expensive to do twice.
- **Rust crates** are named independently of the Python package. The current Cargo package is the PyO3 extension that becomes the Python adapter. The Lean integration crate, when it exists, gets its own name and may be published separately.
- A more accurate name for the Python package can be reconsidered once the toolkit has a shape worth naming.

## What changed in the repository

- README opening and pyproject description rewritten to match the current direction.
- This file added.

## What did not change

- Tooling, CI, release workflow, commit conventions, license.
- The exploratory notes in `docs/notes/`.
- Package and crate names.
