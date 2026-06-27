//! Python bindings for SIMI (via PyO3 + maturin).
//!
//! Exposes every algorithm, the preprocessor, and the SimiFlow router.

use pyo3::prelude::*;
use pyo3::types::PyDict;

// ─── Error helpers ────────────────────────────────────────────────────────

fn to_py_err(e: crate::SimiError) -> pyo3::PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}

fn build_result_dict(result: crate::router::ComparisonResult) -> PyResult<Py<PyDict>> {
    Python::try_attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("score", result.score)?;
        dict.set_item("tier", result.tier)?;
        dict.set_item("algorithm", result.algorithm.as_str())?;
        dict.set_item("fallback_called", result.fallback_called)?;
        dict.set_item("fallback_data", result.fallback_data.as_deref().unwrap_or(""))?;
        Ok::<_, PyErr>(dict.into())
    })
    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("No Python interpreter active"))?
}

// ─── Algorithm functions ──────────────────────────────────────────────────

#[pyfunction]
fn levenshtein_distance(a: &str, b: &str) -> usize {
    crate::algo::levenshtein::distance(a, b)
}

#[pyfunction]
fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    crate::algo::levenshtein::similarity(a, b)
}

#[pyfunction]
fn jaro_winkler_similarity(a: &str, b: &str) -> f64 {
    crate::algo::jaro_winkler::similarity(a, b)
}

#[pyfunction]
fn hamming_similarity(a: &str, b: &str) -> PyResult<f64> {
    crate::algo::hamming::similarity(a, b)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Strings must have equal length"))
}

#[pyfunction]
fn hamming_distance(a: &str, b: &str) -> PyResult<usize> {
    crate::algo::hamming::distance(a, b)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Strings must have equal length"))
}

#[pyfunction]
fn jaccard_similarity(a: &str, b: &str, n: usize) -> f64 {
    crate::algo::jaccard::similarity(a, b, n)
}

#[pyfunction]
fn jaccard_bigram_similarity(a: &str, b: &str) -> f64 {
    crate::algo::jaccard::bigram_similarity(a, b)
}

#[pyfunction]
fn jaccard_trigram_similarity(a: &str, b: &str) -> f64 {
    crate::algo::jaccard::trigram_similarity(a, b)
}

#[pyfunction]
fn jaccard_word_similarity(a: &str, b: &str) -> f64 {
    crate::algo::jaccard::word_similarity(a, b)
}

#[pyfunction]
fn minhash_signature(text: &str, shingle_size: usize, num_hashes: usize) -> Vec<u64> {
    crate::algo::minhash::signature(text, shingle_size, num_hashes).signatures
}

#[pyfunction]
fn minhash_similarity(a: &str, b: &str, shingle_size: usize, num_hashes: usize) -> f64 {
    crate::algo::minhash::compare(a, b, shingle_size, num_hashes)
}

#[pyfunction]
fn minhash_similarity_default(a: &str, b: &str) -> f64 {
    crate::algo::minhash::compare_default(a, b)
}

#[pyfunction]
fn simhash_fingerprint(text: &str, shingle_size: usize) -> u64 {
    crate::algo::simhash::fingerprint(text, shingle_size).0
}

#[pyfunction]
fn simhash_fingerprint_default(text: &str) -> u64 {
    crate::algo::simhash::fingerprint_default(text).0
}

#[pyfunction]
fn simhash_similarity(a: &str, b: &str, shingle_size: usize) -> f64 {
    crate::algo::simhash::compare(a, b, shingle_size)
}

#[pyfunction]
fn simhash_similarity_default(a: &str, b: &str) -> f64 {
    crate::algo::simhash::compare_default(a, b)
}

#[pyfunction]
fn bm25_similarity(a: &str, b: &str) -> f64 {
    crate::algo::bm25::similarity(a, b)
}

#[pyfunction]
fn tfidf_similarity(a: &str, b: &str) -> f64 {
    crate::algo::tfidf::similarity(a, b)
}

// ─── Preprocessor ─────────────────────────────────────────────────────────

#[pyclass(name = "Preprocessor", from_py_object)]
#[derive(Clone)]
struct PyPreprocessor {
    inner: crate::preprocess::Preprocessor,
}

#[pymethods]
impl PyPreprocessor {
    #[new]
    fn new() -> Self {
        Self { inner: crate::preprocess::Preprocessor::default() }
    }

    fn with_lowercase(&self, v: bool) -> Self {
        Self { inner: self.inner.clone().with_lowercase(v) }
    }
    fn with_collapse_whitespace(&self, v: bool) -> Self {
        Self { inner: self.inner.clone().with_collapse_whitespace(v) }
    }
    fn with_trim(&self, v: bool) -> Self {
        Self { inner: self.inner.clone().with_trim(v) }
    }
    fn with_normalize_unicode(&self, v: bool) -> Self {
        Self { inner: self.inner.clone().with_normalize_unicode(v) }
    }
    fn with_remove_stopwords(&self, v: bool) -> Self {
        Self { inner: self.inner.clone().with_remove_stopwords(v) }
    }
    fn with_stopwords(&self, words: Vec<String>) -> Self {
        Self { inner: self.inner.clone().with_stopwords(words) }
    }
    fn with_max_length(&self, max: usize) -> Self {
        Self { inner: self.inner.clone().with_max_length(max) }
    }
    fn process(&self, text: &str) -> String {
        self.inner.process(text)
    }
    fn __repr__(&self) -> String {
        format!("Preprocessor(lowercase={}, stopwords={})",
            self.inner.to_lowercase, self.inner.remove_stopwords)
    }
}

#[pyfunction]
fn clean_text(text: &str) -> String {
    crate::preprocess::clean(text)
}

#[pyfunction]
fn clean_text_stopwords(text: &str) -> String {
    crate::preprocess::clean_with_stopwords(text)
}

// ─── SimiFlow router ──────────────────────────────────────────────────────

use crate::router::{Algo, SimiFlow, Threshold, Intent};

fn parse_algo(s: &str) -> PyResult<Algo> {
    match s {
        "levenshtein" => Ok(Algo::Levenshtein),
        "jaro_winkler" => Ok(Algo::JaroWinkler),
        "hamming" => Ok(Algo::Hamming),
        "jaccard_bigram" => Ok(Algo::JaccardBigram),
        "jaccard_trigram" => Ok(Algo::JaccardTrigram),
        "jaccard_word" => Ok(Algo::JaccardWord),
        "minhash_default" => Ok(Algo::MinHashDefault),
        "simhash_default" => Ok(Algo::SimHashDefault),
        "bm25" => Ok(Algo::Bm25),
        "tfidf" => Ok(Algo::TfIdf),
        other => Err(pyo3::exceptions::PyValueError::new_err(
            format!("unknown algorithm: {other}")))
    }
}

fn parse_threshold(op: &str, val: f64) -> PyResult<Threshold> {
    match op {
        "gt" | "greater_than" => Ok(Threshold::GreaterThan(val)),
        "lt" | "less_than" => Ok(Threshold::LessThan(val)),
        other => Err(pyo3::exceptions::PyValueError::new_err(
            format!("unknown threshold op: {other}")))
    }
}

fn parse_intent(s: &str) -> PyResult<Intent> {
    match s {
        "names" => Ok(Intent::Names),
        "typos" => Ok(Intent::Typos),
        "codes" => Ok(Intent::Codes),
        "documents" => Ok(Intent::Documents),
        "deduplication" | "dedup" => Ok(Intent::Deduplication),
        "auto" => Ok(Intent::Auto),
        other => Err(pyo3::exceptions::PyValueError::new_err(
            format!("unknown intent: {other}")))
    }
}

#[pyclass(name = "SimiFlow", from_py_object)]
#[derive(Clone)]
struct PySimiFlow {
    preprocess_enabled: bool,
    tier_1_algo: Option<String>,
    tier_1_match_op: Option<String>,
    tier_1_match_val: Option<f64>,
    tier_1_mismatch_op: Option<String>,
    tier_1_mismatch_val: Option<f64>,
    tier_2_algo: Option<String>,
    tier_2_op: Option<String>,
    tier_2_val1: Option<f64>,
    tier_2_val2: Option<f64>,
}

#[pymethods]
impl PySimiFlow {
    #[new]
    fn new() -> Self {
        Self {
            preprocess_enabled: false,
            tier_1_algo: None, tier_1_match_op: None, tier_1_match_val: None,
            tier_1_mismatch_op: None, tier_1_mismatch_val: None,
            tier_2_algo: None, tier_2_op: None, tier_2_val1: None, tier_2_val2: None,
        }
    }

    fn preprocess(&self, enable: bool) -> Self {
        let mut c = self.clone();
        c.preprocess_enabled = enable;
        c
    }

    fn tier_1(&self, algo: &str, match_op: &str, match_val: f64,
              mismatch_op: &str, mismatch_val: f64) -> PyResult<Self> {
        let mut c = self.clone();
        c.tier_1_algo = Some(algo.to_string());
        c.tier_1_match_op = Some(match_op.to_string());
        c.tier_1_match_val = Some(match_val);
        c.tier_1_mismatch_op = Some(mismatch_op.to_string());
        c.tier_1_mismatch_val = Some(mismatch_val);
        Ok(c)
    }

    fn tier_2(&self, algo: &str, op: &str, val1: f64, val2: f64) -> PyResult<Self> {
        let mut c = self.clone();
        c.tier_2_algo = Some(algo.to_string());
        c.tier_2_op = Some(op.to_string());
        c.tier_2_val1 = Some(val1);
        c.tier_2_val2 = Some(val2);
        Ok(c)
    }

    fn compare_with_intent(&self, intent: &str, a: &str, b: &str) -> PyResult<Py<PyDict>> {
        let flow = SimiFlow::new().preprocess(self.preprocess_enabled);
        let intent = parse_intent(intent)?;
        let result = flow.compare_with_intent(intent, a, b).map_err(to_py_err)?;
        build_result_dict(result)
    }

    fn compare(&self, a: &str, b: &str) -> PyResult<Py<PyDict>> {
        let mut flow = SimiFlow::new().preprocess(self.preprocess_enabled);
        if let (Some(algo), Some(match_op), Some(match_val), Some(mismatch_op), Some(mismatch_val)) =
            (&self.tier_1_algo, &self.tier_1_match_op, self.tier_1_match_val,
             &self.tier_1_mismatch_op, self.tier_1_mismatch_val)
        {
            let a = parse_algo(algo)?;
            flow = flow.tier_1(a, parse_threshold(match_op, match_val)?,
                               parse_threshold(mismatch_op, mismatch_val)?);
        }
        if let (Some(algo), Some(op), Some(val1), Some(val2)) =
            (&self.tier_2_algo, &self.tier_2_op, self.tier_2_val1, self.tier_2_val2)
        {
            let a = parse_algo(algo)?;
            let thresh = match op.as_str() {
                "between" => Threshold::Between(val1, val2),
                "gt" | "greater_than" => Threshold::GreaterThan(val1),
                "lt" | "less_than" => Threshold::LessThan(val1),
                other => return Err(pyo3::exceptions::PyValueError::new_err(
                    format!("unknown threshold op: {other}"))),
            };
            flow = flow.tier_2(a, thresh);
        }
        let result = flow.compare(a, b).map_err(to_py_err)?;
        build_result_dict(result)
    }

    fn __repr__(&self) -> String {
        "SimiFlow()".to_string()
    }
}

// ─── Module initialization ─────────────────────────────────────────────────

#[pymodule]
fn simi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein_distance, m)?)?;
    m.add_function(wrap_pyfunction!(levenshtein_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaro_winkler_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(hamming_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(hamming_distance, m)?)?;
    m.add_function(wrap_pyfunction!(jaccard_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaccard_bigram_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaccard_trigram_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(jaccard_word_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(minhash_signature, m)?)?;
    m.add_function(wrap_pyfunction!(minhash_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(minhash_similarity_default, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_fingerprint, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_fingerprint_default, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_similarity_default, m)?)?;
    m.add_function(wrap_pyfunction!(bm25_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(tfidf_similarity, m)?)?;
    m.add_class::<PyPreprocessor>()?;
    m.add_function(wrap_pyfunction!(clean_text, m)?)?;
    m.add_function(wrap_pyfunction!(clean_text_stopwords, m)?)?;
    m.add_class::<PySimiFlow>()?;
    Ok(())
}
