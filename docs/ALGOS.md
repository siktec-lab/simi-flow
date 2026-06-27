# SIMI -- Algorithm Reference

This document describes every similarity algorithm implemented in SIMI:
its mathematical foundation, time and space complexity, and the data types
it works best on.

---

## 1. Levenshtein Distance

**Category:** Short Strings and Typos

**What it measures:** The minimum number of single-character edits
(insertions, deletions, substitutions) needed to turn one string into
the other.

**Normalized score:** `1.0 - (distance / max(len(a), len(b)))`

**Implementation:** Single-row dynamic programming with O(min(n,m)) space.

**Complexity:** O(n * m) time, O(min(n, m)) space.

**Best for:** Typos, misspellings, and short strings where every character
counts.

**Example:**
```
levenshtein::similarity("kitten", "sitting")  -> 0.571
levenshtein::similarity("hello", "hello")     -> 1.000
levenshtein::similarity("abc", "xyz")         -> 0.000
```

---

## 2. Jaro-Winkler

**Category:** Short Strings and Typos

**What it measures:** The Jaro similarity counts matching characters within
a sliding window and shared transpositions. Winkler adds a prefix bonus for
strings that share a common prefix (heavily favors names and short
identifiers).

**Parameters:**
- Scaling factor: 0.1
- Maximum prefix length: 4 characters
- Match window: `max(len(a), len(b)) / 2 - 1`

**Normalized score:** `[0.0, 1.0]`, where 1.0 = identical.

**Complexity:** O(n + m) time, O(n + m) space.

**Best for:** Personal names, brand names, short identifiers where the
beginning of the string carries the most signal.

**Example:**
```
jaro_winkler::similarity("MARTHA", "MARHTA")  -> 0.961
jaro_winkler::similarity("DWAYNE", "DUANE")   -> 0.840
jaro_winkler::similarity("hello", "hello")    -> 1.000
```

---

## 3. Hamming Distance

**Category:** Short Strings and Typos

**What it measures:** Counts positions where corresponding characters
differ. Requires strings of equal length.

**Normalized score:** `1.0 - (differing_positions / length)`

**Safety:** Returns `Option<f64>`. Returns `None` if the inputs have
different lengths, forcing the caller to handle the invariant.

**Complexity:** O(n) time, O(1) space.

**Best for:** Equal-length codes, checksums, binary strings, fixed-width
identifiers where position matters.

**Example:**
```
hamming::similarity("karolin", "kathrin")  -> Some(0.625)
hamming::similarity("abc", "xyz")          -> Some(0.000)
hamming::similarity("abc", "abcd")         -> None
```

---

## 4. Jaccard Similarity

**Category:** Sets and Documents

**What it measures:** Jaccard similarity between two sets of n-grams
(or words):

```
J(A, B) = |A ∩ B| / |A ∪ B|
```

**Variants:**
| Function | Set Type |
|---|---|
| `jaccard::similarity(a, b, n)` | Configurable n-grams |
| `jaccard::bigram_similarity(a, b)` | Bigrams (n=2) |
| `jaccard::trigram_similarity(a, b)` | Trigrams (n=3) |
| `jaccard::word_similarity(a, b)` | Whole words, split on whitespace |

**Complexity:** O(n + m) time with hash sets. O(k) space where k is the
number of distinct n-grams.

**Best for:** Comparing short texts by overlapping n-grams, fuzzy matching
on token sets.

**Example:**
```
jaccard::bigram_similarity("hello", "hallo")     -> ~0.429
jaccard::word_similarity("a b c", "a b d")       -> 0.500
jaccard::trigram_similarity("hello", "hello")    -> 1.000
```

---

## 5. MinHash

**Category:** Sets and Documents (Probabilistic)

**What it measures:** Estimates Jaccard similarity using a fixed number of
hash functions. Converts a document into a fingerprint (signature) of
`num_hashes` values via shingling and the minimum hash value per function.

**Parameters:**
- `shingle_size` (default: 3): character n-gram size
- `num_hashes` (default: 128): number of hash permutations

**Normalized score:** Fraction of matching MinHash values (approximates
Jaccard).

**Complexity:** O(num_hashes * shingles) time per fingerprint.

**Best for:** Large document sets, near-duplicate detection, clustering at
scale where exact Jaccard is too expensive.

**Example:**
```
minhash::compare_default("hello world", "hello there")  -> 0.45..0.55
minhash::compare_default("hello world", "hello world")  -> ~1.0
```

---

## 6. SimHash

**Category:** Sets and Documents (Probabilistic)

**What it measures:** Produces a 64-bit fingerprint via Locality-Sensitive
Hashing (LSH). Similar documents produce similar bit patterns; Hamming
distance between fingerprints approximates cosine similarity.

**Parameters:**
- `shingle_size` (default: 4): character n-gram size

**Normalized score:** `1.0 - (hamming_distance / 64)`

**Complexity:** O(shingles * 64) time per fingerprint.

**Best for:** Near-duplicate detection on large corpora, content
deduplication, spam detection.

**Example:**
```
simhash::compare_default("hello world", "hello there")  -> 0.75..0.90
simhash::compare_default("hello world", "hello world")  -> 1.0
```

---

## 7. BM25

**Category:** Statistical Meaning

**What it measures:** A probabilistic retrieval function that scores how
well a query matches a document. Uses term frequency (TF) and inverse
document frequency (IDF) with saturation and length normalization.

**Formula (simplified):**
```
BM25(d, q) = sum over terms in q of:
    IDF(t) * (TF(t,d) * (k1 + 1)) / (TF(t,d) + k1 * (1 - b + b * (|d| / avgdl)))
```

**Parameters:**
- `k1` (default: 1.2): term frequency saturation
- `b` (default: 0.75): length normalization factor

**Normalized score:** The internal BM25 sum is normalized to `[0.0, 1.0]`
by dividing by the self-score of the query. A `Bm25Index` is built from
both input strings for the comparison.

**Complexity:** O(|vocab|) to build index; O(|query_terms|) to score.

**Best for:** Search relevance ranking, comparing documents where some
words matter more than others (rare words carry more weight).

**Example:**
```
bm25::similarity("the quick brown fox", "the quick brown fox")  -> 1.0
bm25::similarity("the quick brown fox", "the quick red fox")    -> 0.5..0.8
```

---

## 8. TF-IDF + Cosine Similarity

**Category:** Statistical Meaning

**What it measures:** Represents each string as a weighted vector in a
high-dimensional space, then computes the cosine of the angle between
the two vectors:

```
cos(theta) = (A · B) / (|A| * |B|)
```

Where each dimension is a term weighted by:
```
TF-IDF = TF(t,d) * log(N / DF(t))
```

**Normalized score:** `[0.0, 1.0]` by definition of cosine similarity
(non-negative weights).

**Complexity:** O(|vocab|) time, O(|vocab|) space per vector.

**Best for:** Comparing documents by overall topic similarity, information
retrieval, and any task where understanding which terms are
"characteristic" of a document matters.

**Example:**
```
tfidf::similarity("the quick brown fox", "the quick brown fox")  -> 1.0
tfidf::similarity("the quick brown fox", "the lazy dog")         -> 0.0
tfidf::similarity("the quick brown fox", "the quick blue fox")   -> 0.5..0.7
```

---

## Algorithm Selection Guide

| Your data looks like | Use |
|---|---|
| Short strings (<50 chars), typos matter | Levenshtein or Jaro-Winkler |
| Equal-length codes or binary strings | Hamming |
| Token sets, keyword overlap | Jaccard (word or n-gram) |
| Large document corpus, near-duplicate detection | MinHash or SimHash |
| Search relevance, "some words matter more" | BM25 |
| Document topic comparison, information retrieval | TF-IDF + Cosine |

For an automated selection pipeline, use the **SimBouncer** router, which
cascades through algorithms based on confidence thresholds. See
[ROUTER.md](ROUTER.md) for details.
