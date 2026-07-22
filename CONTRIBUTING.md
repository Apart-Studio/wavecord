# Contributing to WaveCord

Thanks for your interest in improving WaveCord. This guide covers how to set up
the project, run the checks, and submit changes that get merged quickly. Please
read it fully before opening a pull request. By taking part, you agree to follow
our [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- Report bugs and request features by opening an issue.
- Improve the documentation or the examples.
- Submit code via a pull request.

If it is your first pull request here, welcome. Small, focused contributions are
the easiest to review and the most likely to be merged.

## Development setup

WaveCord is a Rust core with Python bindings. You need a Rust toolchain, Python
3.9 or newer, and [maturin](https://www.maturin.rs/).

```bash
git clone https://github.com/Apart-Studio/wavecord
cd wavecord

python -m venv .venv && source .venv/bin/activate
pip install ".[dev]"
maturin develop
```

## Running the checks

Run the same checks CI runs before you push. If they pass locally, your pull
request will almost certainly pass CI.

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

- Keep each pull request focused on a single change.
- Fill in the pull request template completely. Pull requests that remove or
  rename its sections are closed automatically and labelled `invalid`.
- Add or update tests for any behaviour you change.
- Make sure `cargo test` and `pytest` both pass before opening the request.
- Match the surrounding code style. Rust follows `rustfmt` and `clippy`; Python
  targets clean, typed, PEP 8 code checked with `ruff`.
- Update `CHANGELOG.md` when your change is user-visible.
- Write clear commit messages that explain the why, not just the what.

## AI policy

AI coding assistants (such as GitHub Copilot, ChatGPT, and other AI Tools) are welcome
tools, and AI-assisted contributions are allowed. We only ask for honesty and
ownership. The rules are simple:

1. **Disclose it.** Use the AI disclosure section in the pull request template to
   state whether AI tools were used. Being transparent will never get your pull
   request rejected on its own. Hiding it can get it closed.
2. **Understand every line.** You must fully understand, be able to explain, and
   be able to maintain every line you submit, including anything an AI produced.
   If you cannot explain how your code works, it is not ready.
3. **You are responsible.** AI-assisted code is held to exactly the same standard
   as hand-written code. You, the author, are accountable for its correctness,
   security, and licensing, not the tool.
4. **Respect licensing.** Do not submit AI output that reproduces code from
   sources that are incompatible with the MIT license. Only submit work you have
   the right to license to this project (see below).
5. **No low-effort automation.** Unreviewed, machine-generated "spam" pull
   requests, mass find-and-replace changes, or output pasted without thought will
   be closed without further review.

## Contributor license and ownership

Please read this section carefully. It is important and it is binding.

**Inbound equals outbound.** WaveCord is released under the
[MIT License](LICENSE). By submitting a contribution (code, documentation,
examples, or any other material) you agree that your contribution is licensed to
the project and to everyone else under that same MIT License.

**License grant.** You grant the WaveCord project and its maintainers a
perpetual, worldwide, non-exclusive, royalty-free, and irrevocable license to
use, reproduce, modify, adapt, publish, sublicense, and distribute your
contribution, on its own and as part of WaveCord, without restriction.

**Once it is merged, it belongs to the project.** After a contribution is
merged it becomes an integral part of the WaveCord codebase and is maintained,
changed, relicensed within the terms of the MIT License, and redistributed by
the project as a whole. You keep the copyright to your original work, but you do
not retain any separate control over the merged result and cannot demand its
removal.

**You confirm that:**

- The contribution is your own original work, or you have the full right and
  authority to submit it and to license it under the terms above.
- Your contribution does not knowingly infringe the copyright, patent,
  trademark, trade secret, or any other right of a third party.
- No employer, client, or other party can claim rights over the contribution
  that would conflict with this license grant.
- Any AI-assisted portions are, to the best of your knowledge, free of code that
  you are not entitled to license under the MIT License.

If you cannot agree to all of the above, please do not submit a pull request.

## Reporting security issues

Do not open a public issue for security problems. Follow the process in our
[Security Policy](SECURITY.md) instead.

## Project layout

- `crates/wavecord-core`: the pure-Rust engine (protocol, models, node manager)
- `crates/wavecord-py`: the PyO3 bindings
- `python/wavecord`: the public Python API and Discord-library adapters
- `tests`: Python tests
- `examples`: minimal bots for each supported Discord library

## License

By contributing, you agree that your contributions are licensed under the MIT
License, the same license that covers the project.
