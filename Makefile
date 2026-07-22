PY := $(shell [ -x .venv/bin/python ] && echo .venv/bin/python || echo python3)
RUFF := $(shell [ -x .venv/bin/ruff ] && echo .venv/bin/ruff || echo ruff)

.PHONY: help develop build test lint check clean

help:
	@echo "make develop build the Rust extension into your environment"
	@echo "make check run everything CI runs (rustfmt, clippy, ruff, tests)"
	@echo "make test cargo tests + pytest"
	@echo "make lint rustfmt, clippy, ruff"
	@echo "make build build a release wheel"
	@echo "make clean remove build artifacts"

develop:
	$(PY) -m maturin develop

build:
	$(PY) -m maturin build --release

lint:
	cargo fmt --all --check
	cargo clippy --all-targets --all-features -- -D warnings
	$(RUFF) check .

test:
	cargo test -p wavecord-core
	$(PY) -m pytest

check: lint
	cargo test -p wavecord-core
	$(PY) -m maturin develop
	$(PY) -m pytest
	@echo "All checks passed."

clean:
	cargo clean
	rm -rf target/wheels
