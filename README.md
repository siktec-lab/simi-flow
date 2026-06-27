# SIMI — a Similarity & Text-Analysis Engine

SIMI is a production-grade similarity and text-analysis toolkit for **Rust, Python, and
Node.js**. It packages **8 battle-tested algorithms** behind one clean API and adds
**SimiFlow** — an intent-aware routing pipeline — so you can build and integrate reliable
similarity checks across a wide range of real-world workloads:

- **Bot & abuse protection** — fingerprint and cluster near-identical submissions, payloads, or behaviour.
- **Spam & content moderation** — detect reworded duplicates and template spam at scale.
- **Record matching & entity resolution** — reconcile names, addresses, SKUs, and accounts across systems.
- **Deduplication** — collapse near-duplicate documents, listings, or tickets.
- **Search & ranking** — score and order candidates by relevance.
- **Fuzzy input handling** — tolerate typos and formatting noise in user input.

One core, three languages, identical results — pick the right algorithm per job, or let
SimiFlow route by intent.

```rust
use simi::router::{SimiFlow, Intent};

// Declare what you're comparing; SIMI selects and runs the right algorithm natively.
let result = SimiFlow::new()
    .compare_with_intent(Intent::Names, "MARTHA", "MARHTA")
    .unwrap();
// result.score == 0.961, result.algorithm == "jaro_winkler"
```

## Why SIMI

- **🧭 Intent-based routing** — tell SIMI *what* you're comparing (`names`, `typos`, `codes`, `documents`, `dedup`) and it selects the right algorithm. Or use `auto` and let it decide from the input.
- **🧰 8 algorithms, one API** — edit distance, name matching, set overlap, document fingerprinting, and probabilistic retrieval — all returning a normalized `[0.0, 1.0]` score.
- **⚡ Native speed** — pure-Rust core with a tuned release profile. Single comparisons land in **nanoseconds to microseconds** (see [Performance](#performance)).
- **🚀 Batch + parallel** — evaluate thousands of pairs across every CPU core with rayon.
- **🎯 Composable & tunable** — preprocessing, confidence thresholds, and a tiered cascade with an optional escalation hook for the genuinely ambiguous cases.
- **🌍 Three languages, one core** — identical algorithms in Rust, Python (PyO3), and Node.js (napi-rs).

> **A note on origin.** SIMI grew out of a need to cut the cost, latency, and unpredictability
> of throwing an LLM at every "are these the same?" decision. Most of those checks are
> deterministic and belong in fast, testable local code. SimiFlow's cascade reflects that:
> resolve confidently locally, and escalate (to an LLM or any custom hook) only when you must.

## Quick Start

```rust
use simi::algo::{levenshtein, jaro_winkler};

// Levenshtein similarity (typos and spelling)
let d = levenshtein::similarity("kitten", "sitting");
println!("{:.3}", d); // ~0.571

// Jaro-Winkler similarity (names)
let j = jaro_winkler::similarity("MARTHA", "MARHTA");
println!("{:.3}", j); // ~0.961
```

## Features

### 8 Algorithms, Categorized by Data Type

| Category | Algorithms | Best For |
|---|---|---|
| Short Strings and Typos | Levenshtein, Jaro-Winkler, Hamming | Names, typos, equal-length codes |
| Sets and Documents | Jaccard, MinHash, SimHash | N-gram sets, large document fingerprints |
| Statistical Meaning | BM25, TF-IDF + Cosine | Search ranking, term-weighted vectors |

### SimiFlow — intent-aware routing

The headline feature. Instead of hand-picking an algorithm, tell SimiFlow your **intent** and
it routes to the right one:

```rust
use simi::router::{SimiFlow, Intent};

let sf = SimiFlow::new();

sf.compare_with_intent(Intent::Names,     "MARTHA", "MARHTA")?;        // → Jaro-Winkler
sf.compare_with_intent(Intent::Typos,     "recieve", "receive")?;     // → Levenshtein
sf.compare_with_intent(Intent::Codes,     "ABC123", "ABC124")?;       // → Hamming
sf.compare_with_intent(Intent::Documents, long_a, long_b)?;           // → BM25
sf.compare_with_intent(Intent::Auto,      a, b)?;                      // → SIMI decides from the input
```

`Auto` inspects the inputs and chooses: short equal-length → Hamming, short → Jaro-Winkler,
medium → BM25, long → SimHash. One API call covers names, typos, codes, and documents.

### SimiFlow — the confidence cascade

For the "is this a match?" decision, build a tiered cascade. SIMI resolves the confident cases
with a cheap fast pass, escalates the ambiguous middle to a heavier local algorithm, and only
reaches your custom hook (an LLM, a DB lookup, a human review queue) when nothing local can decide:

```rust
use simi::router::{SimiFlow, Strategy, Threshold, Algo};

let result = SimiFlow::new()
    .preprocess(true)
    .strategy(Strategy::Cascade)
    // Tier 1: cheap, fast. >0.95 → confident match, <0.10 → confident mismatch. Resolve & stop.
    .tier_1(Algo::JaroWinkler, Threshold::GreaterThan(0.95), Threshold::LessThan(0.10))
    // Tier 2: heavier local pass for the in-between scores.
    .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
    // Tier 3: only reached when local algorithms can't decide — call your model here.
    .fallback(|a, b| {
        // Your LLM API call — runs for a fraction of inputs, not all of them.
        (0.8, Some("llm_verified".into()))
    })
    .compare("hello world", "hello there")
    .unwrap();
```

The `ComparisonResult` tells you exactly which tier answered (`tier`, `algorithm`,
`fallback_called`) — so you can measure how often you actually hit the model.

### Batch Parallelism

```rust
use simi::batch::BatchComparator;
use simi::router::Algo;

let comparator = BatchComparator::new(Algo::Levenshtein);
let results = comparator.compare_pairs(&thousands_of_strings_a, &thousands_of_strings_b)?;
```

Powered by rayon -- evaluates thousands of pairs across all CPU cores. Three modes:
`compare_pairs` (element-wise), `compare_one_to_many` (one reference vs. a list), and
`compare_matrix` (full cross-product).

### String Preprocessing

```rust
use simi::preprocess::Preprocessor;

let cleaned = Preprocessor::new()
    .with_lowercase(true)
    .with_remove_stopwords(true)
    .process("  The   Quick Brown Fox  ");
// => "quick brown fox"
```

## Installation

### Rust (crates.io)

```bash
cargo add simi-flow
```

### Python (PyPI)

```bash
pip install simi-flow
```

### Node.js (npm)

```bash
npm install @siktec-lab/simi-flow
```

## Performance

SIMI's core is pure Rust with a release profile tuned for speed (`lto`, `codegen-units = 1`).
Single comparisons are effectively free next to a network call to a model:

| Algorithm | Input | Time |
|---|---|---|
| Levenshtein | "kitten"/"sitting" | ~80 ns |
| Jaro-Winkler | "MARTHA"/"MARHTA" | ~200 ns |
| Hamming | 7-char equal | ~150 ns |
| Jaccard bigram | Short texts | ~1.7 µs |
| MinHash (128) | Short doc | ~17 µs |
| SimHash | Short doc | ~5 µs |
| BM25 | Short docs | ~2.9 µs |
| TF-IDF | Short texts | ~2.7 µs |

At these speeds you can run millions of comparisons inline, in a request path, or across a
batch job without it showing up on a flame graph.

Reproduce these on your own hardware with `cargo bench` (Criterion benches in `benches/`).

## Architecture

```
simi
├── algo/       -- 8 similarity algorithms
├── preprocess  -- Unicode normalization, whitespace, stopwords
├── router      -- SimiFlow pipeline builder
├── batch       -- rayon-based parallel evaluation
├── python      -- PyO3 bindings
└── nodejs      -- napi-rs bindings
```

## License

MIT -- see [LICENSE](LICENSE).
