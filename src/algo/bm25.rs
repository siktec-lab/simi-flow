//! BM25 (Best Matching 25) — a probabilistic retrieval function for ranking
//! documents against a query. An improvement over TF-IDF for contextual
//! similarity without the overhead of an LLM.
//!
//! ## Performance
//! BM25 scores between query tokens and document tokens. O(q·d) for a single
//! query-document pair where q = query tokens, d = document tokens.

use std::collections::HashMap;

/// BM25 configuration parameters.
///
/// Default values (k1 = 1.2, b = 0.75) work well for most use cases.
#[derive(Clone, Debug)]
pub struct Bm25Params {
    /// Term frequency saturation parameter (default: 1.2).
    pub k1: f64,
    /// Length normalization parameter (default: 0.75).
    pub b: f64,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

/// BM25 scoring context built from a corpus.
///
/// Pre-computes document lengths, average document length, and
/// inverse document frequencies so that scoring is fast.
#[derive(Clone, Debug)]
pub struct Bm25Index {
    /// Number of documents in the corpus.
    pub num_docs: usize,
    /// Average document length (in tokens).
    pub avg_doc_len: f64,
    /// Document length for each doc (in tokens).
    pub doc_lengths: Vec<usize>,
    /// Number of documents containing each term.
    pub doc_freqs: HashMap<String, usize>,
    /// Term frequency matrix: doc_index -> {term -> count}
    pub term_freqs: Vec<HashMap<String, usize>>,
    /// BM25 parameters.
    pub params: Bm25Params,
    /// Total tokens in corpus.
    pub total_tokens: usize,
}

impl Bm25Index {
    /// Build a BM25 index from a collection of tokenized documents.
    ///
    /// Each document is a slice of individual word tokens.
    #[inline]
    pub fn build(documents: &[Vec<String>], params: Bm25Params) -> Self {
        let num_docs = documents.len();
        let mut doc_lengths = Vec::with_capacity(num_docs);
        let mut term_freqs = Vec::with_capacity(num_docs);
        let mut doc_freqs: HashMap<String, usize> = HashMap::new();
        let mut total_tokens = 0;

        for doc in documents {
            let mut tf = HashMap::new();
            let doc_len = doc.len();

            for token in doc {
                let counter = tf.entry(token.clone()).or_insert(0);
                *counter += 1;
            }

            // Track document frequencies: each unique term in this doc
            for token in tf.keys() {
                *doc_freqs.entry(token.clone()).or_insert(0) += 1;
            }

            doc_lengths.push(doc_len);
            total_tokens += doc_len;
            term_freqs.push(tf);
        }

        let avg_doc_len = if num_docs > 0 {
            total_tokens as f64 / num_docs as f64
        } else {
            0.0
        };

        Self {
            num_docs,
            avg_doc_len,
            doc_lengths,
            doc_freqs,
            term_freqs,
            params,
            total_tokens,
        }
    }

    /// Score a query against a single document by index.
    ///
    /// Query is a sequence of tokens. Returns the BM25 score.
    #[inline]
    pub fn score(&self, query: &[String], doc_index: usize) -> f64 {
        let doc_len = self.doc_lengths[doc_index] as f64;
        let tf = &self.term_freqs[doc_index];
        let mut score = 0.0;

        for term in query {
            let df = self.doc_freqs.get(term).copied().unwrap_or(0);
            if df == 0 {
                continue;
            }

            // IDF formula used in BM25
            let idf = ((self.num_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();

            let term_freq = tf.get(term).copied().unwrap_or(0) as f64;

            // BM25 score contribution
            let numerator = term_freq * (self.params.k1 + 1.0);
            let denominator = term_freq
                + self.params.k1
                    * (1.0 - self.params.b + self.params.b * doc_len / self.avg_doc_len);
            let tf_component = numerator / denominator;

            score += idf * tf_component;
        }

        score
    }

    /// Score a query against all documents.
    ///
    /// Returns a vector of (doc_index, score) pairs, unsorted.
    #[inline]
    pub fn score_all(&self, query: &[String]) -> Vec<(usize, f64)> {
        (0..self.num_docs)
            .map(|i| (i, self.score(query, i)))
            .collect()
    }

    /// Normalize BM25 scores to `[0.0, 1.0]` by dividing by the max score.
    #[inline]
    pub fn normalized_score(&self, query: &[String], doc_index: usize) -> f64 {
        let raw = self.score(query, doc_index);
        let max = self
            .score_all(query)
            .into_iter()
            .map(|(_, s)| s)
            .fold(0.0_f64, f64::max);
        if max > 0.0 {
            raw / max
        } else {
            0.0
        }
    }
}

/// Simple word-level tokenizer splitting on whitespace and punctuation.
#[inline]
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Quick one-shot BM25 similarity between two texts.
///
/// Tokenizes both texts, builds a mini-index with just the two documents,
/// and returns the normalized BM25 score of query (first text) against
/// the second document.
#[inline]
pub fn similarity(a: &str, b: &str) -> f64 {
    let tokens_a = tokenize(a);
    let tokens_b = tokenize(b);
    let docs = vec![tokens_a.clone(), tokens_b];
    let index = Bm25Index::build(&docs, Bm25Params::default());
    index.normalized_score(&tokens_a, 1)
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
        let s = similarity("", "");
        // Both empty → both token lists empty → max score is 0 → 0.0
        assert!((s - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn index_builds() {
        let docs = vec![
            tokenize("the cat sat on the mat"),
            tokenize("the dog sat on the log"),
        ];
        let index = Bm25Index::build(&docs, Bm25Params::default());
        assert_eq!(index.num_docs, 2);
        assert!(index.avg_doc_len > 0.0);
    }
}
