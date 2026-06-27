//! Levenshtein Distance — minimum single-character edits to change one
//! string into another. Normalized to `[0.0, 1.0]` where `1.0` = identical.
//!
//! ## Performance
//! O(min(n,m) · max(n,m)) time, O(min(n,m)) memory (single-row DP).

/// Compute the Levenshtein edit distance between two strings.
///
/// Uses a single-row space optimization for O(min(n,m)) memory.
#[inline]
pub fn distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    // Early exit for identical strings
    if a == b {
        return 0;
    }

    // Ensure a is the shorter string for space efficiency
    if a_len > b_len {
        return distance(b, a);
    }

    // Single-row DP — only need one row of size a_len + 1
    let mut prev_row: Vec<usize> = (0..=a_len).collect();

    for (j, bc) in b.chars().enumerate() {
        // First element: distance from empty string to b[..j+1]
        let mut prev = j + 1;
        let mut current;

        for (i, ac) in a.chars().enumerate() {
            let cost = if ac == bc { 0 } else { 1 };
            current = std::cmp::min(
                std::cmp::min(prev + 1, prev_row[i + 1] + 1),
                prev_row[i] + cost,
            );
            prev_row[i] = prev;
            prev = current;
        }
        prev_row[a_len] = prev;
    }

    prev_row[a_len]
}

/// Normalized Levenshtein similarity in `[0.0, 1.0]`.
///
/// `1.0` means identical strings, `0.0` means completely different.
/// Similarity = 1 - (distance / max_len)
#[inline]
pub fn similarity(a: &str, b: &str) -> f64 {
    let max_len = std::cmp::max(a.chars().count(), b.chars().count());
    if max_len == 0 {
        return 1.0; // Both empty
    }
    1.0 - (distance(a, b) as f64 / max_len as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings() {
        assert_eq!(similarity("hello", "hello"), 1.0);
        assert_eq!(distance("hello", "hello"), 0);
    }

    #[test]
    fn completely_different() {
        let s = similarity("abc", "xyz");
        assert!(s < 0.01, "expected near 0, got {s}");
    }

    #[test]
    fn one_edit_away() {
        assert_eq!(distance("kitten", "sitten"), 1);
        let s = similarity("kitten", "sitten");
        assert!(s > 0.8 && s < 1.0);
    }

    #[test]
    fn classic_example() {
        // kitten -> sitten (sub 's'), sitten -> sittin (sub 'i'),
        // sittin -> sitting (add 'g')
        assert_eq!(distance("kitten", "sitting"), 3);
        let s = similarity("kitten", "sitting");
        assert!((s - 0.571).abs() < 0.01, "got {s}");
    }

    #[test]
    fn empty_strings() {
        assert_eq!(similarity("", ""), 1.0);
        assert_eq!(distance("", ""), 0);
        assert_eq!(similarity("abc", ""), 0.0);
    }

    #[test]
    fn symmetric() {
        let a = similarity("saturday", "sunday");
        let b = similarity("sunday", "saturday");
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn unicode_characters() {
        let s = similarity("café", "cafe");
        assert!(s > 0.0 && s < 1.0);
    }
}
