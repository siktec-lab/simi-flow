//! ## The "LLM Bouncer" Routing Pipeline
//!
//! Developers should not have to manually write fallback logic. The pipeline
//! allows defining thresholds to avoid expensive API calls.
//!
//! ### Execution Flow
//!
//! 1. Tier 1 (Fast Pass): Run a fast algorithm like Levenshtein.
//!    If it is an obvious match (> 0.95) or mismatch (< 0.10), return
//!    immediately for zero cost.
//! 2. Tier 2 (Heavy Local Pass): If it falls in the ambiguous middle,
//!    fallback to a heavier algorithm like BM25.
//! 3. Tier 3 (API Hook): Provide an optional callback where the user
//!    can trigger their expensive LLM API only if Tiers 1 and 2 fail.

use crate::algo;
use crate::preprocess::Preprocessor;
use crate::SimiError;

/// A normalized similarity score from `[0.0, 1.0]`.
pub type Score = f64;

/// The comparison strategy for the router.
#[derive(Clone, Debug, PartialEq)]
pub enum Strategy {
    /// Cascade: try Tier 1, then Tier 2, then Tier 3 (fallback).
    Cascade,
}

/// Threshold configuration for a tier.
#[derive(Clone, Debug)]
pub enum Threshold {
    /// Return if score is greater than this value.
    /// Matches: if score > value, it's a match.
    GreaterThan(f64),
    /// Return if score is less than this value.
    /// Mismatch: if score < value, it's clearly not a match.
    LessThan(f64),
    /// Return if score falls in this inclusive range.
    Between(f64, f64),
}

/// Algorithm selector for a tier.
#[derive(Clone, Debug)]
pub enum Algo {
    Levenshtein,
    JaroWinkler,
    Hamming,
    Jaccard(usize), // n-gram size
    JaccardBigram,
    JaccardTrigram,
    JaccardWord,
    MinHash(usize, usize), // shingle_size, num_hashes
    MinHashDefault,
    SimHash(usize), // shingle_size
    SimHashDefault,
    Bm25,
    TfIdf,
}

/// The result of comparing two strings through the pipeline.
#[derive(Clone, Debug)]
pub struct ComparisonResult {
    /// Final similarity score.
    pub score: Score,
    /// Which tier produced the result.
    pub tier: usize,
    /// The algorithm used at the decision tier.
    pub algorithm: String,
    /// Whether the fallback/API hook was called.
    pub fallback_called: bool,
    /// Optional user-defined metadata from the fallback.
    pub fallback_data: Option<String>,
}

/// Callback type for the Tier 3 (LLM/API) fallback.
pub type FallbackFn = Box<dyn Fn(&str, &str) -> (Score, Option<String>) + Send + Sync>;

/// The SimBouncer: a declarative pipeline builder for similarity checks.
///
/// ```rust
/// use simi::router::{SimBouncer, Strategy, Threshold, Algo};
///
/// let result = SimBouncer::new()
///     .preprocess(true)
///     .strategy(Strategy::Cascade)
///     .tier_1(Algo::JaroWinkler, Threshold::GreaterThan(0.95), Threshold::LessThan(0.10))
///     .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
///     .compare("hello world", "hello there");
/// ```
pub struct SimBouncer {
    strategy: Strategy,
    preprocessor: Option<Preprocessor>,
    tier_1: Option<(Algo, Threshold, Threshold)>, // (algo, match_threshold, mismatch_threshold)
    tier_2: Option<(Algo, Threshold)>,
    fallback: Option<FallbackFn>,
}

impl Default for SimBouncer {
    fn default() -> Self {
        Self {
            strategy: Strategy::Cascade,
            preprocessor: Some(Preprocessor::default()),
            tier_1: None,
            tier_2: None,
            fallback: None,
        }
    }
}

impl SimBouncer {
    /// Create a new `SimBouncer` with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable preprocessing.
    #[inline]
    pub fn preprocess(mut self, enable: bool) -> Self {
        self.preprocessor = if enable {
            Some(Preprocessor::default())
        } else {
            None
        };
        self
    }

    /// Set a custom preprocessor.
    #[inline]
    pub fn with_preprocessor(mut self, pre: Preprocessor) -> Self {
        self.preprocessor = Some(pre);
        self
    }

    /// Set the comparison strategy.
    #[inline]
    pub fn strategy(mut self, s: Strategy) -> Self {
        self.strategy = s;
        self
    }

    /// Configure Tier 1 (Fast Pass) with algorithm and thresholds.
    ///
    /// `match_threshold`: if score > this, return immediately as match.
    /// `mismatch_threshold`: if score < this, return immediately as mismatch.
    #[inline]
    pub fn tier_1(
        mut self,
        algo: Algo,
        match_threshold: Threshold,
        mismatch_threshold: Threshold,
    ) -> Self {
        self.tier_1 = Some((algo, match_threshold, mismatch_threshold));
        self
    }

    /// Configure Tier 2 (Heavy Local Pass) with algorithm and threshold range.
    #[inline]
    pub fn tier_2(mut self, algo: Algo, threshold: Threshold) -> Self {
        self.tier_2 = Some((algo, threshold));
        self
    }

    /// Set the Tier 3 fallback (LLM/API hook).
    #[inline]
    pub fn fallback<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> (Score, Option<String>) + Send + Sync + 'static,
    {
        self.fallback = Some(Box::new(f));
        self
    }

    /// Compare two strings through the pipeline.
    ///
    /// Returns a `ComparisonResult` with the final score and decision metadata.
    #[inline]
    pub fn compare(&self, a: &str, b: &str) -> Result<ComparisonResult, SimiError> {
        let a = if let Some(ref pre) = self.preprocessor {
            pre.process(a)
        } else {
            a.to_string()
        };

        let b = if let Some(ref pre) = self.preprocessor {
            pre.process(b)
        } else {
            b.to_string()
        };

        match self.strategy {
            Strategy::Cascade => self.run_cascade(&a, &b),
        }
    }

    /// Run the cascade strategy: Tier 1 -> Tier 2 -> Tier 3 (fallback).
    fn run_cascade(&self, a: &str, b: &str) -> Result<ComparisonResult, SimiError> {
        // Tier 1
        if let Some((ref algo, ref match_thresh, ref mismatch_thresh)) = self.tier_1 {
            let (score, algo_name) = run_algorithm(algo, a, b)?;

            // Check for decisive match (above match threshold)
            if let Threshold::GreaterThan(t) = match_thresh {
                if score > *t {
                    return Ok(ComparisonResult {
                        score,
                        tier: 1,
                        algorithm: algo_name,
                        fallback_called: false,
                        fallback_data: None,
                    });
                }
            }

            // Check for decisive mismatch (below mismatch threshold)
            if let Threshold::LessThan(t) = mismatch_thresh {
                if score < *t {
                    return Ok(ComparisonResult {
                        score,
                        tier: 1,
                        algorithm: algo_name,
                        fallback_called: false,
                        fallback_data: None,
                    });
                }
            }
        }

        // Tier 2
        if let Some((ref algo, ref threshold)) = self.tier_2 {
            let (score, algo_name) = run_algorithm(algo, a, b)?;

            let in_range = match threshold {
                Threshold::Between(lo, hi) => score >= *lo && score <= *hi,
                Threshold::GreaterThan(t) => score > *t,
                Threshold::LessThan(t) => score < *t,
            };

            if in_range {
                return Ok(ComparisonResult {
                    score,
                    tier: 2,
                    algorithm: algo_name,
                    fallback_called: false,
                    fallback_data: None,
                });
            }
        }

        // Tier 3 (Fallback)
        if let Some(ref fallback) = self.fallback {
            let (score, data) = fallback(a, b);
            return Ok(ComparisonResult {
                score,
                tier: 3,
                algorithm: "fallback".to_string(),
                fallback_called: true,
                fallback_data: data,
            });
        }

        // No fallback configured; Tier 1 result is the best we have
        if let Some((ref algo, _, _)) = self.tier_1 {
            let (score, algo_name) = run_algorithm(algo, a, b)?;
            return Ok(ComparisonResult {
                score,
                tier: 1,
                algorithm: algo_name,
                fallback_called: false,
                fallback_data: None,
            });
        }

        Err(SimiError::RouterError("No tiers configured".to_string()))
    }
}

/// Run an algorithm and return (score, name).
fn run_algorithm(algo: &Algo, a: &str, b: &str) -> Result<(f64, String), SimiError> {
    match algo {
        Algo::Levenshtein => Ok((algo::levenshtein::similarity(a, b), "levenshtein".into())),
        Algo::JaroWinkler => Ok((algo::jaro_winkler::similarity(a, b), "jaro_winkler".into())),
        Algo::Hamming => algo::hamming::similarity(a, b)
            .map(|s| (s, "hamming".into()))
            .ok_or_else(|| {
                SimiError::AlgorithmError("Hamming requires equal-length strings".into())
            }),
        Algo::Jaccard(n) => Ok((algo::jaccard::similarity(a, b, *n), "jaccard".into())),
        Algo::JaccardBigram => Ok((
            algo::jaccard::bigram_similarity(a, b),
            "jaccard_bigram".into(),
        )),
        Algo::JaccardTrigram => Ok((
            algo::jaccard::trigram_similarity(a, b),
            "jaccard_trigram".into(),
        )),
        Algo::JaccardWord => Ok((algo::jaccard::word_similarity(a, b), "jaccard_word".into())),
        Algo::MinHash(shingle_size, num_hashes) => {
            let s = algo::minhash::compare(a, b, *shingle_size, *num_hashes);
            Ok((s, "minhash".into()))
        }
        Algo::MinHashDefault => {
            let s = algo::minhash::compare_default(a, b);
            Ok((s, "minhash".into()))
        }
        Algo::SimHash(shingle_size) => {
            let s = algo::simhash::compare(a, b, *shingle_size);
            Ok((s, "simhash".into()))
        }
        Algo::SimHashDefault => {
            let s = algo::simhash::compare_default(a, b);
            Ok((s, "simhash".into()))
        }
        Algo::Bm25 => Ok((algo::bm25::similarity(a, b), "bm25".into())),
        Algo::TfIdf => Ok((algo::tfidf::similarity(a, b), "tfidf".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_comparison() {
        let result = SimBouncer::new()
            .tier_1(
                Algo::JaroWinkler,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .compare("MARTHA", "MARHTA")
            .unwrap();
        // Jaro-Winkler for MARTHA/MARHTA is ~0.96, so it passes Tier 1
        assert_eq!(result.tier, 1);
        assert!((result.score - 0.961).abs() < 0.01);
    }

    #[test]
    fn tier_2_fallback() {
        let result = SimBouncer::new()
            .tier_1(
                Algo::Levenshtein,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.95))
            .compare("the quick brown fox", "the quick blue fox")
            .unwrap();
        // Not a close enough match for Levenshtein Tier 1 (>0.95)
        // But BM25 should land in the 0.30-0.95 range
        assert_eq!(result.tier, 2);
        assert_eq!(result.algorithm, "bm25");
        assert!(result.score > 0.3 && result.score < 0.95);
    }

    #[test]
    fn fallback_called() {
        let result = SimBouncer::new()
            .tier_1(
                Algo::Levenshtein,
                Threshold::GreaterThan(0.99),
                Threshold::LessThan(0.01),
            )
            .fallback(|a, b| {
                // Simulate an LLM API call
                let score = if a == b { 1.0 } else { 0.5 };
                (score, Some("llm_verified".to_string()))
            })
            .compare("hello", "world")
            .unwrap();
        assert_eq!(result.tier, 3);
        assert!(result.fallback_called);
        assert_eq!(result.fallback_data, Some("llm_verified".to_string()));
    }

    #[test]
    fn tier_1_mismatch() {
        // Comparing "abc" and "xyz" with Levenshtein should give ~0.0
        // which is below the mismatch threshold of 0.10
        let result = SimBouncer::new()
            .tier_1(
                Algo::Levenshtein,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .compare("abc", "xyz")
            .unwrap();
        assert_eq!(result.tier, 1);
        assert!(result.score < 0.01);
    }

    #[test]
    fn with_preprocessing() {
        let result = SimBouncer::new()
            .preprocess(true)
            .tier_1(
                Algo::Levenshtein,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .compare("  Hello   World  ", "hello world")
            .unwrap();
        // After preprocessing, both become "hello world" → 1.0
        assert!((result.score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn no_fallback_returns_tier_1() {
        let result = SimBouncer::new()
            .tier_1(
                Algo::Levenshtein,
                Threshold::GreaterThan(0.99),
                Threshold::LessThan(0.01),
            )
            .compare("kitten", "sitting")
            .unwrap();
        // Levenshtein ~0.615: not above 0.99 or below 0.01
        // No Tier 2, no fallback; should still return Tier 1 result
        assert!(result.tier == 1 || result.tier == 2); // Will be tier 1 since it returns the best attempt
    }

    #[test]
    fn builder_pattern() {
        let bouncer = SimBouncer::new()
            .strategy(Strategy::Cascade)
            .tier_1(
                Algo::JaroWinkler,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.95));

        let result = bouncer.compare("hello world", "hello world").unwrap();
        assert!((result.score - 1.0).abs() < 0.01);
        assert_eq!(result.tier, 1);
    }
}
