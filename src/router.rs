//! ## The SimiFlow Routing Pipeline
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

/// User intent: declares what kind of comparison is being performed.
///
/// The intent maps directly to the best algorithm for that data type.
/// Use `Intent::Auto` to let SIMI inspect the inputs and pick automatically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    /// Personal names, brand names, short identifiers.
    /// Maps to: Jaro-Winkler (prefix-weighted matching).
    Names,
    /// Typos, misspellings, character-level errors.
    /// Maps to: Levenshtein (edit distance).
    Typos,
    /// Equal-length codes, checksums, fixed-width identifiers.
    /// Maps to: Hamming (position-based comparison).
    Codes,
    /// Documents, paragraphs, product descriptions.
    /// Maps to: BM25 (term-weighted retrieval).
    Documents,
    /// Large-scale near-duplicate detection.
    /// Maps to: SimHash (64-bit LSH fingerprint).
    Deduplication,
    /// Inspect input lengths and pick the best algorithm automatically.
    Auto,
}

/// Algorithm selector for a tier.
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// The SimiFlow: a declarative pipeline builder for similarity checks.
///
/// ```rust
/// use simi::router::{SimiFlow, Strategy, Threshold, Algo};
///
/// let result = SimiFlow::new()
///     .preprocess(true)
///     .strategy(Strategy::Cascade)
///     .tier_1(Algo::JaroWinkler, Threshold::GreaterThan(0.95), Threshold::LessThan(0.10))
///     .tier_2(Algo::Bm25, Threshold::Between(0.60, 0.94))
///     .compare("hello world", "hello there");
/// ```
pub struct SimiFlow {
    strategy: Strategy,
    preprocessor: Option<Preprocessor>,
    tier_1: Option<(Algo, Threshold, Threshold)>, // (algo, match_threshold, mismatch_threshold)
    tier_2: Option<(Algo, Threshold)>,
    fallback: Option<FallbackFn>,
    /// If true, the algorithm is re-resolved per pair via `auto_select`.
    auto_mode: bool,
}

impl Default for SimiFlow {
    fn default() -> Self {
        Self {
            strategy: Strategy::Cascade,
            preprocessor: Some(Preprocessor::default()),
            tier_1: None,
            tier_2: None,
            fallback: None,
            auto_mode: false,
        }
    }
}

impl SimiFlow {
    /// Create a new `SimiFlow` with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `SimiFlow` pre-configured for a specific intent.
    ///
    /// The intent picks the algorithm and reasonable default thresholds.
    /// ```rust
    /// use simi::router::{SimiFlow, Intent};
    ///
    /// let result = SimiFlow::for_intent(Intent::Names)
    ///     .compare("MARTHA", "MARHTA")
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn for_intent(intent: Intent) -> Self {
        let algo = resolve_intent(intent, "", "");
        let match_thresh = match intent {
            Intent::Names | Intent::Typos => Threshold::GreaterThan(0.95),
            Intent::Codes => Threshold::GreaterThan(0.95),
            Intent::Documents => Threshold::GreaterThan(0.90),
            Intent::Deduplication => Threshold::GreaterThan(0.90),
            Intent::Auto => Threshold::GreaterThan(0.95),
        };
        let mismatch_thresh = match intent {
            Intent::Names | Intent::Typos => Threshold::LessThan(0.10),
            Intent::Codes => Threshold::LessThan(0.10),
            Intent::Documents => Threshold::LessThan(0.05),
            Intent::Deduplication => Threshold::LessThan(0.10),
            Intent::Auto => Threshold::LessThan(0.10),
        };
        Self {
            strategy: Strategy::Cascade,
            preprocessor: Some(Preprocessor::default()),
            tier_1: Some((algo, match_thresh, mismatch_thresh)),
            tier_2: None,
            fallback: None,
            auto_mode: false,
        }
    }

    /// Create a `SimiFlow` that auto-detects the best algorithm from inputs.
    ///
    /// The algorithm is re-resolved at each `compare()` call based on the
    /// actual input lengths. Short equal-length strings get Hamming,
    /// short-medium get Jaro-Winkler, medium get BM25, long get SimHash.
    #[inline]
    pub fn auto() -> Self {
        Self {
            strategy: Strategy::Cascade,
            preprocessor: Some(Preprocessor::default()),
            tier_1: None,
            tier_2: None,
            fallback: None,
            auto_mode: true,
        }
    }

    /// Compare two strings, picking the algorithm based on intent at call time.
    ///
    /// This ignores any configured tiers and runs the intent-selected
    /// algorithm directly.
    #[inline]
    pub fn compare_with_intent(
        &self,
        intent: Intent,
        a: &str,
        b: &str,
    ) -> Result<ComparisonResult, SimiError> {
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
        let algo = resolve_intent(intent, &a, &b);
        let (score, name) = run_algorithm(&algo, &a, &b)?;
        Ok(ComparisonResult {
            score,
            tier: 0,
            algorithm: name,
            fallback_called: false,
            fallback_data: None,
        })
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
    ///
    /// When `auto_mode` is true (created via `SimiFlow::auto()`), the
    /// algorithm is selected per pair based on input lengths.
    #[inline]
    pub fn compare(&self, a: &str, b: &str) -> Result<ComparisonResult, SimiError> {
        if self.auto_mode {
            return self.compare_with_intent(Intent::Auto, a, b);
        }

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

/// Map an intent to an algorithm.
///
/// For `Intent::Auto`, inspects input lengths to choose the best fit.
pub fn resolve_intent(intent: Intent, a: &str, b: &str) -> Algo {
    match intent {
        Intent::Names => Algo::JaroWinkler,
        Intent::Typos => Algo::Levenshtein,
        Intent::Codes => Algo::Hamming,
        Intent::Documents => Algo::Bm25,
        Intent::Deduplication => Algo::SimHashDefault,
        Intent::Auto => auto_select(a, b),
    }
}

/// Auto-detect the best algorithm based on input characteristics.
fn auto_select(a: &str, b: &str) -> Algo {
    let max_len = a.len().max(b.len());
    if a.len() == b.len() && max_len <= 20 {
        return Algo::Hamming;
    }
    if max_len <= 50 {
        return Algo::JaroWinkler;
    }
    if max_len <= 500 {
        return Algo::Bm25;
    }
    Algo::SimHashDefault
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
        let result = SimiFlow::new()
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
        let result = SimiFlow::new()
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
        let result = SimiFlow::new()
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
        let result = SimiFlow::new()
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
        let result = SimiFlow::new()
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
        let result = SimiFlow::new()
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
        let flow = SimiFlow::new()
            .strategy(Strategy::Cascade)
            .tier_1(
                Algo::JaroWinkler,
                Threshold::GreaterThan(0.95),
                Threshold::LessThan(0.10),
            )
            .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.95));

        let result = flow.compare("hello world", "hello world").unwrap();
        assert!((result.score - 1.0).abs() < 0.01);
        assert_eq!(result.tier, 1);
    }

    // ─── Intent-based auto-selection tests ────────────────────────

    #[test]
    fn intent_names_uses_jaro_winkler() {
        let flow = SimiFlow::for_intent(Intent::Names);
        let result = flow.compare("MARTHA", "MARHTA").unwrap();
        assert_eq!(result.algorithm, "jaro_winkler");
        assert!((result.score - 0.961).abs() < 0.01);
    }

    #[test]
    fn intent_typos_uses_levenshtein() {
        let flow = SimiFlow::for_intent(Intent::Typos);
        let result = flow.compare("kitten", "sitting").unwrap();
        assert_eq!(result.algorithm, "levenshtein");
    }

    #[test]
    fn intent_codes_uses_hamming() {
        let flow = SimiFlow::for_intent(Intent::Codes);
        let result = flow.compare("hello", "hello").unwrap();
        assert_eq!(result.algorithm, "hamming");
    }

    #[test]
    fn intent_documents_uses_bm25() {
        let flow = SimiFlow::for_intent(Intent::Documents);
        let result = flow
            .compare("the quick brown fox", "the quick brown fox")
            .unwrap();
        assert_eq!(result.algorithm, "bm25");
    }

    #[test]
    fn intent_deduplication_uses_simhash() {
        let flow = SimiFlow::for_intent(Intent::Deduplication);
        let result = flow
            .compare("the quick brown fox", "the quick brown fox")
            .unwrap();
        assert_eq!(result.algorithm, "simhash");
    }

    #[test]
    fn auto_select_short_picks_hamming() {
        let flow = SimiFlow::auto();
        let result = flow.compare("abc", "abc").unwrap();
        // equal length, <= 20 chars → Hamming
        assert_eq!(result.algorithm, "hamming");
    }

    #[test]
    fn auto_select_medium_picks_jaro_winkler() {
        let flow = SimiFlow::auto();
        // 33 chars, >20 but <=50 → Jaro-Winkler
        let result = flow
            .compare("the quick brown fox", "the quick lazy dog")
            .unwrap();
        assert_eq!(result.algorithm, "jaro_winkler");
    }

    #[test]
    fn auto_select_long_picks_bm25() {
        let flow = SimiFlow::auto();
        let a = "the quick brown fox jumps over the lazy dog near the river bank on a sunny day";
        let b = "the quick brown fox jumps over the lazy cat near the river bank on a rainy day";
        // ~80 chars, >50 but <=500 → BM25
        let result = flow.compare(a, b).unwrap();
        assert_eq!(result.algorithm, "bm25");
    }

    #[test]
    fn compare_with_intent_bypasses_tiers() {
        let flow = SimiFlow::new().tier_1(
            Algo::Levenshtein,
            Threshold::GreaterThan(0.99),
            Threshold::LessThan(0.01),
        );
        let result = flow
            .compare_with_intent(Intent::Names, "MARTHA", "MARHTA")
            .unwrap();
        assert_eq!(result.algorithm, "jaro_winkler");
        assert_eq!(result.tier, 0);
    }

    #[test]
    fn compare_with_intent_all_variants() {
        let flow = SimiFlow::new();
        for (intent, expected_algo, a, b) in [
            (Intent::Names, "jaro_winkler", "MARTHA", "MARHTA"),
            (Intent::Typos, "levenshtein", "kitten", "sitting"),
            (
                Intent::Documents,
                "bm25",
                "the quick brown fox",
                "the quick brown fox",
            ),
            (
                Intent::Deduplication,
                "simhash",
                "hello world",
                "hello world",
            ),
        ] {
            let r = flow.compare_with_intent(intent, a, b).unwrap();
            assert_eq!(r.algorithm, expected_algo, "intent {intent:?}");
            assert_normalized(r.score);
        }
    }

    // ─── Auto-select edge cases ────────────────────────────────────

    #[test]
    fn auto_select_boundary_20_equal() {
        let a = "x".repeat(20);
        let b = "y".repeat(20);
        // equal length, max_len == 20 → Hamming
        assert_eq!(auto_select(&a, &b), Algo::Hamming);
    }

    #[test]
    fn auto_select_boundary_21_jaro_winkler() {
        let a = "x".repeat(21);
        let b = "y".repeat(21);
        // max_len 21, >20, <=50 → Jaro-Winkler
        assert_eq!(auto_select(&a, &b), Algo::JaroWinkler);
    }

    #[test]
    fn auto_select_unequal_short_jaro_winkler() {
        // Different lengths, max_len <= 50 → Jaro-Winkler (not Hamming)
        assert_eq!(auto_select("hello", "world!"), Algo::JaroWinkler);
    }

    #[test]
    fn auto_select_boundary_50() {
        let a = "x".repeat(50);
        let b = "y".repeat(50);
        assert_eq!(auto_select(&a, &b), Algo::JaroWinkler);
    }

    #[test]
    fn auto_select_boundary_51_bm25() {
        let a = "x".repeat(51);
        let b = "y".repeat(51);
        assert_eq!(auto_select(&a, &b), Algo::Bm25);
    }

    #[test]
    fn auto_select_boundary_500() {
        let a = "x".repeat(500);
        let b = "y".repeat(500);
        assert_eq!(auto_select(&a, &b), Algo::Bm25);
    }

    #[test]
    fn auto_select_boundary_501_simhash() {
        let a = "x".repeat(501);
        let b = "y".repeat(501);
        assert_eq!(auto_select(&a, &b), Algo::SimHashDefault);
    }

    #[test]
    fn auto_select_empty_strings() {
        assert_eq!(auto_select("", ""), Algo::Hamming); // equal, 0 <= 20
    }

    #[test]
    fn auto_select_single_char_equal() {
        assert_eq!(auto_select("a", "a"), Algo::Hamming);
    }

    #[test]
    fn auto_select_single_char_unequal() {
        assert_eq!(auto_select("a", "b"), Algo::Hamming); // equal length, <=20
    }

    #[test]
    fn for_intent_all_variants_work() {
        for intent in [
            Intent::Names,
            Intent::Typos,
            Intent::Codes,
            Intent::Documents,
            Intent::Deduplication,
        ] {
            let flow = SimiFlow::for_intent(intent);
            let result = flow.compare("hello", "hello").unwrap();
            assert_normalized(result.score);
            assert!(result.score > 0.9);
        }
    }

    #[test]
    fn for_intent_auto_loads_with_empty_heuristic() {
        // for_intent(Auto) resolves with empty strings → Hamming
        let flow = SimiFlow::for_intent(Intent::Auto);
        let result = flow.compare("hello", "hello").unwrap();
        // but compare() gate may re-resolve… actually for_intent stores
        // the resolved algo at construction time, and compare() with
        // auto_mode=false uses it directly.
        assert_normalized(result.score);
    }

    #[test]
    fn auto_mode_re_resolves_per_pair() {
        let flow = SimiFlow::auto();
        // Short equal → Hamming
        let r1 = flow.compare("abc", "abc").unwrap();
        assert_eq!(r1.algorithm, "hamming");
        // Long equal >500 → SimHash
        let long = "x".repeat(600);
        let r2 = flow.compare(&long, &long).unwrap();
        assert_eq!(r2.algorithm, "simhash");
    }

    fn assert_normalized(score: f64) {
        assert!(score.is_finite() && (0.0..=1.0).contains(&score));
    }
}
