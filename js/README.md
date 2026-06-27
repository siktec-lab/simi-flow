# SIMI Node.js Bindings

Node.js bindings for the [SIMI](https://github.com/siktec-lab/simi-flow) similarity
toolkit, powered by napi-rs. SIMI provides eight similarity algorithms and
a text preprocessing layer.

## Installation

```bash
npm install @siktec-lab/simi-flow
```

Requires Node.js 18 or later.

## Algorithms

All similarity functions return a normalized score in `[0.0, 1.0]` where
`1.0 = identical`.

### Levenshtein (edit distance)

```javascript
const simi = require('@siktec-lab/simi-flow');

// Raw distance
simi.levenshtein_distance('kitten', 'sitting');  // 3

// Normalized similarity
simi.levenshtein_similarity('kitten', 'sitting');  // 0.571
```

### Jaro-Winkler (names and short strings)

```javascript
simi.jaro_winkler_similarity('MARTHA', 'MARHTA');  // 0.961
simi.jaro_winkler_similarity('DWAYNE', 'DUANE');   // 0.840
```

### Hamming (equal-length codes)

Throws an error if the strings have different lengths.

```javascript
simi.hamming_distance('karolin', 'kathrin');        // 3
simi.hamming_similarity('karolin', 'kathrin');      // 0.571
simi.hamming_similarity('hello', 'hello');          // 1.0

// Different-length strings throw
simi.hamming_similarity('abc', 'abcd');  // throws: "Strings must have equal length"
```

### Jaccard (n-grams and word sets)

```javascript
// Configurable n-gram size
simi.jaccard_similarity('hello', 'hallo', 2);

// Convenience functions
simi.jaccard_bigram_similarity('hello', 'hallo');
simi.jaccard_trigram_similarity('hello', 'hallo');
simi.jaccard_word_similarity('the quick brown fox', 'the quick lazy dog');
```

### MinHash (document fingerprinting)

```javascript
// Get a 128-hash signature
const sig = simi.minhash_signature('large document text...', 3, 128);
// sig is an array of 128 numbers

// Compare with custom parameters
simi.minhash_similarity(a, b, 3, 128);

// Compare with defaults (shingle=3, hashes=128)
simi.minhash_similarity_default(a, b);
```

### SimHash (64-bit LSH fingerprints)

```javascript
// Get a 64-bit fingerprint as a number
const fp = simi.simhash_fingerprint('document text', 4);
const fp = simi.simhash_fingerprint_default('document text');  // shingle_size=4

// Compare
simi.simhash_similarity(a, b, 4);
simi.simhash_similarity_default(a, b);
```

### BM25 (probabilistic retrieval)

```javascript
simi.bm25_similarity('the quick brown fox', 'the quick blue fox');  // 0.5..0.8
simi.bm25_similarity('the quick brown fox', 'the quick brown fox');  // 1.0
```

### TF-IDF + Cosine (term-weighted vectors)

```javascript
simi.tfidf_similarity('the quick brown fox', 'the quick blue fox');  // 0.5..0.7
simi.tfidf_similarity('abc', 'xyz');                                  // 0.0
```

## Preprocessing

Normalize text before comparison:

```javascript
// Quick one-liners
simi.clean_text('  Hello   World!  ');               // 'hello world!'
simi.clean_text_stopwords('the quick brown fox');     // 'quick brown fox'
```

`clean_text` applies: Unicode NFC normalization, lowercase, whitespace
collapse, and trimming. `clean_text_stopwords` does the same plus removes
150+ common English stopwords.

## Performance

SIMI is built in Rust with napi-rs, so calls run at native speed:

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

These timings are from the Rust core. The npm binding adds minimal FFI
overhead per call.

## Reference

### All exported functions

| Function | Parameters | Returns |
|---|---|---|
| `levenshtein_distance` | `a, b` | `number` |
| `levenshtein_similarity` | `a, b` | `number` |
| `jaro_winkler_similarity` | `a, b` | `number` |
| `hamming_distance` | `a, b` | `number` |
| `hamming_similarity` | `a, b` | `number` |
| `jaccard_similarity` | `a, b, n` | `number` |
| `jaccard_bigram_similarity` | `a, b` | `number` |
| `jaccard_trigram_similarity` | `a, b` | `number` |
| `jaccard_word_similarity` | `a, b` | `number` |
| `minhash_signature` | `text, shingle_size, num_hashes` | `number[]` |
| `minhash_similarity` | `a, b, shingle_size, num_hashes` | `number` |
| `minhash_similarity_default` | `a, b` | `number` |
| `simhash_fingerprint` | `text, shingle_size` | `number` |
| `simhash_fingerprint_default` | `text` | `number` |
| `simhash_similarity` | `a, b, shingle_size` | `number` |
| `simhash_similarity_default` | `a, b` | `number` |
| `bm25_similarity` | `a, b` | `number` |
| `tfidf_similarity` | `a, b` | `number` |
| `clean_text` | `text` | `string` |
| `clean_text_stopwords` | `text` | `string` |

## License

MIT -- see the [LICENSE](https://github.com/siktec-lab/simi-flow/blob/main/LICENSE) file.
