// Comprehensive tests for SIMI Node.js bindings.
// Run: node test.js

const simi = require('./index.js');
const assert = require('assert');

// ─── Helpers ──────────────────────────────────────────────────────

function assertClose(actual, expected, delta, label) {
    const diff = Math.abs(actual - expected);
    assert(diff < delta, `${label || ''}: expected ~${expected}, got ${actual} (diff ${diff})`);
}

function assertNormalized(score, label) {
    assert(score >= 0.0 && score <= 1.0,
        `${label || 'score'}: ${score} not in [0, 1]`);
}

// ─── Levenshtein ───────────────────────────────────────────────────

console.log('Levenshtein...');
assert.strictEqual(simi.levenshtein_distance('kitten', 'sitting'), 3);
assert.strictEqual(simi.levenshtein_distance('hello', 'hello'), 0);
assert.strictEqual(simi.levenshtein_distance('', 'abc'), 3);
assert.strictEqual(simi.levenshtein_distance('', ''), 0);

assert.strictEqual(simi.levenshtein_similarity('hello', 'hello'), 1.0);
assert.strictEqual(simi.levenshtein_similarity('abc', 'xyz'), 0.0);
const lev = simi.levenshtein_similarity('kitten', 'sitting');
assertClose(lev, 0.571, 0.01, 'levenshtein kitten/sitting');

// symmetry
const a = simi.levenshtein_similarity('ab', 'ba');
const b = simi.levenshtein_similarity('ba', 'ab');
assert.strictEqual(a, b);

// ─── Jaro-Winkler ──────────────────────────────────────────────────

console.log('Jaro-Winkler...');
assert.strictEqual(simi.jaro_winkler_similarity('hello', 'hello'), 1.0);
const jw = simi.jaro_winkler_similarity('MARTHA', 'MARHTA');
assert(jw > 0.95 && jw < 0.97, `jaro-winkler = ${jw}`);
const jw2 = simi.jaro_winkler_similarity('abc', 'xyz');
assert(jw2 < 0.2, `jaro-winkler abc/xyz = ${jw2}`);

// ─── Hamming ───────────────────────────────────────────────────────

console.log('Hamming...');
assert.strictEqual(simi.hamming_distance('karolin', 'kathrin'), 3);
assert.strictEqual(simi.hamming_distance('hello', 'hello'), 0);
assert.strictEqual(simi.hamming_distance('abc', 'xyz'), 3);

assert.strictEqual(simi.hamming_similarity('hello', 'hello'), 1.0);
assert.strictEqual(simi.hamming_similarity('abc', 'xyz'), 0.0);
const ham = simi.hamming_similarity('karolin', 'kathrin');
assertClose(ham, 0.57142, 0.001, 'hamming');

// unequal lengths throw
assert.throws(() => simi.hamming_distance('abc', 'abcd'), /equal length/i);
assert.throws(() => simi.hamming_similarity('abc', 'abcd'), /equal length/i);

// ─── Jaccard ───────────────────────────────────────────────────────

console.log('Jaccard...');
assert.strictEqual(simi.jaccard_similarity('hello', 'hello', 2), 1.0);
const jac = simi.jaccard_similarity('hello', 'hallo', 2);
assert(jac > 0.3 && jac < 0.7, `jaccard hello/hallo = ${jac}`);

assert.strictEqual(simi.jaccard_bigram_similarity('hello', 'hello'), 1.0);
assertNormalized(simi.jaccard_bigram_similarity('abc', 'xyz'), 'jaccard bigram');

assert.strictEqual(simi.jaccard_trigram_similarity('hello', 'hello'), 1.0);

assert.strictEqual(simi.jaccard_word_similarity('a b c', 'a b c'), 1.0);
const jwv = simi.jaccard_word_similarity('the quick brown fox', 'the quick lazy dog');
assertClose(jwv, 0.333, 0.01, 'jaccard word');

// ─── MinHash ───────────────────────────────────────────────────────

console.log('MinHash...');
const sig = simi.minhash_signature('hello world', 3, 128);
assert.strictEqual(sig.length, 128);
assert(sig.every(v => typeof v === 'number'));

const mh = simi.minhash_similarity('hello world', 'hello world', 3, 128);
assert(mh > 0.9, `minhash identical = ${mh}`);

const mh2 = simi.minhash_similarity_default('hello world', 'hello world');
assert(mh2 > 0.9, `minhash_default identical = ${mh2}`);

// symmetry
const mha = simi.minhash_similarity_default('hello world', 'hello there');
const mhb = simi.minhash_similarity_default('hello there', 'hello world');
assert.strictEqual(mha, mhb);

// ─── SimHash ───────────────────────────────────────────────────────

console.log('SimHash...');
const fp = simi.simhash_fingerprint('hello world', 4);
assert.strictEqual(typeof fp, 'number');
const fp2 = simi.simhash_fingerprint('hello world', 4);
assert.strictEqual(fp, fp2);  // deterministic

assert.strictEqual(typeof simi.simhash_fingerprint_default('the quick brown fox'), 'number');

assert.strictEqual(simi.simhash_similarity('hello world', 'hello world', 4), 1.0);
const sh = simi.simhash_similarity_default('hello world', 'hello world');
assert(sh > 0.9, `simhash_default identical = ${sh}`);

const sh2 = simi.simhash_similarity_default('the quick brown fox', 'lorem ipsum dolor sit');
assertNormalized(sh2, 'simhash different');

// ─── BM25 ──────────────────────────────────────────────────────────

console.log('BM25...');
assert.strictEqual(simi.bm25_similarity('the quick brown fox', 'the quick brown fox'), 1.0);
const bm = simi.bm25_similarity('the quick brown fox', 'the quick blue fox');
assert(bm > 0.3 && bm < 0.95, `bm25 = ${bm}`);

// ─── TF-IDF ────────────────────────────────────────────────────────

console.log('TF-IDF...');
assert.strictEqual(simi.tfidf_similarity('the quick brown fox', 'the quick brown fox'), 1.0);
const tf = simi.tfidf_similarity('the quick brown fox', 'the quick blue fox');
assert(tf > 0.3 && tf < 0.95, `tfidf = ${tf}`);
assert.strictEqual(simi.tfidf_similarity('abc', 'xyz'), 0.0);

// symmetry
assert.strictEqual(
    simi.tfidf_similarity('the quick brown fox', 'the quick red fox'),
    simi.tfidf_similarity('the quick red fox', 'the quick brown fox')
);

// ─── Preprocessor ──────────────────────────────────────────────────

console.log('Preprocessor...');
assert.strictEqual(simi.clean_text('  Hello   World!  '), 'hello world!');
assert.strictEqual(simi.clean_text(''), '');

const sw = simi.clean_text_stopwords('the quick brown fox jumps over the lazy dog');
assert(!sw.includes('the'), `stopwords not removed: ${sw}`);
assert(sw.includes('quick'));
assert(sw.includes('fox'));

// ─── Edge cases ────────────────────────────────────────────────────

console.log('Edge cases...');
assert.strictEqual(simi.levenshtein_similarity('', ''), 1.0);
assert.strictEqual(simi.jaro_winkler_similarity('', ''), 1.0);
assert.strictEqual(simi.levenshtein_similarity('', 'hello'), 0.0);
assert.strictEqual(simi.tfidf_similarity('', 'hello'), 0.0);

// long strings
const longA = 'a'.repeat(1000);
const longB = 'a'.repeat(999) + 'b';
const longScore = simi.levenshtein_similarity(longA, longB);
assert(longScore > 0.99, `long string = ${longScore}`);

// ─── Coverage sweep ────────────────────────────────────────────────

console.log('Coverage sweep...');
const pairs = [
    ['hello', 'hello'],
    ['hello', 'world'],
    ['the quick brown fox', 'the quick brown fox'],
    ['the quick brown fox', 'the lazy dog'],
    ['MARTHA', 'MARHTA'],
    ['kitten', 'sitting'],
    ['', ''],
    ['hello', ''],
    ['abc', 'xyz'],
];

for (const [a, b] of pairs) {
    assertNormalized(simi.levenshtein_similarity(a, b), `lev ${a}/${b}`);
    assertNormalized(simi.jaro_winkler_similarity(a, b), `jw ${a}/${b}`);
    if (a.length === b.length) {
        assertNormalized(simi.hamming_similarity(a, b), `hamming ${a}/${b}`);
    }
    assertNormalized(simi.jaccard_bigram_similarity(a, b), `jac ${a}/${b}`);
    assertNormalized(simi.minhash_similarity_default(a, b), `minhash ${a}/${b}`);
    assertNormalized(simi.simhash_similarity_default(a, b), `simhash ${a}/${b}`);
    assertNormalized(simi.bm25_similarity(a, b), `bm25 ${a}/${b}`);
    assertNormalized(simi.tfidf_similarity(a, b), `tfidf ${a}/${b}`);
}

console.log('\nAll tests passed.');
