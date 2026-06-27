//! SimHash — locality-sensitive hash for deduplication of large documents.
//!
//! Generates a 64-bit fingerprint where similar documents have similar
//! fingerprints. Similarity is measured by the Hamming distance between
//! fingerprints.
//!
//! ## Performance
//! O(n) time where n = number of features. O(1) memory (single u64).

/// A 64-bit SimHash fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimHashFingerprint(pub u64);

const DEFAULT_SHINGLE_SIZE: usize = 4;

/// Generate a SimHash fingerprint for a text.
///
/// Uses character-level 4-grams (tetragrams) as features by default.
/// Each feature is hashed, and the bits are accumulated weighted by
/// feature frequency.
#[inline]
pub fn fingerprint(text: &str, shingle_size: usize) -> SimHashFingerprint {
    let chars: Vec<char> = text.chars().collect();
    let mut shingles = Vec::new();

    if chars.len() < shingle_size {
        shingles.push(text.to_string());
    } else {
        // Collect shingles with their frequencies
        for w in chars.windows(shingle_size) {
            shingles.push(w.iter().collect::<String>());
        }
    }

    // SimHash: accumulate +1/-1 for each bit position
    let mut v = [0i64; 64];

    for shingle in &shingles {
        let h = fxhash64(shingle);
        for (i, val) in v.iter_mut().enumerate() {
            if (h >> i) & 1 == 1 {
                *val += 1;
            } else {
                *val -= 1;
            }
        }
    }

    // Compress to 64-bit fingerprint
    let mut fp: u64 = 0;
    for (i, val) in v.iter().enumerate() {
        if *val > 0 {
            fp |= 1 << i;
        }
    }

    SimHashFingerprint(fp)
}

/// Default fingerprint with tetragram shingles.
#[inline]
pub fn fingerprint_default(text: &str) -> SimHashFingerprint {
    fingerprint(text, DEFAULT_SHINGLE_SIZE)
}

/// Hamming distance between two SimHash fingerprints.
///
/// Returns the number of differing bits.
#[inline]
pub fn hamming_distance(a: SimHashFingerprint, b: SimHashFingerprint) -> u32 {
    (a.0 ^ b.0).count_ones()
}

/// Normalized similarity from two SimHash fingerprints.
///
/// Returns `1.0 - (hamming_distance / 64)` in `[0.0, 1.0]`.
#[inline]
pub fn similarity(a: SimHashFingerprint, b: SimHashFingerprint) -> f64 {
    1.0 - hamming_distance(a, b) as f64 / 64.0
}

/// One-shot: hash both strings and compare.
#[inline]
pub fn compare(a: &str, b: &str, shingle_size: usize) -> f64 {
    let fp_a = fingerprint(a, shingle_size);
    let fp_b = fingerprint(b, shingle_size);
    similarity(fp_a, fp_b)
}

/// Default one-shot with tetragram shingles.
#[inline]
pub fn compare_default(a: &str, b: &str) -> f64 {
    compare(a, b, DEFAULT_SHINGLE_SIZE)
}

// FNV-1a 64-bit hash
fn fxhash64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_documents() {
        let s = compare_default("hello world this is a test", "hello world this is a test");
        assert!(
            (s - 1.0).abs() < f64::EPSILON,
            "identical should be 1.0, got {s}"
        );
    }

    #[test]
    fn similar_documents() {
        let s = compare_default(
            "the quick brown fox jumps over the lazy dog",
            "the quick brown fox jumps over the lazy cat",
        );
        assert!(s > 0.75, "expected high similarity, got {s}");
    }

    #[test]
    fn different_documents() {
        let fp_a = fingerprint_default("completely unrelated content one");
        let fp_b = fingerprint_default("totally different text here two");
        let d = hamming_distance(fp_a, fp_b);
        assert!(d > 20, "expected at least 20 bits different, got {d}");
    }

    #[test]
    fn symmetric() {
        let a = compare_default("abc def", "abc xyz");
        let b = compare_default("abc xyz", "abc def");
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn small_text() {
        let fp = fingerprint("hi", 4);
        // Single shingle < 4 chars just uses the whole text
        assert_ne!(fp.0, 0);
    }
}
