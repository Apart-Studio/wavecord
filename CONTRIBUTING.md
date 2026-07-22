# Contributing to WaveCord

Thanks for your interest in improving WaveCord. This guide covers how to set up
the project, run the checks, and submit changes. By participating, you agree to
follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- Report bugs and request features by opening an issue.
- Improve documentation or examples.
- Submit code via a pull request.

## Development setup

WaveCord is a Rust core with Python bindings. You need a Rust toolchain, Python
3.9 or newer, and [maturin](https://www.maturin.rs/).

```bash
git clone https://github.com/Apart-studio/wavecord
cd WaveCord

python -m venv .venv && source .venv/bin/activate
pip install maturin pytest pytest-asyncio
maturin develop
```

## Running the checks

These are the same checks CI runs:

```bash
# Rust
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p wavecord-core

# Python
ruff check .
maturin develop
pytest
```

For changes that touch the protocol or the network layer, please also test
against a running [Lavalink](https://lavalink.dev) node (v3 or v4) using one of
the bots in [examples/](examples/).

## Pull request guidelines

- Keep pull requests focused on a single change.
- Add or update tests for any behavior you change.
- Make sure `cargo test` and `pytest` both pass before opening the request.
- Match the surrounding code style. Rust follows `rustfmt`; Python targets clean,
  typed, PEP 8 code.
- Update `CHANGELOG.md` under the unreleased section when your change is
  user-visible.

## Project layout

- `crates/wavecord-core`: the pure-Rust engine (protocol, models, node manager)
- `crates/wavecord-py`: the PyO3 bindings
- `python/wavecord`: the public Python API and Discord-library adapters
- `tests`: Python tests

## License

By contributing, you agree that your contributions are licensed under the MIT
License, the same license that covers the project.
