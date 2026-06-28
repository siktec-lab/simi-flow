//! JavaScript/Node.js bindings for SIMI (via napi-rs).
//!
//! Exposes every algorithm, the preprocessor, and the SimiFlow router.

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── Levenshtein ──────────────────────────────────────────────────────────

/// Levenshtein distance (raw edit count).
#[napi(js_name = "levenshtein_distance")]
pub fn levenshtein_distance(a: String, b: String) -> u32 {
    crate::algo::levenshtein::distance(&a, &b) as u32
}

/// Levenshtein similarity (normalized 0..1).
#[napi(js_name = "levenshtein_similarity")]
pub fn levenshtein_similarity(a: String, b: String) -> f64 {
    crate::algo::levenshtein::similarity(&a, &b)
}

// ─── Jaro-Winkler ─────────────────────────────────────────────────────────

/// Jaro-Winkler similarity.
#[napi(js_name = "jaro_winkler_similarity")]
pub fn jaro_winkler_similarity(a: String, b: String) -> f64 {
    crate::algo::jaro_winkler::similarity(&a, &b)
}

// ─── Hamming ──────────────────────────────────────────────────────────────

/// Hamming distance (raw differing character count).
/// Throws if strings have different lengths.
#[napi(js_name = "hamming_distance")]
pub fn hamming_distance(a: String, b: String) -> Result<u32> {
    crate::algo::hamming::distance(&a, &b)
        .map(|d| d as u32)
        .ok_or_else(|| napi::Error::from_reason("Strings must have equal length"))
}

/// Hamming similarity (normalized 0..1).
/// Throws if strings have different lengths.
#[napi(js_name = "hamming_similarity")]
pub fn hamming_similarity(a: String, b: String) -> Result<f64> {
    crate::algo::hamming::similarity(&a, &b)
        .ok_or_else(|| napi::Error::from_reason("Strings must have equal length"))
}

// ─── Jaccard ──────────────────────────────────────────────────────────────

/// Jaccard similarity with configurable n-gram size.
#[napi(js_name = "jaccard_similarity")]
pub fn jaccard_similarity(a: String, b: String, n: i32) -> f64 {
    crate::algo::jaccard::similarity(&a, &b, n as usize)
}

/// Jaccard similarity using bigrams (n=2).
#[napi(js_name = "jaccard_bigram_similarity")]
pub fn jaccard_bigram_similarity(a: String, b: String) -> f64 {
    crate::algo::jaccard::bigram_similarity(&a, &b)
}

/// Jaccard similarity using trigrams (n=3).
#[napi(js_name = "jaccard_trigram_similarity")]
pub fn jaccard_trigram_similarity(a: String, b: String) -> f64 {
    crate::algo::jaccard::trigram_similarity(&a, &b)
}

/// Jaccard similarity at the word level.
#[napi(js_name = "jaccard_word_similarity")]
pub fn jaccard_word_similarity(a: String, b: String) -> f64 {
    crate::algo::jaccard::word_similarity(&a, &b)
}

// ─── MinHash ──────────────────────────────────────────────────────────────

/// MinHash signature as array of numbers.
#[napi(js_name = "minhash_signature")]
pub fn minhash_signature(text: String, shingle_size: i32, num_hashes: i32) -> Vec<u32> {
    let sig = crate::algo::minhash::signature(&text, shingle_size as usize, num_hashes as usize);
    sig.signatures.iter().map(|&v| v as u32).collect()
}

/// MinHash similarity between two strings.
#[napi(js_name = "minhash_similarity")]
pub fn minhash_similarity(a: String, b: String, shingle_size: i32, num_hashes: i32) -> f64 {
    crate::algo::minhash::compare(&a, &b, shingle_size as usize, num_hashes as usize)
}

/// MinHash similarity with default parameters (shingle=3, hashes=128).
#[napi(js_name = "minhash_similarity_default")]
pub fn minhash_similarity_default(a: String, b: String) -> f64 {
    crate::algo::minhash::compare_default(&a, &b)
}

// ─── SimHash ──────────────────────────────────────────────────────────────

/// SimHash 64-bit fingerprint as a number.
#[napi(js_name = "simhash_fingerprint")]
pub fn simhash_fingerprint(text: String, shingle_size: i32) -> u32 {
    crate::algo::simhash::fingerprint(&text, shingle_size as usize).0 as u32
}

/// SimHash 64-bit fingerprint with default shingle size (4).
#[napi(js_name = "simhash_fingerprint_default")]
pub fn simhash_fingerprint_default(text: String) -> u32 {
    crate::algo::simhash::fingerprint_default(&text).0 as u32
}

/// SimHash similarity between two strings.
#[napi(js_name = "simhash_similarity")]
pub fn simhash_similarity(a: String, b: String, shingle_size: i32) -> f64 {
    crate::algo::simhash::compare(&a, &b, shingle_size as usize)
}

/// SimHash similarity with default shingle size (4).
#[napi(js_name = "simhash_similarity_default")]
pub fn simhash_similarity_default(a: String, b: String) -> f64 {
    crate::algo::simhash::compare_default(&a, &b)
}

// ─── BM25 ─────────────────────────────────────────────────────────────────

/// BM25 similarity between two strings.
#[napi(js_name = "bm25_similarity")]
pub fn bm25_similarity(a: String, b: String) -> f64 {
    crate::algo::bm25::similarity(&a, &b)
}

// ─── TF-IDF ───────────────────────────────────────────────────────────────

/// TF-IDF + Cosine similarity between two strings.
#[napi(js_name = "tfidf_similarity")]
pub fn tfidf_similarity(a: String, b: String) -> f64 {
    crate::algo::tfidf::similarity(&a, &b)
}

// ─── Preprocessor ─────────────────────────────────────────────────────────

/// Convenience: preprocess with defaults.
#[napi(js_name = "clean_text")]
pub fn clean_text(text: String) -> String {
    crate::preprocess::clean(&text)
}

/// Convenience: preprocess and remove stopwords.
#[napi(js_name = "clean_text_stopwords")]
pub fn clean_text_stopwords(text: String) -> String {
    crate::preprocess::clean_with_stopwords(&text)
}
