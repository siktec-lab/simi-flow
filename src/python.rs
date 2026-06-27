//! Python bindings for SIMI (via PyO3 + maturin).

use pyo3::prelude::*;

#[allow(dead_code)]
fn to_py_err(e: crate::SimiError) -> pyo3::PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}

/// Compute Levenshtein similarity between two strings.
#[pyfunction]
fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    crate::algo::levenshtein::similarity(a, b)
}

/// Compute Jaro-Winkler similarity between two strings.
#[pyfunction]
fn jaro_winkler_similarity(a: &str, b: &str) -> f64 {
    crate::algo::jaro_winkler::similarity(a, b)
}

/// Compute Hamming similarity between two strings (must be equal length).
#[pyfunction]
fn hamming_similarity(a: &str, b: &str) -> PyResult<f64> {
    crate::algo::hamming::similarity(a, b)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Strings must have equal length"))
}

/// Compute Jaccard similarity between two strings using n-grams.
#[pyfunction]
fn jaccard_similarity(a: &str, b: &str, n: usize) -> f64 {
    crate::algo::jaccard::similarity(a, b, n)
}

/// Compute BM25 similarity between two strings.
#[pyfunction]
fn bm25_similarity(a: &str, b: &str) -> f64 {
    crate::algo::bm25::similarity(a, b)
}

/// Compute TF-IDF + Cosine similarity between two strings.
#[pyfunction]
fn tfidf_similarity(a: &str, b: &str) -> f64 {
    crate::algo::tfidf::similarity(a, b)
}

/// Preprocess a string (normalize, lowercase, collapse whitespace).
#[pyfunction]
fn clean_text(text: &str) -> String {
    crate::preprocess::clean(text)
}

// ─── Module initialization ─────────────────────────────────────────────────

#[pymodule]
fn simi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaro_winkler_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(hamming_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaccard_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(bm25_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(tfidf_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(clean_text, m)?)?;
    Ok(())
}
