# SIMI -- Similarity Made Intelligent

SIMI is a general-purpose Rust crate for string similarity. It provides
eight algorithms across three categories, a composable preprocessing
layer, an automatic routing pipeline, and batched parallel evaluation
backed by rayon.

## Why SIMI

Developers regularly reach for an LLM to compare two strings, detect
near-duplicates, or rank documents by relevance. LLMs are expensive and
slow. Most of these tasks can be solved locally with deterministic
algorithms.

SIMI gives you those algorithms in one library: pick the right tool for
the data, compose it with preprocessing, let the router cascade through
confidence tiers, or run thousands of comparisons in parallel on every
CPU core.

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
cargo add simi
```

Python (PyPI):
```bash
pip install simi-sim
```

Node.js (npm):
```bash
npm install @siktec-lab/simi
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
lowercase conversion, stopword removal (150+ built-in, or bring your own).

### Router (`simi::router`)

The SimBouncer pipeline automates algorithm selection by cascading
through confidence tiers:

```rust
use simi::router::{SimBouncer, Strategy, Threshold, Algo};

let result = SimBouncer::new()
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

## Use Cases

### Avoiding LLM Calls for Simple Comparisons

Instead of calling GPT for "are these two company names the same?":

```rust
use simi::router::{SimBouncer, Strategy, Threshold, Algo};

let result = SimBouncer::new()
    .preprocess(true)
    .tier_1(Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),
        Threshold::LessThan(0.10))
    .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
    .fallback(|a, b| {
        // Only call the LLM as a last resort
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

let fingerprint_a = simhash::compute("article body text...", 4);
let fingerprint_b = simhash::compute("article body text...", 4);

let similarity = simhash::compare_fingerprints(fingerprint_a, fingerprint_b);
if similarity > 0.90 {
    // Near-duplicate detected
}
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
├── router      -- SimBouncer pipeline builder
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
