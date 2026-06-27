# SIMI -- Similarity & Text-Analysis Engine

SIMI is a production-grade similarity and text-analysis toolkit with a pure-Rust core and
bindings for Python and Node.js. It provides eight algorithms across three categories, a
composable preprocessing layer, the **SimiFlow** intent-aware routing pipeline, and batched
parallel evaluation backed by rayon — the building blocks for reliable similarity checks across
many use cases.

## Use Cases

SIMI is designed to be integrated into real systems:

- **Bot & abuse protection** — fingerprint and cluster near-identical submissions or payloads.
- **Spam & content moderation** — detect reworded duplicates and template spam at scale.
- **Record matching & entity resolution** — reconcile names, addresses, SKUs, and accounts.
- **Deduplication** — collapse near-duplicate documents, listings, or tickets.
- **Search & ranking** — score and order candidates by relevance.
- **Fuzzy input handling** — tolerate typos and formatting noise in user input.

## Why SIMI

Picking the right similarity algorithm and wiring it up correctly for each task is the hard
part. SIMI does that work for you:

- **Intent-aware routing.** Tell SimiFlow *what* you're comparing — `names`, `typos`, `codes`, `documents`, `dedup` — and it selects the right algorithm. Or use `auto` and it decides from the input.
- **A confidence cascade.** Resolve the obvious matches and mismatches with a cheap fast pass, escalate only the ambiguous middle to a heavier local algorithm, and reach a custom hook (an LLM, a DB lookup, a review queue) only when nothing local can decide.
- **Native throughput.** A single clean API over eight algorithms, plus rayon-backed batch evaluation that scales across every CPU core.

> **A note on origin.** SIMI grew out of a need to cut the cost, latency, and unpredictability
> of using an LLM for every "are these the same?" decision. Most of those checks are
> deterministic and belong in fast, testable local code — which is exactly what SIMI provides.

## Quick Example

```rust
use simi::algo::{levenshtein, jaro_winkler};

let d = levenshtein::similarity("kitten", "sitting");
// 0.571

let j = jaro_winkler::similarity("MARTHA", "MARHTA");
// 0.961
```

## Installation

Rust (crates.io):
```bash
cargo add simi-flow
```

Python (PyPI):
```bash
pip install simi-flow
```

Node.js (npm):
```bash
npm install @siktec-lab/simi-flow
```

## Core Modules

### Algorithms (`simi::algo`)

Eight algorithms in three categories:

| Category | Algorithms | What it solves |
|---|---|---|
| Short Strings and Typos | Levenshtein, Jaro-Winkler, Hamming | Edit distance, name matching, position-based codes |
| Sets and Documents | Jaccard, MinHash, SimHash | N-gram overlap, large-scale deduplication |
| Statistical Meaning | BM25, TF-IDF + Cosine | Search relevance, term-weighted similarity |

Detailed documentation: [ALGOS.md](docs/ALGOS.md)

### Preprocessing (`simi::preprocess`)

Normalize inputs before comparison to reduce noise:

```rust
use simi::preprocess::Preprocessor;

let cleaned = Preprocessor::new()
    .with_lowercase(true)
    .with_remove_stopwords(true)
    .process("The Quick Brown Fox");
// "quick brown fox"
```

Operations: Unicode NFC normalization, whitespace collapse, trimming,
lowercase conversion, stopword removal (180+ built-in, or bring your own).

### Router (`simi::router`)

The SimiFlow pipeline automates algorithm selection by cascading
through confidence tiers, or by selecting the best algorithm via intent:

```rust
use simi::router::{SimiFlow, Intent};

// Intent-based: pick by data type
SimiFlow::for_intent(Intent::Names).compare("MARTHA", "MARHTA")?;

// Auto-detect per pair
SimiFlow::auto().compare(a, b)?;

// Manual tier configuration
use simi::router::{SimiFlow, Strategy, Threshold, Algo};

let result = SimiFlow::new()
    .preprocess(true)
    .strategy(Strategy::Cascade)
    .tier_1(Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),
        Threshold::LessThan(0.10))
    .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
    .fallback(|a, b| (0.8, Some("llm_verified".into())))
    .compare("hello world", "hello there")
    .unwrap();
```

Full documentation: [ROUTER.md](docs/ROUTER.md)

### Batch Processing (`simi::batch`)

Rayon-powered parallel evaluation:

```rust
use simi::batch::BatchComparator;
use simi::router::Algo;

let cmp = BatchComparator::new(Algo::Levenshtein);

// Element-wise
let scores = cmp.compare_pairs(&vec_a, &vec_b)?;

// One reference against many candidates
let scores = cmp.compare_one_to_many(&reference, &candidates)?;

// Full cross-product matrix
let scores = cmp.compare_matrix(&vec_a, &vec_b)?;
```

Tens of thousands of comparisons process in parallel across all CPU cores
with zero configuration.

---

## Use Cases in Practice

### Record Matching with a Confidence Cascade

Decide whether two company names refer to the same entity — resolving confident cases
locally and escalating only the ambiguous ones:

```rust
use simi::router::{SimiFlow, Strategy, Threshold, Algo};

let result = SimiFlow::new()
    .preprocess(true)
    .tier_1(Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),
        Threshold::LessThan(0.10))
    .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
    .fallback(|a, b| {
        // Only reached for genuinely ambiguous pairs — your LLM, DB, or review hook
        call_expensive_llm_api(a, b)
    })
    .compare(record_a, record_b)?;
```

### Entity Resolution

Deduplicate customer records by name, address, or product SKU:

```rust
use simi::batch::BatchComparator;
use simi::router::Algo;

let comparator = BatchComparator::new(Algo::JaroWinkler);
let matches = comparator.compare_matrix(&names, &names)?;

// Filter pairs above a threshold
let duplicates: Vec<_> = matches
    .iter()
    .filter(|r| r.score > 0.92 && r.index_a != r.index_b)
    .collect();
```

### Search Relevance

Rank documents by how well they match a query string:

```rust
use simi::algo::bm25;

let query = "machine learning rust";
let documents = vec![
    // thousands of docs...
];

let scores: Vec<f64> = documents
    .iter()
    .map(|doc| bm25::similarity(query, doc))
    .collect();
```

### Content Deduplication

Identify near-duplicate articles in a content pipeline:

```rust
use simi::algo::simhash;

// Compare two documents directly (shingle size 4)
let similarity = simhash::compare("article body text...", "article body text...", 4);
if similarity > 0.90 {
    // Near-duplicate detected
}

// Or fingerprint once and reuse across many comparisons
let fp_a = simhash::fingerprint("article body text...", 4);
let fp_b = simhash::fingerprint("another article...", 4);
```

### CLI Tool Integration

SIMI works well as the engine behind a CLI tool:

```rust
use simi::preprocess::clean;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a = clean(&args[1]);
    let b = clean(&args[2]);

    let score = simi::algo::levenshtein::similarity(&a, &b);
    println!("{:.3}", score);
}
```

---

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

## Performance

- Release build: `opt-level = 3`, LTO enabled, single codegen unit.
- Levenshtein: O(min(n,m)) space, single-row DP.
- Batch: rayon parallel iterator, scales linearly with cores.
- Criterion benchmarks for every algorithm (run `cargo bench`).

## License

MIT -- see [LICENSE](LICENSE).
