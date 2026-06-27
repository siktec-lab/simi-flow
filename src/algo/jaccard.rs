//! Jaccard Similarity — measures similarity between two sets of words or
//! n-grams. Normalized to `[0.0, 1.0]`.
//!
//! $J(A,B) = |A ∩ B| / |A ∪ B|$
//!
//! ## Performance
//! O(n + m) time, O(min(n,m)) memory for set construction.

use std::collections::HashSet;

/// Tokenize a string into a set of n-grams (character-level).
///
/// Default n-gram size is 2 (bigrams), which performs well for most
/// text similarity use cases.
#[inline]
pub fn ngram_set(s: &str, n: usize) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < n {
        let mut set = HashSet::new();
        set.insert(s.to_string());
        return set;
    }
    chars
        .windows(n)
        .map(|w| w.iter().collect::<String>())
        .collect()
}

/// Jaccard similarity between two strings using n-gram sets.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` means identical n-gram sets.
#[inline]
pub fn similarity(a: &str, b: &str, n: usize) -> f64 {
    if a == b {
        return 1.0;
    }

    let set_a = ngram_set(a, n);
    let set_b = ngram_set(b, n);

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    let intersection_size = set_a.intersection(&set_b).count();
    let union_size = set_a.union(&set_b).count();

    intersection_size as f64 / union_size as f64
}

/// Convenience function using bigrams (n=2).
#[inline]
pub fn bigram_similarity(a: &str, b: &str) -> f64 {
    similarity(a, b, 2)
}

/// Convenience function using trigrams (n=3).
#[inline]
pub fn trigram_similarity(a: &str, b: &str) -> f64 {
    similarity(a, b, 3)
}

/// Word-level Jaccard similarity.
///
/// Splits on whitespace and compares word sets.
#[inline]
pub fn word_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<&str> = a.split_whitespace().collect();
    let set_b: HashSet<&str> = b.split_whitespace().collect();

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    let intersection_size = set_a.intersection(&set_b).count();
    let union_size = set_a.union(&set_b).count();

    intersection_size as f64 / union_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings() {
        let s = similarity("hello world", "hello world", 2);
        assert!((s - 1.0).abs() < f64::EPSILON, "got {s}");
    }

    #[test]
    fn completely_different() {
        let s = similarity("abc", "xyz", 2);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn partial_overlap() {
        let s = similarity("hello world", "hello there", 2);
        assert!(s > 0.0 && s < 1.0, "expected partial, got {s}");
    }

    #[test]
    fn bigram_vs_trigram() {
        let bi = bigram_similarity("hello", "hallo");
        let tri = trigram_similarity("hello", "hallo");
        assert!(
            bi > tri,
            "bigrams should give higher similarity than trigrams"
        );
    }

    #[test]
    fn word_similarity_test() {
        let s = super::word_similarity("the quick brown fox", "the quick lazy dog");
        assert!((s - 0.333).abs() < 0.01, "got {s}");
    }

    #[test]
    fn empty_strings() {
        assert!((similarity("", "", 2) - 1.0).abs() < f64::EPSILON);
    }
}
