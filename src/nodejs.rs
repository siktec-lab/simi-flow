//! JavaScript/Node.js bindings for SIMI (via napi-rs).

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[allow(dead_code)]
fn to_napi_err(e: crate::SimiError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

/// Compute Levenshtein similarity between two strings.
#[napi]
pub fn levenshtein_similarity(a: String, b: String) -> f64 {
    crate::algo::levenshtein::similarity(&a, &b)
}

/// Compute Jaro-Winkler similarity between two strings.
#[napi]
pub fn jaro_winkler_similarity(a: String, b: String) -> f64 {
    crate::algo::jaro_winkler::similarity(&a, &b)
}

/// Compute Hamming similarity between two strings (must be equal length).
#[napi]
pub fn hamming_similarity(a: String, b: String) -> Result<f64> {
    crate::algo::hamming::similarity(&a, &b)
        .ok_or_else(|| napi::Error::from_reason("Strings must have equal length"))
}

/// Compute Jaccard similarity between two strings using n-grams.
#[napi]
pub fn jaccard_similarity(a: String, b: String, n: i32) -> f64 {
    crate::algo::jaccard::similarity(&a, &b, n as usize)
}

/// Compute BM25 similarity between two strings.
#[napi]
pub fn bm25_similarity(a: String, b: String) -> f64 {
    crate::algo::bm25::similarity(&a, &b)
}

/// Compute TF-IDF + Cosine similarity between two strings.
#[napi]
pub fn tfidf_similarity(a: String, b: String) -> f64 {
    crate::algo::tfidf::similarity(&a, &b)
}

/// Preprocess a string (normalize, lowercase, collapse whitespace).
#[napi]
pub fn clean_text(text: String) -> String {
    crate::preprocess::clean(&text)
}
