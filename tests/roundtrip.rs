// Roundtrip integration test for the SIMI crate.
//
// This file exercises every public API surface and algorithm end-to-end,
// ensuring the library is coherent, consistent, and free of panics.

use simi::algo::*;
use simi::batch::BatchComparator;
use simi::preprocess::Preprocessor;
use simi::router::{Algo, SimiFlow, Threshold, Intent, resolve_intent};

// ─── Helpers ───────────────────────────────────────────────────────

/// Assert that a score is `f64`-normalized.
fn assert_normalized(score: f64) {
    assert!(score.is_finite(), "score must be finite, got {score}");
    assert!((0.0..=1.0).contains(&score), "score must be in [0,1], got {score}");
}

/// Two strings that are identical across all algorithms.
const IDENTICAL: (&str, &str) = ("hello world", "hello world");

/// Two short strings that are different.
const DIFFERENT: (&str, &str) = ("abc", "xyz");

// ─── Levenshtein ───────────────────────────────────────────────────

#[test]
fn roundtrip_levenshtein_identical() {
    let s = levenshtein::similarity(IDENTICAL.0, IDENTICAL.1);
    assert!((s - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_levenshtein_different() {
    let s = levenshtein::similarity(DIFFERENT.0, DIFFERENT.1);
    assert_normalized(s);
    assert!(s < 0.5);
}

#[test]
fn roundtrip_levenshtein_known() {
    // "kitten" -> "sitting": 3 edits, max(6,7)=7, similarity = 1 - 3/7 = 0.571...
    let s = levenshtein::similarity("kitten", "sitting");
    assert!((s - 0.571).abs() < 0.01, "expected ~0.571, got {s}");
}

#[test]
fn roundtrip_levenshtein_symmetry() {
    let a = levenshtein::similarity("ab", "ba");
    let b = levenshtein::similarity("ba", "ab");
    assert!((a - b).abs() < f64::EPSILON);
}

// ─── Jaro-Winkler ──────────────────────────────────────────────────

#[test]
fn roundtrip_jaro_winkler_identical() {
    let s = jaro_winkler::similarity(IDENTICAL.0, IDENTICAL.1);
    assert!((s - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_jaro_winkler_known() {
    let s = jaro_winkler::similarity("MARTHA", "MARHTA");
    assert!((0.95..=0.97).contains(&s), "expected ~0.961, got {s}");
}

#[test]
fn roundtrip_jaro_winkler_different() {
    let s = jaro_winkler::similarity("abc", "xyz");
    assert_normalized(s);
    assert!(s < 0.2);
}

#[test]
fn roundtrip_jaro_winkler_symmetry() {
    let a = jaro_winkler::similarity("hello", "hallo");
    let b = jaro_winkler::similarity("hallo", "hello");
    assert!((a - b).abs() < f64::EPSILON);
}

// ─── Hamming ───────────────────────────────────────────────────────

#[test]
fn roundtrip_hamming_identical() {
    let s = hamming::similarity("hello", "hello").expect("equal-length strings");
    assert!((s - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_hamming_all_different() {
    let s = hamming::similarity("abc", "xyz").expect("equal-length strings");
    assert!((s).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_hamming_unequal_length() {
    assert!(hamming::similarity("abc", "abcd").is_none());
}

#[test]
fn roundtrip_hamming_known() {
    // "karolin" vs "kathrin": both 7 chars, 3 differ -> similarity 4/7 = 0.571
    let s = hamming::similarity("karolin", "kathrin").expect("equal-length strings");
    assert!((s - 0.57142).abs() < 0.001, "expected ~0.571, got {s}");
}

// ─── Jaccard ───────────────────────────────────────────────────────

#[test]
fn roundtrip_jaccard_identical() {
    let s = jaccard::bigram_similarity(IDENTICAL.0, IDENTICAL.1);
    assert!((s - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_jaccard_different() {
    let s = jaccard::bigram_similarity(DIFFERENT.0, DIFFERENT.1);
    assert_normalized(s);
    assert!(s < 0.2);
}

#[test]
fn roundtrip_jaccard_symmetry() {
    let a = jaccard::bigram_similarity("hello", "world");
    let b = jaccard::bigram_similarity("world", "hello");
    assert!((a - b).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_jaccard_n2_n3() {
    let s1 = jaccard::bigram_similarity("hello", "hallo");
    let s2 = jaccard::trigram_similarity("hello", "hallo");
    assert_normalized(s1);
    assert_normalized(s2);
    // Trigram decomposition yields different values, but both in [0,1].
}

#[test]
fn roundtrip_jaccard_word_level() {
    let s = jaccard::word_similarity("the quick brown fox", "the quick lazy dog");
    assert!((s - 0.333).abs() < 0.01, "expected ~0.333, got {s}");
}

// ─── MinHash ───────────────────────────────────────────────────────

#[test]
fn roundtrip_minhash_identical() {
    let s = minhash::compare_default(IDENTICAL.0, IDENTICAL.1);
    assert_normalized(s);
    assert!(s > 0.9, "MinHash should score identical docs high, got {s}");
}

#[test]
fn roundtrip_minhash_different() {
    let s = minhash::compare_default(DIFFERENT.0, DIFFERENT.1);
    assert_normalized(s);
    assert!(s < 0.5, "MinHash should score unrelated strings low, got {s}");
}

#[test]
fn roundtrip_minhash_shingle_variants() {
    let s3 = minhash::compare("hello world", "hello world", 3, 64);
    let s5 = minhash::compare("hello world", "hello world", 5, 64);
    assert_normalized(s3);
    assert_normalized(s5);
}

// ─── SimHash ───────────────────────────────────────────────────────

#[test]
fn roundtrip_simhash_identical() {
    let s = simhash::compare_default(IDENTICAL.0, IDENTICAL.1);
    assert_normalized(s);
    assert!(s > 0.9, "SimHash should score identical docs high, got {s}");
}

#[test]
fn roundtrip_simhash_different() {
    // SimHash baseline for unrelated docs is ~0.5 (random 64-bit overlap).
    // Assert the score is well below the identical-doc threshold.
    let s = simhash::compare_default("the quick brown fox", "lorem ipsum dolor sit");
    assert_normalized(s);
    assert!(s < 0.65, "SimHash should score unrelated strings low, got {s}");
}

#[test]
fn roundtrip_simhash_shingle_variants() {
    let s2 = simhash::compare("hello world", "hello world", 2);
    let s4 = simhash::compare("hello world", "hello world", 4);
    assert_normalized(s2);
    assert_normalized(s4);
}

// ─── BM25 ──────────────────────────────────────────────────────────

#[test]
fn roundtrip_bm25_identical() {
    let s = bm25::similarity(IDENTICAL.0, IDENTICAL.1);
    assert_normalized(s);
    assert!(s > 0.9, "BM25 should score identical docs high, got {s}");
}

#[test]
fn roundtrip_bm25_different() {
    let s = bm25::similarity(DIFFERENT.0, DIFFERENT.1);
    assert_normalized(s);
    assert!(s < 0.5);
}

#[test]
fn roundtrip_bm25_symmetry() {
    let a = bm25::similarity("the quick brown fox", "the quick red fox");
    let b = bm25::similarity("the quick red fox", "the quick brown fox");
    // BM25 is not strictly symmetric due to IDF weights, but scores stay close.
    assert!((a - b).abs() < 0.15);
}

// ─── TF-IDF ────────────────────────────────────────────────────────

#[test]
fn roundtrip_tfidf_identical() {
    let s = tfidf::similarity(IDENTICAL.0, IDENTICAL.1);
    assert_normalized(s);
    assert!(s > 0.9, "TF-IDF should score identical docs high, got {s}");
}

#[test]
fn roundtrip_tfidf_different() {
    let s = tfidf::similarity(DIFFERENT.0, DIFFERENT.1);
    assert_normalized(s);
    assert!(s < 0.5);
}

#[test]
fn roundtrip_tfidf_symmetry() {
    let a = tfidf::similarity("the quick brown fox", "the quick red fox");
    let b = tfidf::similarity("the quick red fox", "the quick brown fox");
    assert!((a - b).abs() < f64::EPSILON);
}

// ─── Preprocessor ──────────────────────────────────────────────────

#[test]
fn roundtrip_preprocessor_default() {
    let p = Preprocessor::default();
    let result = p.process("  Hello   World  ");
    assert_eq!(result, "hello world");
}

#[test]
fn roundtrip_preprocessor_no_lowercase() {
    let p = Preprocessor::default().with_lowercase(false);
    let result = p.process("  Hello   World  ");
    assert_eq!(result, "Hello World");
}

#[test]
fn roundtrip_preprocessor_with_stopwords() {
    let p = Preprocessor::default().with_remove_stopwords(true);
    let result = p.process("the quick brown fox");
    // "the" is a standard stopword
    assert!(!result.contains("the"), "stopword 'the' not removed: {result}");
    assert!(result.contains("quick"), "expected 'quick' in {result}");
}

#[test]
fn roundtrip_preprocessor_custom_stopwords() {
    let custom = vec!["hello".to_string(), "world".to_string()];
    let p = Preprocessor::default()
        .with_remove_stopwords(true)
        .with_stopwords(custom);
    let result = p.process("hello beautiful world");
    assert_eq!(result, "beautiful");
}

#[test]
fn roundtrip_preprocessor_max_length() {
    let p = Preprocessor::default().with_max_length(5);
    let result = p.process("hello world");
    assert_eq!(result, "hello");
}

#[test]
fn roundtrip_preprocessor_unicode() {
    let p = Preprocessor::default();
    // e + combining acute accent -> NFC should become é
    let result = p.process("\u{0065}\u{0301}");
    assert_eq!(result, "\u{00e9}");
}

// ─── Router / SimiFlow ───────────────────────────────────────────

#[test]
fn roundtrip_flow_tier_1_match() {
    let result = SimiFlow::new()
        .tier_1(
            Algo::JaroWinkler,
            Threshold::GreaterThan(0.95),
            Threshold::LessThan(0.10),
        )
        .compare("MARTHA", "MARHTA")
        .expect("comparison should succeed");
    assert_eq!(result.tier, 1);
    assert!(!result.fallback_called);
    assert_normalized(result.score);
}

#[test]
fn roundtrip_flow_tier_1_mismatch() {
    let result = SimiFlow::new()
        .tier_1(
            Algo::Levenshtein,
            Threshold::GreaterThan(0.95),
            Threshold::LessThan(0.10),
        )
        .compare("abc", "xyz")
        .expect("comparison should succeed");
    assert_eq!(result.tier, 1);
    assert!(result.score < 0.01);
}

#[test]
fn roundtrip_flow_tier_2() {
    let result = SimiFlow::new()
        .tier_1(
            Algo::Levenshtein,
            Threshold::GreaterThan(0.95),
            Threshold::LessThan(0.05),
        )
        .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.95))
        .compare("the quick brown fox", "the quick red fox")
        .expect("comparison should succeed");
    assert_eq!(result.tier, 2);
    assert_normalized(result.score);
}

#[test]
fn roundtrip_flow_fallback_called() {
    let result = SimiFlow::new()
        .tier_1(
            Algo::Levenshtein,
            Threshold::GreaterThan(0.99),
            Threshold::LessThan(0.01),
        )
        .fallback(|a, b| {
            let score = if a == b { 1.0 } else { 0.5 };
            (score, Some("llm_verified".into()))
        })
        .compare("hello", "world")
        .expect("comparison should succeed");
    assert_eq!(result.tier, 3);
    assert!(result.fallback_called);
    assert_eq!(result.fallback_data, Some("llm_verified".into()));
}

#[test]
fn roundtrip_flow_preprocessing() {
    let result = SimiFlow::new()
        .preprocess(true)
        .tier_1(
            Algo::Levenshtein,
            Threshold::GreaterThan(0.95),
            Threshold::LessThan(0.10),
        )
        .compare("  Hello   World  ", "hello world")
        .expect("comparison should succeed");
    assert!((result.score - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_flow_no_tiers_error() {
    let result = SimiFlow::new().compare("a", "b");
    assert!(result.is_err());
}

// ─── Batch Comparator ──────────────────────────────────────────────

#[test]
fn roundtrip_batch_pairs() {
    let a: Vec<String> = vec!["hello".into(), "world".into(), "rust".into()];
    let b: Vec<String> = vec!["hello".into(), "word".into(), "rusty".into()];

    let cmp = BatchComparator::new(Algo::Levenshtein);
    let results = cmp.compare_pairs(&a, &b).expect("batch should succeed");

    assert_eq!(results.len(), 3);
    assert!((results[0].score - 1.0).abs() < f64::EPSILON);
    for r in &results {
        assert_normalized(r.score);
    }
}

#[test]
fn roundtrip_batch_one_to_many() {
    let ref_str = "hello".to_string();
    let candidates: Vec<String> = vec!["hello".into(), "hallo".into(), "world".into()];

    let cmp = BatchComparator::new(Algo::Levenshtein);
    let results = cmp
        .compare_one_to_many(&ref_str, &candidates)
        .expect("batch should succeed");

    assert_eq!(results.len(), 3);
    assert!((results[0].score - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_batch_matrix() {
    let a: Vec<String> = vec!["hello".into(), "world".into()];
    let b: Vec<String> = vec!["hello".into(), "word".into()];

    let cmp = BatchComparator::new(Algo::JaroWinkler);
    let results = cmp.compare_matrix(&a, &b).expect("batch should succeed");

    assert_eq!(results.len(), 4); // 2 × 2
    for r in &results {
        assert_normalized(r.score);
    }
}

#[test]
fn roundtrip_batch_unequal_length() {
    let a = vec!["hello".into()];
    let b = vec!["hello".into(), "world".into()];

    let cmp = BatchComparator::new(Algo::Levenshtein);
    let result = cmp.compare_pairs(&a, &b);
    assert!(result.is_err());
}

#[test]
fn roundtrip_batch_large_parallel() {
    let size = 500;
    let a: Vec<String> = (0..size).map(|i| format!("doc{0:0>5}", i)).collect();
    let b: Vec<String> = (0..size).map(|i| format!("doc{0:0>5}", i + 1)).collect();

    let cmp = BatchComparator::new(Algo::Levenshtein);
    let results = cmp.compare_pairs(&a, &b).expect("batch should succeed");

    assert_eq!(results.len(), size);
    for r in &results {
        assert_normalized(r.score);
    }
}

// ─── Cross-algorithm consistency ───────────────────────────────────

#[test]
fn roundtrip_cross_algo_identical() {
    // All algorithms should report maximum similarity for identical inputs.
    let a = "hello world";
    let b = "hello world";

    assert!((levenshtein::similarity(a, b) - 1.0).abs() < f64::EPSILON);
    assert!((jaro_winkler::similarity(a, b) - 1.0).abs() < f64::EPSILON);
    assert!((jaccard::bigram_similarity(a, b) - 1.0).abs() < f64::EPSILON);
    assert!((jaccard::word_similarity(a, b) - 1.0).abs() < f64::EPSILON);
    // Probabilistic algorithms
    assert!(minhash::compare_default(a, b) > 0.9);
    assert!(simhash::compare_default(a, b) > 0.9);
    assert!(bm25::similarity(a, b) > 0.9);
    assert!(tfidf::similarity(a, b) > 0.9);
}

#[test]
fn roundtrip_cross_algo_different() {
    // All algorithms should report low similarity for different strings.
    // Shorter strings for edit-distance algorithms.
    let a = "abc";
    let b = "xyz";
    assert!(levenshtein::similarity(a, b) < 0.5);
    assert!(jaro_winkler::similarity(a, b) < 0.5);

    // Longer strings for document-oriented algorithms.
    let a = "the quick brown fox jumps over the lazy dog";
    let b = "lorem ipsum dolor sit amet consectetur adipiscing elit";
    assert!(jaccard::bigram_similarity(a, b) < 0.5);
    assert!(minhash::compare_default(a, b) < 0.5);
    assert!(simhash::compare_default(a, b) < 0.65); // probabilistic baseline ~0.5
    assert!(bm25::similarity(a, b) < 0.5);
    assert!(tfidf::similarity(a, b) < 0.5);
}

#[test]
fn roundtrip_all_scores_normalized() {
    let pairs = [
        ("hello", "hello"),
        ("hello", "world"),
        ("the quick brown fox", "the quick brown fox"),
        ("the quick brown fox", "the lazy dog"),
        ("MARTHA", "MARHTA"),
        ("kitten", "sitting"),
        ("", ""),
        ("hello", ""),
        ("abc", "xyz"),
    ];

    for (a, b) in &pairs {
        assert_normalized(levenshtein::similarity(a, b));
        assert_normalized(jaro_winkler::similarity(a, b));
        if a.len() == b.len() {
            if let Some(s) = hamming::similarity(a, b) {
                assert_normalized(s);
            }
        }
        assert_normalized(jaccard::bigram_similarity(a, b));
        assert_normalized(minhash::compare_default(a, b));
        assert_normalized(simhash::compare_default(a, b));
        assert_normalized(bm25::similarity(a, b));
        assert_normalized(tfidf::similarity(a, b));
    }
}

// ─── Edge cases ────────────────────────────────────────────────────

#[test]
fn roundtrip_empty_strings() {
    let s = levenshtein::similarity("", "");
    assert!((s - 1.0).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_empty_vs_nonempty() {
    let s = levenshtein::similarity("", "hello");
    assert_normalized(s);
    assert!((s).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_single_character() {
    let s = levenshtein::similarity("a", "a");
    assert!((s - 1.0).abs() < f64::EPSILON);
    let s = levenshtein::similarity("a", "b");
    assert!((s).abs() < f64::EPSILON);
}

#[test]
fn roundtrip_long_strings() {
    let a = "a".repeat(1000);
    let b = "a".repeat(999) + "b";
    let s = levenshtein::similarity(&a, &b);
    assert_normalized(s);
    assert!(s > 0.99);
}

#[test]
fn roundtrip_unicode_strings() {
    let s = levenshtein::similarity("café", "cafe");
    assert_normalized(s);
    let s = jaro_winkler::similarity("café", "café");
    assert!((s - 1.0).abs() < f64::EPSILON);
}

// ─── Intent / SimiFlow auto ──────────────────────────────────────

#[test]
fn roundtrip_flow_for_intent_names() {
    let flow = SimiFlow::for_intent(Intent::Names);
    let r = flow.compare("MARTHA", "MARHTA").unwrap();
    assert_eq!(r.algorithm, "jaro_winkler");
    assert!((r.score - 0.961).abs() < 0.01);
}

#[test]
fn roundtrip_flow_for_intent_typos() {
    let flow = SimiFlow::for_intent(Intent::Typos);
    let r = flow.compare("kitten", "sitting").unwrap();
    assert_eq!(r.algorithm, "levenshtein");
}

#[test]
fn roundtrip_flow_for_intent_documents() {
    let flow = SimiFlow::for_intent(Intent::Documents);
    let r = flow.compare("the quick brown fox", "the quick brown fox").unwrap();
    assert_eq!(r.algorithm, "bm25");
    assert!(r.score > 0.9);
}

#[test]
fn roundtrip_flow_for_intent_deduplication() {
    let flow = SimiFlow::for_intent(Intent::Deduplication);
    let r = flow.compare("the quick brown fox", "the quick brown fox").unwrap();
    assert_eq!(r.algorithm, "simhash");
    assert!(r.score > 0.9);
}

#[test]
fn roundtrip_flow_auto_re_resolves() {
    let flow = SimiFlow::auto();
    // Short -> Hamming
    let r = flow.compare("abc", "abc").unwrap();
    assert_eq!(r.algorithm, "hamming");
    // Long -> SimHash
    let long = "x".repeat(600);
    let r = flow.compare(&long, &long).unwrap();
    assert_eq!(r.algorithm, "simhash");
}

#[test]
fn roundtrip_flow_compare_with_intent() {
    let flow = SimiFlow::new();
    for (intent, algo, a, b) in [
        (Intent::Names, "jaro_winkler", "MARTHA", "MARHTA"),
        (Intent::Typos, "levenshtein", "kitten", "sitting"),
        (Intent::Documents, "bm25", "the quick brown fox", "the quick brown fox"),
        (Intent::Deduplication, "simhash", "hello world", "hello world"),
    ] {
        let r = flow.compare_with_intent(intent, a, b).unwrap();
        assert_eq!(r.algorithm, algo);
        assert_eq!(r.tier, 0);
        assert_normalized(r.score);
    }
}

#[test]
fn roundtrip_resolve_intent_consistency() {
    let pairs = [
        ("abc", "xyz"),
        ("hello", "world"),
        ("the quick brown fox", "the quick brown fox"),
    ];
    for (a, b) in pairs {
        let algo = resolve_intent(Intent::Auto, a, b);
        let (score, _) = run_algorithm(&algo, a, b).unwrap();
        assert_normalized(score);
    }
}

// ─── Batch with Intent ──────────────────────────────────────────────

#[test]
fn roundtrip_batch_for_intent_names() {
    let names: Vec<String> = vec!["MARTHA".into(), "JOHN".into(), "NIKOLA".into()];
    let refs: Vec<String> = vec!["MARHTA".into(), "JON".into(), "NICOLA".into()];
    let cmp = BatchComparator::for_intent(Intent::Names);
    let results = cmp.compare_pairs(&names, &refs).unwrap();
    assert_eq!(results.len(), 3);
    for r in &results {
        assert_normalized(r.score);
    }
    assert!(results[0].score > 0.9);
}

#[test]
fn roundtrip_batch_for_intent_deduplication_matrix() {
    let docs: Vec<String> = (0..10).map(|i| format!("doc text number {i} with some content")).collect();
    let cmp = BatchComparator::for_intent(Intent::Deduplication);
    let results = cmp.compare_matrix(&docs, &docs).unwrap();
    assert_eq!(results.len(), 100);
    for r in &results {
        assert_normalized(r.score);
    }
}

#[test]
fn roundtrip_batch_auto_one_to_many() {
    let ref_str = "the quick brown fox".to_string();
    let candidates: Vec<String> = (0..20).map(|i| format!("candidate text number {i}")).collect();
    let cmp = BatchComparator::auto();
    let results = cmp.compare_one_to_many(&ref_str, &candidates).unwrap();
    assert_eq!(results.len(), 20);
    for r in &results {
        assert_normalized(r.score);
    }
}

fn run_algorithm(algo: &Algo, a: &str, b: &str) -> Result<(f64, String), simi::SimiError> {
    use simi::algo::*;
    match algo {
        Algo::Levenshtein => Ok((levenshtein::similarity(a, b), "levenshtein".into())),
        Algo::JaroWinkler => Ok((jaro_winkler::similarity(a, b), "jaro_winkler".into())),
        Algo::Hamming => hamming::similarity(a, b)
            .map(|s| (s, "hamming".into()))
            .ok_or_else(|| simi::SimiError::AlgorithmError("hamming error".into())),
        Algo::Bm25 => Ok((bm25::similarity(a, b), "bm25".into())),
        Algo::SimHashDefault => Ok((simhash::compare_default(a, b), "simhash".into())),
        other => Err(simi::SimiError::AlgorithmError(format!("unsupported: {other:?}"))),
    }
}
