# SIMI — a Similarity & Text-Analysis Engine for Python

Python bindings for [SIMI](https://github.com/siktec-lab/simi-flow), a production-grade
similarity and text-analysis toolkit powered by PyO3 — a Rust core with the ergonomics of a
plain Python module. Use it to build and integrate reliable similarity checks across real
workloads: **bot/abuse protection, spam & content moderation, record matching, deduplication,
search ranking, and fuzzy input handling.**

- **8 battle-tested algorithms** behind one clean API (edit distance, name matching, set overlap, document fingerprinting, probabilistic retrieval) — every score normalized to `[0.0, 1.0]`.
- **SimiFlow routing** — tell it your *intent* (`names`, `typos`, `codes`, `documents`, `dedup`, `auto`) and it picks the right algorithm for you.
- **Confidence cascade** — resolve clear matches/mismatches with a cheap fast pass and escalate only the ambiguous middle to a heavier algorithm.
- **Native speed** — algorithm calls run at Rust speed with tiny FFI overhead.

```python
import simi

sf = simi.SimiFlow()
# Declare what you're comparing; SIMI routes "names" to Jaro-Winkler and runs it natively.
sf.compare_with_intent("names", "MARTHA", "MARHTA")
# {'score': 0.961, 'tier': 0, 'algorithm': 'jaro_winkler', ...}
```

> **A note on origin.** SIMI grew out of a need to cut the cost, latency, and unpredictability
> of using an LLM for every "are these the same?" decision. Most of those checks are
> deterministic and belong in fast, testable local code — which is exactly what SIMI provides.

## Installation

```bash
pip install simi-flow
```

Requires Python 3.8 or later.

## Algorithms

SIMI exposes every algorithm as a standalone function. All similarity
functions return a normalized score in `[0.0, 1.0]` where `1.0 = identical`.

### Levenshtein (edit distance)

```python
import simi

# Raw distance
simi.levenshtein_distance("kitten", "sitting")  # 3

# Normalized similarity
simi.levenshtein_similarity("kitten", "sitting")  # 0.571
```

### Jaro-Winkler (names and short strings)

```python
simi.jaro_winkler_similarity("MARTHA", "MARHTA")  # 0.961
simi.jaro_winkler_similarity("DWAYNE", "DUANE")   # 0.840
```

### Hamming (equal-length codes)

Raises `ValueError` if the strings have different lengths.

```python
simi.hamming_distance("karolin", "kathrin")        # 3
simi.hamming_similarity("karolin", "kathrin")      # 0.571
simi.hamming_similarity("hello", "hello")          # 1.0
```

### Jaccard (n-grams and word sets)

```python
# Configurable n-gram size
simi.jaccard_similarity("hello", "hallo", n=2)

# Convenience functions
simi.jaccard_bigram_similarity("hello", "hallo")
simi.jaccard_trigram_similarity("hello", "hallo")
simi.jaccard_word_similarity("the quick brown fox", "the quick lazy dog")
```

### MinHash (document fingerprinting)

```python
# Get a 128-hash signature
sig = simi.minhash_signature("large document text...", shingle_size=3, num_hashes=128)

# Compare with custom parameters
simi.minhash_similarity(a, b, shingle_size=3, num_hashes=128)

# Compare with defaults (shingle=3, hashes=128)
simi.minhash_similarity_default(a, b)
```

### SimHash (64-bit LSH fingerprints)

```python
# Get a 64-bit fingerprint
fp = simi.simhash_fingerprint("document text", shingle_size=4)
fp = simi.simhash_fingerprint_default("document text")  # shingle_size=4

# Compare
simi.simhash_similarity(a, b, shingle_size=4)
simi.simhash_similarity_default(a, b)
```

### BM25 (probabilistic retrieval)

```python
simi.bm25_similarity("the quick brown fox", "the quick blue fox")  # 0.5..0.8
simi.bm25_similarity("the quick brown fox", "the quick brown fox")  # 1.0
```

### TF-IDF + Cosine (term-weighted vectors)

```python
simi.tfidf_similarity("the quick brown fox", "the quick blue fox")  # 0.5..0.7
simi.tfidf_similarity("abc", "xyz")                                  # 0.0
```

## Preprocessing

Normalize text before comparison to reduce noise:

```python
# Quick one-liner
simi.clean_text("  Hello   World!  ")          # "hello world!"
simi.clean_text_stopwords("the quick brown fox")  # "quick brown fox"

# Builder pattern
from simi import Preprocessor

pre = Preprocessor() \
    .with_lowercase(True) \
    .with_collapse_whitespace(True) \
    .with_trim(True) \
    .with_normalize_unicode(True) \
    .with_remove_stopwords(True)

cleaned = pre.process("The Quick Brown Fox")
# "quick brown fox"
```

Available builder options:
- `with_lowercase(bool)`
- `with_collapse_whitespace(bool)`
- `with_trim(bool)`
- `with_normalize_unicode(bool)`
- `with_remove_stopwords(bool)`
- `with_stopwords(list[str])` -- custom stopword list
- `with_max_length(int)`

## SimiFlow Router

The headline feature. Two ways to use it:

1. **Intent routing** (`compare_with_intent`) — say what you're comparing, get the right algorithm.
2. **Cascade** (`tier_1` → `tier_2`) — answer confident cases with a cheap algorithm, escalate
   only the ambiguous middle to a heavier local pass. You inspect the result `tier` to see how
   often the expensive path runs — and route those few gray-zone cases to your own LLM call.

The router cascades through algorithms based on confidence thresholds,
avoiding expensive computation until it is actually needed:

```python
from simi import SimiFlow

sf = SimiFlow() \
    .preprocess(True) \
    .tier_1("jaro_winkler", "gt", 0.95, "lt", 0.10) \
    .tier_2("bm25", "between", 0.60, 0.94)

result = sf.compare("MARTHA", "MARHTA")
# {
#   "score": 0.961,
#   "tier": 1,
#   "algorithm": "jaro_winkler",
#   "fallback_called": False,
#   "fallback_data": None,
# }
```

Algorithm names for the router: `"levenshtein"`, `"jaro_winkler"`,
`"hamming"`, `"jaccard_bigram"`, `"jaccard_trigram"`, `"jaccard_word"`,
`"minhash_default"`, `"simhash_default"`, `"bm25"`, `"tfidf"`.

Threshold operators: `"gt"` (greater than), `"lt"` (less than),
`"between"` (inclusive range, for Tier 2).

### compare_with_intent

Bypass tier configuration and run a specific algorithm by intent:

```python
sf = simi.SimiFlow()

# Intent-based: Names -> Jaro-Winkler
result = sf.compare_with_intent("names", "MARTHA", "MARHTA")
# {'score': 0.961, 'tier': 0, 'algorithm': 'jaro_winkler', ...}

# Auto: inspects input lengths and picks automatically
result = sf.compare_with_intent("auto", a, b)

# All intents: names, typos, codes, documents, dedup/duplication, auto
```

## Performance

SIMI is built in Rust with PyO3, so algorithm calls run at native speed:

| Algorithm | Input | Time |
|---|---|---|
| Levenshtein | "kitten"/"sitting" | ~80 ns |
| Jaro-Winkler | "MARTHA"/"MARHTA" | ~200 ns |
| Hamming | 7-char equal | ~150 ns |
| Jaccard bigram | Short texts | ~1.7 us |
| MinHash (128) | Short doc | ~17 us |
| SimHash | Short doc | ~5 us |
| BM25 | Short docs | ~2.9 us |
| TF-IDF | Short texts | ~2.7 us |

These timings are from the Rust core. The Python binding adds a small
FFI overhead per call (~50-200 ns).

## License

MIT -- see the [LICENSE](https://github.com/siktec-lab/simi-flow/blob/main/LICENSE) file.
