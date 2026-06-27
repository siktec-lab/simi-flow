# SIMI Python Bindings

Python bindings for the [SIMI](https://github.com/siktec-lab/simi-flow) similarity
toolkit, powered by PyO3. SIMI provides eight similarity algorithms, a
composable preprocessing layer, and the SimiFlow routing pipeline.

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
