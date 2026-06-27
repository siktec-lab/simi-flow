// Simple test for SIMI Node.js bindings.
// Run: node test.js

const simi = require('./index.js');
const assert = require('assert');

// Levenshtein
assert.strictEqual(simi.levenshtein_similarity('hello', 'hello'), 1.0);
const lev = simi.levenshtein_similarity('kitten', 'sitting');
assert(lev > 0.5 && lev < 0.65, `levenshtein kitten/sitting = ${lev}`);

// Jaro-Winkler
const jw = simi.jaro_winkler_similarity('MARTHA', 'MARHTA');
assert(jw > 0.95 && jw < 0.97, `jaro-winkler = ${jw}`);

// Hamming
assert.strictEqual(simi.hamming_similarity('hello', 'hello'), 1.0);
assert.strictEqual(simi.hamming_similarity('abc', 'xyz'), 0.0);

// Jaccard
const jac = simi.jaccard_similarity('hello world', 'hello there', 2);
assert(jac >= 0.0 && jac <= 1.0, `jaccard = ${jac}`);

// BM25
const bm25 = simi.bm25_similarity('the quick brown fox', 'the quick brown fox');
assert(Math.abs(bm25 - 1.0) < 0.01, `bm25 identical = ${bm25}`);

// TF-IDF
const tfidf = simi.tfidf_similarity('the quick brown fox', 'the quick blue fox');
assert(tfidf > 0.3 && tfidf < 1.0, `tfidf = ${tfidf}`);

// Clean text
assert.strictEqual(simi.clean_text('  Hello   World!  '), 'hello world!');

console.log('All tests passed!');
