# SIMI Router -- The Correct Way to Use SIMI

The SimBouncer is the recommended entry point for most similarity
workloads. Instead of picking an algorithm manually and writing fallback
logic, you declare a pipeline of confidence thresholds and let the
router cascade through them.

## Why Use the Router

Picking the right algorithm for every input pair is hard:
- Levenshtein catches typos but ignores word order.
- Jaccard handles token overlap but misses edit proximity.
- BM25 weights rare terms correctly but costs more.

The router removes the guesswork: you set thresholds that express "I am
confident these are the same" and "I am confident these are different."
Everything in the middle cascades to a heavier algorithm or your own
API hook.

## Basic Usage

```rust
use simi::router::{SimBouncer, Strategy, Threshold, Algo};

let result = SimBouncer::new()
    .strategy(Strategy::Cascade)
    .tier_1(
        Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),  // obvious match
        Threshold::LessThan(0.10),     // obvious mismatch
    )
    .compare("MARTHA", "MARHTA")
    .unwrap();

println!("Score: {:.3}", result.score);        // ~0.961
println!("Tier: {}", result.tier);             // 1
println!("Algorithm: {}", result.algorithm);   // jaro_winkler
```

## How the Pipeline Works

When you call `.compare(a, b)`, the router executes:

```
Step 1: Preprocess (if enabled)
    Normalize, lowercase, collapse whitespace, remove stopwords.

Step 2: Tier 1 (Fast Pass)
    Run the fast algorithm (e.g., Jaro-Winkler).
    If score > match_threshold (0.95) -> return "match".
    If score < mismatch_threshold (0.10) -> return "mismatch".
    Otherwise, continue to Tier 2.

Step 3: Tier 2 (Heavy Local Pass)
    Run the heavier algorithm (e.g., BM25).
    If score is within the configured range -> return the result.
    Otherwise, continue to Tier 3.

Step 4: Tier 3 (API Hook / Fallback)
    Call your provided callback function.
    Return whatever it returns.
```

If no fallback is configured, the router returns the Tier 1 result
as the best available score.

## Threshold Types

### GreaterThan

```
Threshold::GreaterThan(0.95)
```
The score is a match if it exceeds this value. Use for "I am sure these
are the same" decisions.

### LessThan

```
Threshold::LessThan(0.10)
```
The score is a mismatch if it falls below this value. Use for "I am sure
these are different" decisions.

### Between

```
Threshold::Between(0.60, 0.94)
```
The score is accepted if it falls in this inclusive range. Used in Tier 2
to say "the heavier algorithm confirmed similarity in the ambiguous
range."

## Algorithm Selectors

| Algo | Parameters | Cost |
|---|---|---|
| `Algo::Levenshtein` | None | O(n*m) |
| `Algo::JaroWinkler` | None | O(n+m) |
| `Algo::Hamming` | None | O(n) |
| `Algo::Jaccard(n)` | n-gram size | O(n+m) |
| `Algo::JaccardBigram` | None | O(n+m) |
| `Algo::JaccardTrigram` | None | O(n+m) |
| `Algo::JaccardWord` | None | O(n+m) |
| `Algo::MinHash(shingle_size, num_hashes)` | Both params | O(h*s) |
| `Algo::MinHashDefault` | None (3, 128) | O(h*s) |
| `Algo::SimHash(shingle_size)` | Shingle size | O(s*64) |
| `Algo::SimHashDefault` | None (4) | O(s*64) |
| `Algo::Bm25` | None | O(|vocab|) |
| `Algo::TfIdf` | None | O(|vocab|) |

## Preprocessing

Enable preprocessing to normalize inputs before comparison:

```rust
SimBouncer::new()
    .preprocess(true)
    // ...
```

Or bring your own preprocessor:

```rust
use simi::preprocess::Preprocessor;

let pre = Preprocessor::new()
    .with_lowercase(true)
    .with_remove_stopwords(true);

SimBouncer::new()
    .with_preprocessor(pre)
    // ...
```

Preprocessing is applied to both inputs before any algorithm runs.
Identical inputs after preprocessing always score 1.0.

## The Fallback (API Hook)

Tier 3 is your escape hatch. Use it for:

- Calling an LLM API as a last resort.
- Doing a database lookup.
- Asking a human operator.
- Running an expensive custom comparison.

```rust
SimBouncer::new()
    .tier_1(Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),
        Threshold::LessThan(0.10))
    .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.95))
    .fallback(|a, b| {
        // Only reaches here when both tiers are inconclusive
        let api_result = call_expensive_api(a, b);
        (api_result.score, Some(api_result.reason))
    })
    .compare(a, b)?;
```

The callback receives the preprocessed strings (if preprocessing is
enabled) and returns `(Score, Option<String>)`. The optional string is
stored in `ComparisonResult.fallback_data`.

## The Returned Result

```rust
pub struct ComparisonResult {
    pub score: f64,                // [0.0, 1.0]
    pub tier: usize,               // 1, 2, or 3
    pub algorithm: String,         // algorithm name
    pub fallback_called: bool,     // Tier 3 invoked?
    pub fallback_data: Option<String>, // callback metadata
}
```

Check `result.tier` to know which tier produced the score. Check
`result.fallback_called` to know if your API hook fired.

## Strategy

Only one strategy exists today:

```rust
SimBouncer::new()
    .strategy(Strategy::Cascade)
```

Cascade means: Tier 1 -> Tier 2 -> Tier 3, stopping as soon as a tier
produces a decisive result.

## Recommended Pipeline Patterns

### Pattern 1: Typos and Names

For comparing user-entered names against a reference database:

```rust
SimBouncer::new()
    .preprocess(true)
    .tier_1(Algo::JaroWinkler,
        Threshold::GreaterThan(0.95),
        Threshold::LessThan(0.20))
    .tier_2(Algo::Levenshtein, Threshold::Between(0.60, 0.94))
    .compare(input, reference)?
```

Jaro-Winkler catches close matches with prefix bias; Levenshtein handles
transpositions that Winkler misses.

### Pattern 2: Document Similarity

For comparing short paragraphs or product descriptions:

```rust
SimBouncer::new()
    .preprocess(true)
    .tier_1(Algo::JaccardWord,
        Threshold::GreaterThan(0.90),
        Threshold::LessThan(0.05))
    .tier_2(Algo::Bm25, Threshold::Between(0.40, 0.89))
    .fallback(|a, b| call_llm(a, b))
    .compare(doc_a, doc_b)?
```

Word-level Jaccard catches obvious overlap; BM25 weights important terms;
the LLM only fires for genuinely ambiguous cases.

### Pattern 3: Bulk Deduplication

For large-scale deduplication, skip the router and use batch directly:

```rust
use simi::batch::BatchComparator;
use simi::router::Algo;

let cmp = BatchComparator::new(Algo::SimHashDefault);
let results = cmp.compare_matrix(&documents, &documents)?;
```

The router is for decisions; batch is for throughput.

## Common Mistakes

**Using Levenshtein for long documents.**
Levenshtein counts character edits. A 5000-character document with one
missing sentence gets a misleadingly high score. Use BM25 or Jaccard for
documents longer than a sentence.

**Comparing unequal-length strings with Hamming.**
`hamming::similarity` returns `None` for unequal lengths. Handle this
before passing `Algo::Hamming` to the router, or expect runtime errors.

**Skipping preprocessing.**
`"Hello World"` and `"hello world"` have Levenshtein similarity 0.0.
Enable preprocessing for case-insensitive matching.

**Setting thresholds too narrow.**
If the match threshold is 0.99 and the mismatch threshold is 0.01, almost
everything cascades to Tier 2 or 3. Set thresholds based on your domain:
names might need 0.95, but product descriptions might be fine at 0.80.

## Summary

- Use the router when you do not want to pick an algorithm manually.
- Declare what you are sure about (thresholds), not what algorithm to use.
- Let the Cascade strategy handle the rest.
- Keep the router for decision-making; use batch for throughput.
- The fallback is your safety net for truly ambiguous cases.
