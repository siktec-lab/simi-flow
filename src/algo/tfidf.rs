//! TF-IDF + Cosine — counts word frequencies, weights them by rarity, and
//! calculates the cosine angle between document vectors.
//!
//! $\cos(\theta) = \frac{A \cdot B}{|A||B|}$
//!
//! ## Performance
//! O(n·m) time where n,m = unique terms in each document.

use std::collections::{HashMap, HashSet};

/// A TF-IDF vector representing a document.
#[derive(Clone, Debug)]
pub struct TfIdfVector {
    /// Term -> (TF-IDF weight) map.
    pub weights: HashMap<String, f64>,
    /// Pre-computed magnitude for fast cosine.
    pub magnitude: f64,
}

/// Build a TF-IDF vector for a document given corpus-wide IDF weights.
///
/// # Arguments
/// * `tokens` - Tokenized document (individual word tokens).
/// * `idf_weights` - Pre-computed IDF weights from the corpus.
///
/// Returns the TF-IDF vector with pre-computed magnitude.
#[inline]
pub fn build_vector(tokens: &[String], idf_weights: &HashMap<String, f64>) -> TfIdfVector {
    // Compute term frequencies
    let mut tf: HashMap<String, f64> = HashMap::new();
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0.0) += 1.0;
    }

    // Normalize TF by document length
    let doc_len = tokens.len() as f64;
    let mut weights = HashMap::new();
    let mut magnitude_sq = 0.0;

    for (term, count) in &tf {
        let tf_val = count / doc_len;
        let idf = idf_weights.get(term).copied().unwrap_or(0.0);
        let w = tf_val * idf;
        magnitude_sq += w * w;
        weights.insert(term.clone(), w);
    }

    TfIdfVector {
        weights,
        magnitude: magnitude_sq.sqrt(),
    }
}

/// Compute IDF weights from a corpus of tokenized documents.
///
/// `idf(t) = ln((1 + N) / (1 + df(t))) + 1`
/// where N = total documents, df(t) = documents containing term t.
#[inline]
pub fn compute_idf(documents: &[Vec<String>]) -> HashMap<String, f64> {
    let n = documents.len() as f64;
    let mut df: HashMap<String, usize> = HashMap::new();

    for doc in documents {
        let unique_terms: HashSet<&String> = doc.iter().collect();
        for term in unique_terms {
            *df.entry(term.clone()).or_insert(0) += 1;
        }
    }

    let mut idf = HashMap::new();
    for (term, count) in &df {
        let idf_val = ((1.0 + n) / (1.0 + *count as f64)).ln() + 1.0;
        idf.insert(term.clone(), idf_val);
    }

    idf
}

/// Cosine similarity between two TF-IDF vectors.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` means identical term vectors.
#[inline]
pub fn cosine_similarity(a: &TfIdfVector, b: &TfIdfVector) -> f64 {
    if a.magnitude == 0.0 || b.magnitude == 0.0 {
        return if a.weights.is_empty() && b.weights.is_empty() {
            1.0
        } else {
            0.0
        };
    }

    // Dot product of overlapping terms
    let dot_product: f64 = a
        .weights
        .iter()
        .filter_map(|(term, w_a)| b.weights.get(term).map(|w_b| w_a * w_b))
        .sum();

    dot_product / (a.magnitude * b.magnitude)
}

/// Quick one-shot TF-IDF + Cosine similarity between two texts.
///
/// Tokenizes both, builds a mini-corpus of the two documents, computes
/// IDF, builds vectors, and returns cosine similarity.
#[inline]
pub fn similarity(a: &str, b: &str) -> f64 {
    let tokens_a = tokenize(a);
    let tokens_b = tokenize(b);
    let documents = vec![tokens_a, tokens_b];
    let idf = compute_idf(&documents);
    let vec_a = build_vector(&documents[0], &idf);
    let vec_b = build_vector(&documents[1], &idf);
    // `+ 0.0` collapses a possible IEEE-754 negative zero to positive zero so
    // the public API never returns `-0.0` (which trips strict equality checks).
    cosine_similarity(&vec_a, &vec_b) + 0.0
}

/// Tokenize text into lowercase words.
#[inline]
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_texts() {
        let s = similarity("the quick brown fox", "the quick brown fox");
        assert!((s - 1.0).abs() < 0.01, "identical should be ~1.0, got {s}");
    }

    #[test]
    fn completely_different() {
        let s = similarity("hello world", "completely unrelated");
        assert!(s < 0.3, "expected low similarity, got {s}");
    }

    #[test]
    fn partial_match() {
        let s = similarity("the quick brown fox", "the quick blue fox");
        assert!(s > 0.3 && s < 1.0, "expected moderate, got {s}");
    }

    #[test]
    fn empty_strings() {
        // Both empty → both vectors empty → cosine returns 1.0
        let s = similarity("", "");
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn symmetric() {
        let a = similarity("the cat sat on the mat", "the dog sat on the rug");
        let b = similarity("the dog sat on the rug", "the cat sat on the mat");
        assert!((a - b).abs() < f64::EPSILON);
    }
}
