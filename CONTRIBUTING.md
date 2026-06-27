# Contributing to SIMI

Thanks for your interest in contributing. Here is how to get started.

## Setup

```bash
git clone https://github.com/siktec-lab/simi.git
cd simi
cargo build
cargo test
make help
```

### Prerequisites

- **Rust** (stable, via [rustup](https://rustup.rs)). On Windows use the
  **MSVC** toolchain (`stable-x86_64-pc-windows-msvc`); it is the only Windows
  target napi-rs supports, and the bindings will not build under the GNU
  toolchain.
- **Windows only**: the MSVC toolchain needs the **Visual Studio Build Tools**
  with the *Desktop development with C++* workload **and** the **Windows SDK**
  (the SDK is a separate component; without it `link.exe` cannot link).
- **Python bindings**: `pip install maturin pytest`.
- **Node.js bindings**: Node 18+; the napi-rs CLI is pulled in via
  `cd js && npm install`.

## Running Tests

```bash
make test          # all tests (Rust + Python + Node)
make test-rust     # Rust only
make test-python   # Python (requires `pip install maturin`)
make test-node     # Node.js (requires `npm install @napi-rs/cli`)
```

Verify every build configuration compiles:

```bash
make features
# equivalently:
cargo check                  # default features
cargo check --all-features   # incl. python + nodejs bindings
```

## Benchmarks

```bash
cargo bench   # Criterion harness (benches/algorithms.rs)
```

Run `cargo bench` before and after performance-sensitive changes to catch
regressions; Criterion compares against the previous run automatically.

## Code Style

- Rust: standard `rustfmt`; run `cargo fmt` before committing. CI enforces
  this with `cargo fmt --check`, which fails on any unformatted code.
- No warnings: CI runs `cargo clippy -- -D warnings`, so any warning fails the
  build. Keep `cargo build` / `cargo clippy` output clean.

## Pull Request Process

1. Fork the repo and create your branch from `main`
2. Add tests for any new functionality
3. Run **`make pre-push`**; it runs every gate CI enforces (`fmt --check`,
   `clippy -D warnings`, feature checks, and Rust tests)
4. Run `cargo bench` to confirm no performance regression
5. Update docs if you add or change public APIs
6. Open a PR with a clear description

## Project Structure

```
src/
├── lib.rs          Library entry point
├── prelude.rs      Common imports
├── error.rs        SimiError type
├── algo/           Algorithm implementations
│   ├── mod.rs
│   ├── levenshtein.rs
│   ├── jaro_winkler.rs
│   ├── hamming.rs
│   ├── jaccard.rs
│   ├── minhash.rs
│   ├── simhash.rs
│   ├── bm25.rs
│   └── tfidf.rs
├── preprocess.rs   String preprocessing
├── router.rs       SimiFlow pipeline
├── batch.rs        Rayon-based batch processing
├── python.rs       Python bindings (PyO3)
└── nodejs.rs       Node.js bindings (napi-rs)
```

## Releasing and Publishing

```bash
./scripts/bump-version.sh 0.2.0     # bumps Cargo.toml, pyproject.toml, js/package.json
git add -A && git commit -m "Release v0.2.0"
git tag v0.2.0 && git push --follow-tags
```

The tag triggers the Release workflow which publishes to:
- **crates.io** -- OIDC auth via `rust-lang/crates-io-auth-action`
- **PyPI** -- `maturin` builds per-platform wheels, published via OIDC
- **npm** -- `@siktec-lab/simi` plus per-platform packages via OIDC
