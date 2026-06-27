//! Jaro-Winkler Distance — exceptionally good for name matching as it
//! heavily weights matching prefixes. Normalized to `[0.0, 1.0]`.
//!
//! ## Performance
//! O(n·m) time, O(n + m) memory for the matching window.

/// Compute the raw Jaro similarity between two strings.
///
/// Returns a value in `[0.0, 1.0]`.
#[inline]
pub fn jaro_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    // Matching window: floor(max(len_a, len_b) / 2) - 1
    let match_dist = (std::cmp::max(a_len, b_len) / 2).saturating_sub(1);

    let mut a_matched = vec![false; a_len];
    let mut b_matched = vec![false; b_len];

    let mut matches = 0usize;
    let mut transpositions = 0usize;

    // Find matches within the window
    for i in 0..a_len {
        let start = i.saturating_sub(match_dist);
        let end = std::cmp::min(i + match_dist + 1, b_len);

        for j in start..end {
            if b_matched[j] {
                continue;
            }
            if a_chars[i] != b_chars[j] {
                continue;
            }
            a_matched[i] = true;
            b_matched[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions (order of matched characters)
    let mut b_index = 0;
    for i in 0..a_len {
        if !a_matched[i] {
            continue;
        }
        while b_index < b_len && !b_matched[b_index] {
            b_index += 1;
        }
        if b_index < b_len && a_chars[i] != b_chars[b_index] {
            transpositions += 1;
        }
        b_index += 1;
    }
    transpositions /= 2;

    let m = matches as f64;
    let t = transpositions as f64;

    (m / a_len as f64 + m / b_len as f64 + (m - t) / m) / 3.0
}

/// Jaro-Winkler similarity with prefix boost.
///
/// The standard Winkler scaling factor of `0.1` is used, with a
/// maximum prefix length of `4` characters.
#[inline]
pub fn similarity(a: &str, b: &str) -> f64 {
    let jaro = jaro_similarity(a, b);

    // Count common prefix (max 4 chars)
    let prefix_len = a
        .chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count()
        .min(4);

    // Winkler adjustment: similarity += prefix_len * 0.1 * (1 - jaro)
    jaro + prefix_len as f64 * 0.1 * (1.0 - jaro)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings() {
        assert!((similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn completely_different() {
        let s = similarity("abc", "xyz");
        assert!(s < 0.01, "expected near 0, got {s}");
    }

    #[test]
    fn classic_names() {
        // MARTHA vs MARHTA is the canonical Jaro-Winkler example
        let s = similarity("MARTHA", "MARHTA");
        assert!((s - 0.961).abs() < 0.01, "got {s}");
    }

    #[test]
    fn prefix_boost() {
        // Strings sharing a common prefix get a boost
        let no_prefix = jaro_similarity("abcde", "abfde");
        let with_prefix = similarity("abcde", "abfde");
        assert!(
            with_prefix > no_prefix,
            "Jaro-Winkler should be higher due to prefix boost"
        );
    }

    #[test]
    fn empty_strings() {
        assert_eq!(similarity("", ""), 1.0);
        assert_eq!(similarity("abc", ""), 0.0);
    }

    #[test]
    fn symmetric() {
        let a = similarity("dwayne", "duane");
        let b = similarity("duane", "dwayne");
        assert!((a - b).abs() < f64::EPSILON);
    }
}
