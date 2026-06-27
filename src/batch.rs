//! Batch processing: uses `rayon` to evaluate arrays of thousands of
//! strings in parallel across all CPU cores.

use rayon::prelude::*;

use crate::algo;
use crate::router::{resolve_intent, Algo, Intent};
use crate::SimiError;

/// A batch comparison result.
#[derive(Clone, Debug)]
pub struct BatchResult {
    /// Index of the first string in the input pair.
    pub index_a: usize,
    /// Index of the second string in the input pair.
    pub index_b: usize,
    /// The computed similarity score.
    pub score: f64,
}

/// A batch comparator that evaluates many string pairs in parallel.
#[derive(Clone, Debug)]
pub struct BatchComparator {
    algorithm: Algo,
}

impl BatchComparator {
    /// Create a new batch comparator with the specified algorithm.
    #[inline]
    pub fn new(algorithm: Algo) -> Self {
        Self { algorithm }
    }

    /// Create a batch comparator pre-configured for a specific intent.
    ///
    /// The intent is resolved at construction time (empty strings are used
    /// for the length heuristic). For `Intent::Auto`, the caller should
    /// prefer `BatchComparator::auto_per_pair()` or use `new(Algo::*)`.
    ///
    /// ```rust
    /// use simi::batch::BatchComparator;
    /// use simi::router::Intent;
    ///
    /// let docs = vec!["doc a".to_string(), "doc b".to_string()];
    /// let cmp = BatchComparator::for_intent(Intent::Deduplication);
    /// let results = cmp.compare_matrix(&docs, &docs).unwrap();
    /// ```
    #[inline]
    pub fn for_intent(intent: Intent) -> Self {
        let algo = resolve_intent(intent, "", "");
        Self { algorithm: algo }
    }

    /// Create a batch comparator that auto-detects the best algorithm.
    ///
    /// This uses the empty-string heuristic for construction. For
    /// per-pair auto-detection, use `SimiFlow::compare_with_intent()`
    /// or construct a new `BatchComparator` per pair via the length
    /// heuristic.
    #[inline]
    pub fn auto() -> Self {
        Self::for_intent(Intent::Auto)
    }

    /// Compare two arrays element-wise in parallel.
    ///
    /// Each pair `(A[i], B[i])` is compared using the configured algorithm.
    /// The two slices must have the same length.
    #[inline]
    pub fn compare_pairs(&self, a: &[String], b: &[String]) -> Result<Vec<BatchResult>, SimiError> {
        if a.len() != b.len() {
            return Err(SimiError::BatchError(
                "Input slices must have the same length".into(),
            ));
        }

        let results: Vec<BatchResult> = a
            .par_iter()
            .zip(b.par_iter())
            .enumerate()
            .map(|(i, (sa, sb))| {
                let score = match &self.algorithm {
                    Algo::Levenshtein => algo::levenshtein::similarity(sa, sb),
                    Algo::JaroWinkler => algo::jaro_winkler::similarity(sa, sb),
                    Algo::Hamming => algo::hamming::similarity(sa, sb).unwrap_or(0.0),
                    Algo::Jaccard(n) => algo::jaccard::similarity(sa, sb, *n),
                    Algo::JaccardBigram => algo::jaccard::bigram_similarity(sa, sb),
                    Algo::JaccardTrigram => algo::jaccard::trigram_similarity(sa, sb),
                    Algo::JaccardWord => algo::jaccard::word_similarity(sa, sb),
                    Algo::MinHash(sh, nh) => algo::minhash::compare(sa, sb, *sh, *nh),
                    Algo::MinHashDefault => algo::minhash::compare_default(sa, sb),
                    Algo::SimHash(sh) => algo::simhash::compare(sa, sb, *sh),
                    Algo::SimHashDefault => algo::simhash::compare_default(sa, sb),
                    Algo::Bm25 => algo::bm25::similarity(sa, sb),
                    Algo::TfIdf => algo::tfidf::similarity(sa, sb),
                };
                BatchResult {
                    index_a: i,
                    index_b: i,
                    score,
                }
            })
            .collect();

        Ok(results)
    }

    /// Compare one string against many candidates in parallel.
    ///
    /// Returns scores for `(reference, candidates[0..n])`.
    #[inline]
    pub fn compare_one_to_many(
        &self,
        reference: &str,
        candidates: &[String],
    ) -> Result<Vec<BatchResult>, SimiError> {
        let results: Vec<BatchResult> = candidates
            .par_iter()
            .enumerate()
            .map(|(i, candidate)| {
                let score = match &self.algorithm {
                    Algo::Levenshtein => algo::levenshtein::similarity(reference, candidate),
                    Algo::JaroWinkler => algo::jaro_winkler::similarity(reference, candidate),
                    Algo::Hamming => algo::hamming::similarity(reference, candidate).unwrap_or(0.0),
                    Algo::Jaccard(n) => algo::jaccard::similarity(reference, candidate, *n),
                    Algo::JaccardBigram => algo::jaccard::bigram_similarity(reference, candidate),
                    Algo::JaccardTrigram => algo::jaccard::trigram_similarity(reference, candidate),
                    Algo::JaccardWord => algo::jaccard::word_similarity(reference, candidate),
                    Algo::MinHash(sh, nh) => algo::minhash::compare(reference, candidate, *sh, *nh),
                    Algo::MinHashDefault => algo::minhash::compare_default(reference, candidate),
                    Algo::SimHash(sh) => algo::simhash::compare(reference, candidate, *sh),
                    Algo::SimHashDefault => algo::simhash::compare_default(reference, candidate),
                    Algo::Bm25 => algo::bm25::similarity(reference, candidate),
                    Algo::TfIdf => algo::tfidf::similarity(reference, candidate),
                };
                BatchResult {
                    index_a: 0,
                    index_b: i,
                    score,
                }
            })
            .collect();

        Ok(results)
    }

    /// Compare all pairs in a cross-product (matrix) in parallel.
    ///
    /// Returns `n * m` results comparing every element in `a` against
    /// every element in `b`.
    #[inline]
    pub fn compare_matrix(
        &self,
        a: &[String],
        b: &[String],
    ) -> Result<Vec<BatchResult>, SimiError> {
        let results: Vec<BatchResult> = a
            .par_iter()
            .enumerate()
            .flat_map(|(i, sa)| {
                b.par_iter()
                    .enumerate()
                    .map(|(j, sb)| {
                        let score = match &self.algorithm {
                            Algo::Levenshtein => algo::levenshtein::similarity(sa, sb),
                            Algo::JaroWinkler => algo::jaro_winkler::similarity(sa, sb),
                            Algo::Hamming => algo::hamming::similarity(sa, sb).unwrap_or(0.0),
                            Algo::Jaccard(n) => algo::jaccard::similarity(sa, sb, *n),
                            Algo::JaccardBigram => algo::jaccard::bigram_similarity(sa, sb),
                            Algo::JaccardTrigram => algo::jaccard::trigram_similarity(sa, sb),
                            Algo::JaccardWord => algo::jaccard::word_similarity(sa, sb),
                            Algo::MinHash(sh, nh) => algo::minhash::compare(sa, sb, *sh, *nh),
                            Algo::MinHashDefault => algo::minhash::compare_default(sa, sb),
                            Algo::SimHash(sh) => algo::simhash::compare(sa, sb, *sh),
                            Algo::SimHashDefault => algo::simhash::compare_default(sa, sb),
                            Algo::Bm25 => algo::bm25::similarity(sa, sb),
                            Algo::TfIdf => algo::tfidf::similarity(sa, sb),
                        };
                        BatchResult {
                            index_a: i,
                            index_b: j,
                            score,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_pairs_basic() {
        let a = vec!["hello".into(), "world".into(), "rust".into()];
        let b = vec!["hello".into(), "word".into(), "rusty".into()];

        let comparator = BatchComparator::new(Algo::Levenshtein);
        let results = comparator.compare_pairs(&a, &b).unwrap();

        assert_eq!(results.len(), 3);
        // First pair: "hello" == "hello" → 1.0
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
        // Scores are normalized
        assert!(results[0].score >= 0.0 && results[0].score <= 1.0);
        assert!(results[1].score >= 0.0 && results[1].score <= 1.0);
        assert!(results[2].score >= 0.0 && results[2].score <= 1.0);
    }

    #[test]
    fn compare_one_to_many() {
        let reference = "hello".to_string();
        let candidates = vec!["hello".into(), "hallo".into(), "world".into()];

        let comparator = BatchComparator::new(Algo::Levenshtein);
        let results = comparator
            .compare_one_to_many(&reference, &candidates)
            .unwrap();

        assert_eq!(results.len(), 3);
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compare_matrix() {
        let a = vec!["hello".into(), "world".into()];
        let b = vec!["hello".into(), "word".into()];

        let comparator = BatchComparator::new(Algo::Levenshtein);
        let results = comparator.compare_matrix(&a, &b).unwrap();

        // 2 x 2 = 4 results
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn unequal_lengths_error() {
        let a = vec!["hello".into()];
        let b = vec!["hello".into(), "world".into()];

        let comparator = BatchComparator::new(Algo::Levenshtein);
        let result = comparator.compare_pairs(&a, &b);
        assert!(result.is_err());
    }

    #[test]
    fn large_batch() {
        let size = 100;
        let a: Vec<String> = (0..size).map(|i| format!("string {}", i)).collect();
        let b: Vec<String> = (0..size).map(|i| format!("string {}", i + 1)).collect();

        let comparator = BatchComparator::new(Algo::Levenshtein);
        let results = comparator.compare_pairs(&a, &b).unwrap();

        assert_eq!(results.len(), size);
        // All scores should be valid
        for r in &results {
            assert!(r.score >= 0.0 && r.score <= 1.0);
        }
    }
}
