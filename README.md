# SIMI -- Similarity Toolkit

A general-purpose toolkit of similarity checks, designed to protect developers from wasting compute and money on LLMs for simple tasks.

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

### The LLM Bouncer Pipeline

```rust
use simi::router::{SimBouncer, Strategy, Threshold, Algo};

let result = SimBouncer::new()
    .preprocess(true)
    .strategy(Strategy::Cascade)
    .tier_1(Algo::JaroWinkler, Threshold::GreaterThan(0.95), Threshold::LessThan(0.10))
    .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
    .fallback(|a, b| {
        // Your LLM API call here
        (0.8, Some("llm_verified".into()))
    })
    .compare("hello world", "hello there")
    .unwrap();
```

Tier 1 -- fast pass (Jaro-Winkler). Tier 2 -- heavy local pass (BM25).  
Tier 3 -- optional LLM/API hook. No more writing manual fallback logic.

### Batch Parallelism

```rust
use simi::batch::BatchComparator;
use simi::router::Algo;

let comparator = BatchComparator::new(Algo::Levenshtein);
let results = comparator.compare_pairs(&thousands_of_strings_a, &thousands_of_strings_b)?;
```

Powered by rayon -- evaluates thousands of pairs across all CPU cores.

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
cargo add simi
```

### Python (PyPI)

```bash
pip install simi-sim
```

### Node.js (npm)

```bash
npm install @siktec-lab/simi
```

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

## License

MIT -- see [LICENSE](LICENSE).
