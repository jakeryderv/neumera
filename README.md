# neumera

[![CI](https://github.com/jakeryderv/neumera/actions/workflows/CI.yml/badge.svg)](https://github.com/jakeryderv/neumera/actions/workflows/CI.yml)
[![PyPI](https://img.shields.io/pypi/v/neumera)](https://pypi.org/project/neumera/)
[![Python versions](https://img.shields.io/pypi/pyversions/neumera)](https://pypi.org/project/neumera/)

Neumera brings mathematical meaning into everyday programming. Definitions and theorems are written and checked in Lean, a Rust integration layer preserves the established properties on concrete data, and a Python adapter exposes the same objects and operations. The aim is that guarantees proved once can be carried through ordinary programs and used to discharge requirements, with the trusted code small enough to enumerate.

Start with [the vision and prototype scope](docs/bringing-mathematical-meaning-into-everyday-programming.md), then [the worked design example](docs/design-example-sorted-vectors.md). [Project direction](docs/project-direction.md) records where the repository started and why it changed.

> **Status:** early development. There is no usable API yet, and everything is subject to change.

## Installation

```sh
pip install neumera
```

Prebuilt wheels are available for Linux, macOS, and Windows on Python 3.12 and newer.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, everyday commands, and the release process.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
