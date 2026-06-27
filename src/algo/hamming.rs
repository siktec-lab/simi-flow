//! Hamming Distance — counts positions where corresponding symbols differ
//! for strings of equal length. Normalized to `[0.0, 1.0]`.
//!
//! ## Performance
//! O(n) time, O(1) memory.

/// Compute the Hamming distance between two equal-length strings.
///
/// Returns `None` if the strings have different lengths.
#[inline]
pub fn distance(a: &str, b: &str) -> Option<usize> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.len() != b_chars.len() {
        return None;
    }

    let dist = a_chars
        .iter()
        .zip(b_chars.iter())
        .filter(|(ca, cb)| ca != cb)
        .count();

    Some(dist)
}

/// Normalized Hamming similarity in `[0.0, 1.0]`.
///
/// Returns `None` if strings have different lengths.
/// `1.0` means identical, `0.0` means every position differs.
#[inline]
pub fn similarity(a: &str, b: &str) -> Option<f64> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len = a_chars.len();

    if len != b_chars.len() {
        return None;
    }

    if len == 0 {
        return Some(1.0);
    }

    let dist = a_chars
        .iter()
        .zip(b_chars.iter())
        .filter(|(ca, cb)| ca != cb)
        .count();

    Some(1.0 - dist as f64 / len as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings() {
        assert_eq!(similarity("hello", "hello"), Some(1.0));
        assert_eq!(distance("hello", "hello"), Some(0));
    }

    #[test]
    fn one_difference() {
        assert_eq!(distance("karolin", "kathrin"), Some(3));
        let s = similarity("karolin", "kathrin").unwrap();
        assert!((s - 0.571).abs() < 0.01, "got {s}");
    }

    #[test]
    fn all_different() {
        assert_eq!(distance("abc", "xyz"), Some(3));
        assert_eq!(similarity("abc", "xyz"), Some(0.0));
    }

    #[test]
    fn different_lengths() {
        assert_eq!(distance("abc", "abcd"), None);
        assert_eq!(similarity("abc", "abcd"), None);
    }

    #[test]
    fn empty_strings() {
        assert_eq!(similarity("", ""), Some(1.0));
        assert_eq!(distance("", ""), Some(0));
    }

    #[test]
    fn unicode() {
        let s = similarity("café", "café").unwrap();
        assert!((s - 1.0).abs() < f64::EPSILON);
    }
}
