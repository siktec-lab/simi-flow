## SIMI Build System
##
## Usage:
##   make help           Show this help
##   make pre-push       Run every CI gate locally (fmt, clippy, features, test)
##   make fmt            Format the code (rustfmt)
##   make fmt-check      Check formatting without modifying (CI gate)
##   make clippy         Lint with clippy, warnings-as-errors (CI gate)
##   make features       Check features build (CI gate)
##   make check          Cargo check (all features)
##   make build          Release build
##   make test           Run all tests (Rust + Python + Node)
##   make test-rust      Cargo test
##   make test-python    Build Python wheel + pytest
##   make test-node      Build Node addon + node tests
##   make bench          Run benchmarks
##   make clean          Cargo clean

.DEFAULT_GOAL := help

# Rust targets

check:  ## Cargo check (all features)
	cargo check --all-features

fmt:  ## Format code (rustfmt)
	cargo fmt

fmt-check:  ## Check formatting (CI gate: cargo fmt --check)
	cargo fmt --check

clippy:  ## Lint, warnings-as-errors (CI gate: cargo clippy -- -D warnings)
	cargo clippy -- -D warnings

features:  ## Check default / all-features builds (CI gate)
	@echo "==> default features..." && cargo check
	@echo "==> all-features..."     && cargo check --all-features

build:  ## Release build
	cargo build --release

test-rust:  ## Run Rust tests
	cargo test

bench:  ## Run criterion benchmarks
	cargo bench

bump-version:  ## Bump version (usage: make bump-version V=0.2.0)
	@if [ -z "$(V)" ]; then echo "Usage: make bump-version V=0.2.0"; exit 1; fi
	@bash scripts/bump-version.sh $(V)

clean:  ## Cargo clean
	cargo clean

# Python targets

python-build:  ## Build Python wheel
	@echo "==> Building Python wheel..."
	@pip install maturin 2>/dev/null || true
	@cargo check --features python || { echo "FAIL: cargo check"; exit 1; }
	@maturin build --release 2>&1 | grep -q "Built wheel" && echo "   ok" || { echo "FAIL: maturin build"; echo "   Last output:"; maturin build --release 2>&1; exit 1; }
	@pip install target/wheels/*.whl 2>/dev/null || true

test-python: python-build  ## Build + run Python tests
	@echo "==> Running Python tests..."
	@pytest python/tests/ -v 2>/dev/null || { echo "FAIL: pytest"; echo "   Install: pip install pytest"; }

# Node.js targets

node-build:  ## Build Node.js addon
	@echo "==> Building Node.js addon..."
	@cargo check --features nodejs || { echo "FAIL: cargo check"; exit 1; }
	@cd js && npm install --no-audit --no-fund >/dev/null 2>&1 || true
	@js/node_modules/.bin/napi build --platform --release -c js/package.json --features nodejs --cargo-flags="--lib" js 2>/dev/null && echo "   ok" || { echo "FAIL: napi build"; echo "   Install: cd js && npm install"; exit 1; }

test-node: node-build  ## Build + run Node tests
	@echo "==> Running Node.js tests..."
	@node js/test.js

# Combined targets

test: test-rust test-python test-node  ## Run all tests

## Run every gate CI enforces, in CI order, before pushing.
pre-push: fmt-check clippy features test-rust  ## Run all CI gates locally
	@echo ""
	@echo "==> All pre-push checks passed. Safe to push."

all: build python-build node-build  ## Build everything

# Help

help:  ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'
